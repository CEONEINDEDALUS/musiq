use std::path::{Path, PathBuf};
use std::collections::HashMap;
use lofty::prelude::*;
use lofty::probe::Probe;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub track_number: u32,
    pub duration_secs: u64,
    pub format: String,         // e.g. "MP3", "FLAC"
    pub bitrate: Option<u32>,   // kbps
    pub file_size: u64,         // bytes
    pub album_art: Option<Vec<u8>>,
}

impl Track {
    pub fn duration_display(&self) -> String {
        let m = self.duration_secs / 60;
        let s = self.duration_secs % 60;
        format!("{:02}:{:02}", m, s)
    }

    pub fn file_size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        if self.file_size >= GB {
            format!("{:.2} GB", self.file_size as f64 / GB as f64)
        } else if self.file_size >= MB {
            format!("{:.1} MB", self.file_size as f64 / MB as f64)
        } else if self.file_size >= KB {
            format!("{:.0} KB", self.file_size as f64 / KB as f64)
        } else {
            format!("{} B", self.file_size)
        }
    }

    pub fn bitrate_display(&self) -> String {
        match self.bitrate {
            Some(b) if b > 0 => format!("{} kbps", b),
            _ => "—".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Library {
    pub tracks: Vec<Track>,
    pub artists: Vec<String>,
    pub albums: HashMap<String, Vec<usize>>, // album -> track indices
}

impl Library {
    pub fn scan(folder: &Path) -> Self {
        let mut tracks = Vec::new();

        let audio_exts = ["mp3", "flac", "ogg", "opus", "m4a", "wav", "aac", "aiff", "aif"];

        for entry in WalkDir::new(folder)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !audio_exts.contains(&ext.as_str()) {
                continue;
            }

            if let Some(track) = read_track(path, &ext) {
                tracks.push(track);
            }
        }

        // Sort: artist -> album -> track number
        tracks.sort_by(|a, b| {
            a.artist
                .cmp(&b.artist)
                .then(a.album.cmp(&b.album))
                .then(a.track_number.cmp(&b.track_number))
                .then(a.title.cmp(&b.title))
        });

        // Build artist list
        let mut artist_set: Vec<String> = tracks
            .iter()
            .map(|t| t.artist.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        artist_set.sort();

        // Build album map
        let mut albums: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, track) in tracks.iter().enumerate() {
            albums.entry(track.album.clone()).or_default().push(i);
        }

        Library { tracks, artists: artist_set, albums }
    }

    pub fn tracks_for_artist(&self, artist: &str) -> Vec<usize> {
        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.artist == artist)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn albums_for_artist(&self, artist: &str) -> Vec<String> {
        let mut albums: Vec<String> = self
            .tracks
            .iter()
            .filter(|t| t.artist == artist)
            .map(|t| t.album.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        albums.sort();
        albums
    }

    pub fn tracks_for_album(&self, album: &str) -> Vec<usize> {
        let mut indices = self.albums.get(album).cloned().unwrap_or_default();
        indices.sort_by_key(|&i| self.tracks[i].track_number);
        indices
    }

    pub fn total_duration_secs(&self) -> u64 {
        self.tracks.iter().map(|t| t.duration_secs).sum()
    }
}

fn format_from_ext(ext: &str) -> &'static str {
    match ext {
        "mp3" => "MP3",
        "flac" => "FLAC",
        "ogg" => "OGG",
        "opus" => "OPUS",
        "m4a" => "M4A",
        "wav" => "WAV",
        "aac" => "AAC",
        "aiff" | "aif" => "AIFF",
        _ => "?",
    }
}

fn read_track(path: &Path, ext: &str) -> Option<Track> {
    let file_size = std::fs::metadata(path).ok()?.len();
    let format = format_from_ext(ext).to_string();

    let tagged = Probe::open(path).ok()?.guess_file_type().ok()?.read().ok()?;

    let properties = tagged.properties();
    let duration_secs = properties.duration().as_secs();
    let bitrate = properties.audio_bitrate();

    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());

    let (title, artist, album, track_number, album_art) = if let Some(tag) = tag {
        let title = tag
            .title()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            });

        let artist = tag
            .artist()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let album = tag
            .album()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Album".to_string());

        let track_number = tag.track().unwrap_or(0);

        let album_art = tag.pictures().first().map(|p| p.data().to_vec());

        (title, artist, album, track_number, album_art)
    } else {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        (title, "Unknown Artist".to_string(), "Unknown Album".to_string(), 0, None)
    };

    Some(Track {
        path: path.to_path_buf(),
        title,
        artist,
        album,
        track_number,
        duration_secs,
        format,
        bitrate,
        file_size,
        album_art,
    })
}
