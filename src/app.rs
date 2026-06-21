use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui;
use poll_promise::Promise;

use crate::audio::{AudioEngine, PlayState};
use crate::library::{Library, Track};
use crate::persist::PersistState;
use crate::search;
use crate::theme::TEXT_PRIMARY;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Welcome,
    Player,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NavSection {
    Albums,
    Artists,
    Favorites,
    Tracks,
    Playlists,
    Music,
}

pub struct MusiqApp {
    pub screen: Screen,

    pub input: String,
    pub search_query: String,

    pub library: Library,
    pub library_folder: Option<PathBuf>,
    pub scanning: bool,
    pub scan_files_done: usize,

    pub nav_selected: NavSection,
    pub nav_expanded: bool,
    pub playlists: Vec<String>,

    pub engine: AudioEngine,
    pub current_track_idx: Option<usize>,
    pub scroll_to_track: Option<usize>,

    pub volume: f32,
    pub progress: f64,
    pub viz_phase: f32,
    pub scrubber_hovered: bool,

    pub album_art_cache: HashMap<usize, egui::TextureHandle>,

    pub status_message: Option<(String, std::time::Instant)>,
    pub error: Option<String>,

    pub is_maximized: bool,

    pub folder_promise: Option<Promise<Option<PathBuf>>>,
    pub scan_promise: Option<Promise<Library>>,
    pub recent_folders: Vec<PathBuf>,

    /// Persistent state — saved to ~/.config/musiq/state.toml
    pub persist: PersistState,

    // Simple PRNG state for shuffle
    shuffle_seed: u64,
}

