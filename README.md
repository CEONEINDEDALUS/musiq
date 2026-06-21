# Musiq

A fast, minimal Linux music player in the style of classic GTK music players — dark monospace UI, three-column layout, animated disc visualizer.

Built entirely in **Rust** using:
- **iced** — pure-Rust GUI (Elm architecture, wgpu rendering, no GTK needed)
- **rodio** — audio playback via ALSA/PulseAudio/PipeWire
- **lofty** — audio tag reading (ID3, Vorbis, MP4, AIFF)
- **walkdir** — fast recursive directory scanning
- **rfd** — native file picker dialog (xdg-portal on Linux)

## Features

- Folder picker intro screen — point it at your music directory
- Fast library scan with metadata reading (title, artist, album, track #, duration)
- Artist → Album → Track navigation
- Animated disc visualizer (canvas-drawn, 60fps)
- Play / Pause / Next / Previous / Seek
- Shuffle and Repeat
- Volume control
- Auto-advance to next track
- "Restart or go back" previous button (< 3s → restart, > 3s → previous)

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
├── ui.rs        — All iced view functions + canvas visualizer
└── theme.rs     — Color constants, size constants
```

### State flow

```
User clicks folder → pick_folder() dialog → ScanComplete(Library)
                                                     ↓
                                          Screen::Player shown
                                                     ↓
User clicks track → SelectTrack(idx) → engine.play(path) → Sink running
                                                     ↓
60fps Tick → update progress bar, advance visualizer phase
           → engine.is_finished()? → play_next()
```

### Performance notes

- Library scan runs in a `tokio::task::spawn_blocking` thread — UI stays responsive
- Audio decoding runs in rodio's internal thread with a buffer — zero UI thread blocking
- Visualizer uses iced's `Canvas` widget — only redraws on `Tick` messages
- `lto = true` + `codegen-units = 1` in release gives ~30% smaller, faster binary
- Release binary is typically 6–10MB stripped

## Extending

**Add waveform/spectrum visualizer:**
Use `rodio_tap` crate to intercept audio samples, run FFT with `rustfft`, feed
frequency bins to the canvas via app state.

**Add album art display:**
`lofty` already extracts embedded APIC/FLAC PICTURE blocks (see `library.rs`).
Load the bytes as an `iced::widget::Image` and display in the center panel.

**Add playlist support:**
Serialize/deserialize `Vec<PathBuf>` with `serde_json` to `~/.config/musiq/playlists/`.

**Add mpris2 media keys:**
Use the `mpris` crate to expose a D-Bus MediaPlayer2 interface — lets you control
playback from GNOME/KDE media key buttons and `playerctl`.

## License

MIT
