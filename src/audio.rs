use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

#[derive(Debug, Clone, PartialEq)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

pub struct AudioEngine {
    /// Keep stream alive — dropping it stops all audio immediately.
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
    pub state: PlayState,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: bool,

    track_duration: Duration,
    play_start: Option<Instant>,
    pause_offset: Duration,
    current_path: Option<PathBuf>,

    /// Grace period after calling play() before is_finished() can fire.
    /// Without this, the sink is momentarily empty right after play() and
    /// auto-advance would fire immediately, skipping to the next track.
    play_grace_until: Option<Instant>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(|e| format!("Audio output error: {e}"))?;

        Ok(AudioEngine {
            _stream: stream,
            handle,
            sink: None,
            state: PlayState::Stopped,
            volume: 1.0,
            shuffle: false,
            repeat: false,
            track_duration: Duration::ZERO,
            play_start: None,
            pause_offset: Duration::ZERO,
            current_path: None,
            play_grace_until: None,
        })
    }

    /// Load and immediately play a file.
    pub fn play(&mut self, path: &PathBuf, duration_secs: u64) -> Result<(), String> {
        // Stop and drop any existing sink first
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }

        let file   = File::open(path).map_err(|e| format!("Cannot open file: {e}"))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Cannot decode audio: {e}"))?;

        let sink = Sink::try_new(&self.handle)
            .map_err(|e| format!("Cannot create sink: {e}"))?;
        sink.set_volume(self.volume);
        sink.append(source);

        self.sink            = Some(sink);
        self.state           = PlayState::Playing;
        self.track_duration  = Duration::from_secs(duration_secs);
        self.play_start      = Some(Instant::now());
        self.pause_offset    = Duration::ZERO;
        self.current_path    = Some(path.clone());
        // Give the sink 300 ms before we start polling is_finished()
        self.play_grace_until = Some(Instant::now() + Duration::from_millis(300));

        Ok(())
    }

    pub fn pause(&mut self) {
        if self.state == PlayState::Playing {
            if let Some(sink) = &self.sink {
                sink.pause();
                if let Some(start) = self.play_start.take() {
                    self.pause_offset += start.elapsed();
                }
            }
            self.state = PlayState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == PlayState::Paused {
            if let Some(sink) = &self.sink {
                sink.play();
                self.play_start = Some(Instant::now());
            }
            self.state = PlayState::Playing;
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            PlayState::Playing => self.pause(),
            PlayState::Paused  => self.resume(),
            PlayState::Stopped => {}
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.state            = PlayState::Stopped;
        self.play_start       = None;
        self.pause_offset     = Duration::ZERO;
        self.play_grace_until = None;
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn seek(&mut self, secs: f64) {
        if let Some(sink) = &self.sink {
            let target = Duration::from_secs_f64(secs.max(0.0));
            let _ = sink.try_seek(target);
            self.pause_offset    = target;
            // Extend grace period after seeking so is_finished() doesn't misfire
            self.play_grace_until = Some(Instant::now() + Duration::from_millis(400));
            if self.state == PlayState::Playing {
                self.play_start = Some(Instant::now());
            }
        }
    }

    /// Returns `(elapsed_secs, total_secs)`.
    pub fn position(&self) -> (f64, f64) {
        let total   = self.track_duration.as_secs_f64();
        let elapsed = match self.state {
            PlayState::Playing => {
                let since = self.play_start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO);
                (self.pause_offset + since).as_secs_f64()
            }
            PlayState::Paused  => self.pause_offset.as_secs_f64(),
            PlayState::Stopped => 0.0,
        };
        (elapsed.min(total), total)
    }

    /// True only if the current track has naturally finished playing.
    pub fn is_finished(&self) -> bool {
        if self.state != PlayState::Playing {
            return false;
        }
        // Don't report finished during the grace period right after play()/seek()
        if let Some(grace) = self.play_grace_until {
            if Instant::now() < grace {
                return false;
            }
        }
        self.sink.as_ref().map(|s| s.empty()).unwrap_or(false)
    }
}