impl MusiqApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let engine = AudioEngine::new().unwrap_or_else(|_| {
            panic!("Could not open audio output. Make sure ALSA/PulseAudio is running.");
        });

        crate::theme::apply(&cc.egui_ctx);

        let persist = PersistState::load();

        // Seed from current time
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(12345);

        // Restore recent folders from persist
        let recent_folders = persist.recent_paths();
        let volume = persist.volume.unwrap_or(1.0);

        let mut app = Self {
            screen: Screen::Welcome,
            input: String::new(),
            search_query: String::new(),
            library: Library::default(),
            library_folder: None,
            scanning: false,
            scan_files_done: 0,
            nav_selected: NavSection::Tracks,
            nav_expanded: true,
            playlists: vec!["Chill Mix".into(), "Night Drive".into()],
            engine,
            current_track_idx: None,
            scroll_to_track: None,
            volume,
            progress: 0.0,
            viz_phase: 0.0,
            scrubber_hovered: false,
            album_art_cache: HashMap::new(),
            status_message: None,
            error: None,
            is_maximized: false,
            folder_promise: None,
            scan_promise: None,
            recent_folders,
            persist,
            shuffle_seed: seed,
        };
        app.engine.set_volume(app.volume);

        // Auto-reopen last folder
        if let Some(last) = app.persist.last_folder.clone() {
            let p = PathBuf::from(&last);
            if p.exists() && p.is_dir() {
                app.scan_path(p);
            }
        }

        app
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_track_idx.and_then(|i| self.library.tracks.get(i))
    }

    pub fn track_title(&self) -> String {
        match &self.current_track() {
            Some(t) => format!("musiq — {} · {}", t.artist, t.title),
            None => "musiq".to_string(),
        }
    }

    pub fn queue(&self) -> Vec<usize> {
        (0..self.library.tracks.len()).collect()
    }

    pub fn filtered_tracks(&self) -> Vec<usize> {
        search::ranked_search(&self.library.tracks, &self.search_query)
    }

    pub fn play_count(&self, idx: usize) -> u32 {
        if let Some(t) = self.library.tracks.get(idx) {
            self.persist.play_count_for(&t.path)
        } else {
            0
        }
    }

    pub fn add_recent(&mut self, path: PathBuf) {
        self.recent_folders.retain(|p| p != &path);
        self.recent_folders.insert(0, path.clone());
        if self.recent_folders.len() > 8 {
            self.recent_folders.truncate(8);
        }
        self.persist.push_recent(&path);
    }

    pub fn set_status<S: Into<String>>(&mut self, msg: S) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn select_track(&mut self, idx: usize) {
        self.current_track_idx = Some(idx);
        self.scroll_to_track   = Some(idx);
        self.play_current();
    }

    fn play_current(&mut self) {
        if let Some(idx) = self.current_track_idx {
            if let Some(track) = self.library.tracks.get(idx) {
                let path   = track.path.clone();
                let dur    = track.duration_secs;
                let status = format!("{} — {}", track.artist, track.title);
                if let Err(e) = self.engine.play(&path, dur) {
                    self.error = Some(e);
                } else {
                    self.set_status(status);
                    self.persist.record_play(&path);
                }
            }
        }
    }

    pub fn play_pause(&mut self) {
        if self.engine.state == PlayState::Stopped {
            let q = self.queue();
            if let Some(&first) = q.first() {
                self.current_track_idx = Some(first);
                self.scroll_to_track   = Some(first);
                self.play_current();
            }
        } else {
            self.engine.toggle_pause();
        }
    }

    pub fn play_next(&mut self) {
        let q = self.queue();
        if q.is_empty() {
            self.engine.stop();
            return;
        }
        if let Some(idx) = self.next_queue_idx(&q) {
            self.current_track_idx = Some(idx);
            self.scroll_to_track   = Some(idx);
            self.play_current();
        } else {
            self.engine.stop();
        }
    }

    pub fn play_previous(&mut self) {
        let q = self.queue();
        let (elapsed, _) = self.engine.position();
        if elapsed > 3.0 {
            self.engine.seek(0.0);
            return;
        }
        if let Some(idx) = self.prev_queue_idx(&q) {
            self.current_track_idx = Some(idx);
            self.scroll_to_track   = Some(idx);
            self.play_current();
        }
    }

    pub fn stop(&mut self) {
        self.engine.stop();
    }

    pub fn seek_to(&mut self, ratio: f64) {
        let (_, total) = self.engine.position();
        if total > 0.0 {
            self.engine.seek(ratio * total);
            self.progress = ratio;
        }
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
        self.engine.set_volume(self.volume);
        self.persist.volume = Some(self.volume);
        self.persist.save();
    }

    pub fn toggle_shuffle(&mut self) {
        self.engine.shuffle = !self.engine.shuffle;
        self.set_status(if self.engine.shuffle { "Shuffle on" } else { "Shuffle off" });
    }

    pub fn toggle_repeat(&mut self) {
        self.engine.repeat = !self.engine.repeat;
        self.set_status(if self.engine.repeat { "Repeat on" } else { "Repeat off" });
    }

    pub fn dismiss_error(&mut self) {
        self.error = None;
    }

    fn next_queue_idx(&mut self, q: &[usize]) -> Option<usize> {
        if q.is_empty() { return None; }
        if self.engine.shuffle {
            self.shuffle_seed ^= self.shuffle_seed << 13;
            self.shuffle_seed ^= self.shuffle_seed >> 7;
            self.shuffle_seed ^= self.shuffle_seed << 17;
            let mut candidate = (self.shuffle_seed as usize) % q.len();
            if q.len() > 1 && Some(q[candidate]) == self.current_track_idx {
                self.shuffle_seed ^= self.shuffle_seed << 13;
                self.shuffle_seed ^= self.shuffle_seed >> 7;
                self.shuffle_seed ^= self.shuffle_seed << 17;
                candidate = (self.shuffle_seed as usize) % q.len();
            }
            return Some(q[candidate]);
        }
        let pos = self.current_track_idx.and_then(|ci| q.iter().position(|&x| x == ci));
        match pos {
            Some(p) if p + 1 < q.len() => Some(q[p + 1]),
            _ => None,
        }
    }

    fn prev_queue_idx(&self, q: &[usize]) -> Option<usize> {
        if q.is_empty() { return None; }
        let pos = self.current_track_idx.and_then(|ci| q.iter().position(|&x| x == ci));
        match pos {
            Some(p) if p > 0 => Some(q[p - 1]),
            _ => None,
        }
    }

    pub fn ensure_album_art(
        &mut self,
        ctx: &egui::Context,
        track_idx: usize,
    ) -> Option<egui::TextureHandle> {
        if !self.album_art_cache.contains_key(&track_idx) {
            if self.album_art_cache.len() > 32 {
                let keep_idx = track_idx;
                self.album_art_cache.retain(|&k, _| k.abs_diff(keep_idx) <= 4);
            }
            if let Some(track) = self.library.tracks.get(track_idx) {
                if let Some(bytes) = track.album_art.as_ref() {
                    if let Some(tex) = decode_album_art(ctx, bytes) {
                        self.album_art_cache.insert(track_idx, tex);
                    }
                }
            }
        }
        self.album_art_cache.get(&track_idx).cloned()
    }

    pub fn open_folder_dialog(&mut self) {
        if self.folder_promise.is_some() { return; }
        let promise = Promise::spawn_thread("folder-picker", || {
            rfd::FileDialog::new()
                .set_title("Select Music Folder")
                .pick_folder()
        });
        self.folder_promise = Some(promise);
    }

    fn poll_promises(&mut self) {
        let folder_ready = self.folder_promise.as_ref().and_then(|p| p.ready().cloned());
        if let Some(result) = folder_ready {
            self.folder_promise = None;
            if let Some(path) = result {
                self.scan_path(path);
            }
        }

        let scan_ready = self.scan_promise.as_ref().and_then(|p| p.ready().cloned());
        if let Some(lib) = scan_ready {
            self.scan_promise = None;
            self.scanning = false;
            self.album_art_cache.clear();
            let n_tracks = lib.tracks.len();
            self.library = lib;
            self.screen  = Screen::Player;
            self.set_status(format!("Loaded {} tracks", n_tracks));
        }
    }

    pub fn scan_path(&mut self, path: PathBuf) {
        if self.scanning { return; }
        self.add_recent(path.clone());
        self.library_folder  = Some(path.clone());
        self.scanning        = true;
        self.scan_files_done = 0;
        self.set_status(format!("Scanning: {}", path.display()));

        let promise = Promise::spawn_thread("library-scan", move || Library::scan(&path));
        self.scan_promise = Some(promise);
    }

    fn tick(&mut self) {
        if self.engine.state == PlayState::Playing {
            self.viz_phase += 0.04;
        } else {
            self.viz_phase += 0.012;
        }
        if self.viz_phase > 1000.0 {
            self.viz_phase -= 1000.0;
        }

        let (elapsed, total) = self.engine.position();
        if total > 0.0 {
            self.progress = (elapsed / total).clamp(0.0, 1.0);
        }

        if self.engine.is_finished() {
            if self.engine.repeat {
                self.play_current();
            } else {
                self.play_next();
            }
        }

        if let Some((_, when)) = &self.status_message {
            if when.elapsed() > std::time::Duration::from_secs(4) {
                self.status_message = None;
            }
        }
    }
}

