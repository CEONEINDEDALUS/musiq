//! Persistent state: last folder, recent folders, play history.
//! Saved to ~/.config/musiq/state.toml on every write.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

const APP_DIR: &str = "musiq";
const STATE_FILE: &str = "state.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistState {
    /// Last successfully scanned folder
    pub last_folder: Option<String>,
    /// Up to 8 recent folders (paths as strings)
    pub recent_folders: Vec<String>,
    /// Per-track play count, keyed by canonical path string
    pub play_counts: HashMap<String, u32>,
    /// Per-track last-played unix timestamp (secs since epoch)
    pub last_played: HashMap<String, u64>,
    /// Last played track path
    pub last_track: Option<String>,
    /// Volume level (0.0..=1.0)
    pub volume: Option<f32>,
}

impl PersistState {
    fn config_path() -> Option<PathBuf> {
        let base = dirs_or_home()?;
        Some(base.join(APP_DIR).join(STATE_FILE))
    }

    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Self::default(),
        };
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => return Self::default(),
        };
        toml::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return,
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, text);
        }
    }

    /// Record a play event for the given path.
    pub fn record_play(&mut self, path: &std::path::Path) {
        let key = path.to_string_lossy().to_string();
        *self.play_counts.entry(key.clone()).or_insert(0) += 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_played.insert(key.clone(), now);
        self.last_track = Some(key);
        self.save();
    }

    pub fn play_count_for(&self, path: &std::path::Path) -> u32 {
        *self.play_counts.get(&path.to_string_lossy().to_string()).unwrap_or(&0)
    }

    pub fn push_recent(&mut self, path: &PathBuf) {
        let s = path.to_string_lossy().to_string();
        self.recent_folders.retain(|p| p != &s);
        self.recent_folders.insert(0, s);
        self.recent_folders.truncate(8);
        self.last_folder = Some(path.to_string_lossy().to_string());
        self.save();
    }

    pub fn recent_paths(&self) -> Vec<PathBuf> {
        self.recent_folders
            .iter()
            .map(|s| PathBuf::from(s))
            .filter(|p| p.exists())
            .collect()
    }
}

fn dirs_or_home() -> Option<PathBuf> {
    // Try $XDG_CONFIG_HOME, then $HOME/.config
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg);
        if p.is_absolute() { return Some(p); }
    }
    if let Ok(home) = std::env::var("HOME") {
        return Some(PathBuf::from(home).join(".config"));
    }
    None
}
