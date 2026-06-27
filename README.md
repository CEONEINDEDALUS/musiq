# Musiq

A fast, minimal Linux music player in the style of classic GTK music players — dark monospace UI, three-column layout.

Built entirely in **Rust** using:
- **eframe/egui** — pure-Rust GUI (immediate mode, wgpu rendering)
- **rodio** — audio playback via ALSA/PulseAudio/PipeWire
- **lofty** — audio tag reading (ID3, Vorbis, MP4, AIFF)
- **walkdir** — fast recursive directory scanning
- **rfd** — native file picker dialog (xdg-portal on Linux)
- **image** — album art decoding (JPEG, PNG)

## Features

- Folder picker intro screen — point it at your music directory
- Fast library scan with metadata reading (title, artist, album, track #, duration)
- Artist / Album / Track navigation with sidebar
- Play / Pause / Next / Previous / Seek
- Shuffle and Repeat
- Volume control
- Auto-advance to next track
- "Restart or go back" previous button (< 3s → restart, > 3s → previous)
- Fuzzy search across title, artist, album
- Album art display
- Persistent state (last folder, volume, play counts) saved to `~/.config/musiq/state.toml`
- Drag-and-drop folder support
- Global keyboard shortcuts (Space, Arrows, S, R)
- Custom titlebar with window dragging

## Supported formats

MP3, FLAC, OGG Vorbis, Opus, M4A/AAC, WAV, AIFF

## Quick start

```bash
# Clone
git clone https://github.com/yourname/musiq
cd musiq

# Build and run (script handles system deps)
chmod +x build.sh
./build.sh

./target/release/musiq
```

## Manual build

### 1. Install system dependencies

**Ubuntu / Debian:**
```bash
sudo apt install libasound2-dev libpulse-dev pkg-config build-essential
```

**Fedora / RHEL:**
```bash
sudo dnf install alsa-lib-devel pulseaudio-libs-devel pkg-config gcc
```

**Arch / Manjaro:**
```bash
sudo pacman -S alsa-lib libpulse pkg-config base-devel
```

> **Note:** PipeWire users don't need extra config — rodio finds PipeWire's PulseAudio
> compatibility layer automatically.

### 2. Install Rust (1.82+)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 3. Build

```bash
# Development (faster compile, slower runtime)
cargo build

# Release (optimized binary, ~6-8MB)
cargo build --release
```

### 4. Run

```bash
./target/release/musiq
```

### 5. Install system-wide (optional)

```bash
sudo install -m755 target/release/musiq /usr/local/bin/musiq
sudo install -m644 musiq.desktop /usr/share/applications/musiq.desktop
```

## Architecture

```
src/
├── main.rs      — Entry point, window settings
├── app.rs       — Application state, Message enum, update logic (MVU)
├── audio.rs     — rodio audio engine wrapper (play/pause/seek/volume)
├── library.rs   — Directory scanner, lofty tag reader, Library struct
├── ui.rs        — All egui view functions
├── theme.rs     — Color constants, style setup
├── search.rs    — Fuzzy scored search across tracks
└── persist.rs   — Save/load state to ~/.config/musiq/state.toml
```

### State flow

```
User clicks folder → pick_folder() dialog → scan_path() → ScanComplete(Library)
                                                            ↓
                                                 Screen::Player shown
                                                            ↓
User clicks track → SelectTrack(idx) → engine.play(path) → Sink running
                                                            ↓
Tick → update progress bar, advance visualizer phase
    → engine.is_finished()? → play_next()
```

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| Space | Play / Pause |
| Left Arrow | Previous track (or restart if > 3s) |
| Right Arrow | Next track |
| Up Arrow | Volume up |
| Down Arrow | Volume down |
| S | Toggle shuffle |
| R | Toggle repeat |

## License

MIT