impl eframe::App for MusiqApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.poll_promises();
        self.tick();

        // Drag-and-drop folder
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw.dropped_files.iter()
                .filter_map(|f| f.path.clone())
                .filter(|p| p.is_dir())
                .collect()
        });
        for path in dropped {
            self.scan_path(path);
        }

        // Global keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.play_pause();
            } else if i.key_pressed(egui::Key::ArrowLeft) {
                self.play_previous();
            } else if i.key_pressed(egui::Key::ArrowRight) {
                self.play_next();
            } else if i.key_pressed(egui::Key::ArrowUp) {
                self.set_volume((self.volume + 0.05).min(1.0));
            } else if i.key_pressed(egui::Key::ArrowDown) {
                self.set_volume((self.volume - 0.05).max(0.0));
            } else if i.key_pressed(egui::Key::S) {
                self.toggle_shuffle();
            } else if i.key_pressed(egui::Key::R) {
                self.toggle_repeat();
            }
        });

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.track_title()));

        match self.screen {
            Screen::Welcome => crate::ui::welcome(self, ctx),
            Screen::Player  => crate::ui::player(self, ctx),
        }

        // Error dialog
        if let Some(err) = self.error.clone() {
            egui::Window::new("Error")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new(&err).color(TEXT_PRIMARY));
                    ui.add_space(8.0);
                    if ui.button("Dismiss").clicked() {
                        self.dismiss_error();
                    }
                });
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.engine.stop();
        // Final save
        self.persist.save();
    }
}

fn decode_album_art(ctx: &egui::Context, bytes: &[u8]) -> Option<egui::TextureHandle> {
    let img        = image::load_from_memory(bytes).ok()?.to_rgba8();
    let w          = img.width() as usize;
    let h          = img.height() as usize;
    let pixels     = img.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied([w, h], &pixels);
    Some(ctx.load_texture("album_art", color_image, Default::default()))
}
