# ⚡ FetchDesk

> High-Performance Terminal-Based Multi-Connection Download Manager written in Rust.

`FetchDesk` is a powerful, lightning-fast CLI download manager designed for power users, ethical security researchers, software engineers, and media enthusiasts. It accelerates HTTP/HTTPS downloads using parallel multi-chunk streaming, handles YouTube videos with quality selection, and manages BitTorrent/Magnet transfers with automated peer discovery.

---

## ✨ Features

- **🚀 16-Connection Parallel Acceleration**: Splits direct HTTP/HTTPS files into range-requested chunks and streams them concurrently into pre-allocated files for maximum bandwidth utilization.
- **🎥 YouTube Video & Playlist Downloader**: Native `yt-dlp` integration supporting 4K, 2K, 1080p, 720p, 480p, and Audio-only formats with native 16-way fragment acceleration.
- **🧲 BitTorrent & Magnet Link Engine**: Multi-seeder BitTorrent transfers with automatic tracker injection (`opentrackr`, `stealth.si`, `torrent.eu.org`), DHT, and Transmission peer-ID spoofing for priority unchokes.
- **💾 Smart Disk Space Inspector & Drive Switcher**: Automatically checks target drive free space (`GetDiskFreeSpaceExW`) before downloading. If the current drive (e.g. `C:\`) is low on space, FetchDesk displays an interactive drive selection menu to seamlessly switch to drives with available space (e.g. `D:\Downloads`).
- **📊 Sleek Animated Single-Line UI**: Replaces noisy terminal output with clean, animated `indicatif` progress bars displaying speed, ETA, percent, seeder counts, and peer metrics.
- **📜 Download History & CSV Export**: Tracks all downloads with timing, size, and status. Export history to escaped CSV format at any time.

---

## 🛠️ Prerequisites & Installation

### Core Build Requirement
Ensure you have **Rust** installed (`cargo` & `rustc` 1.70+):
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Optional External Dependencies

To unlock full functionality (YouTube video extraction and BitTorrent downloads), install the following optional tools:

| Component | Purpose | Installation (Windows) | Installation (macOS) | Installation (Linux) |
| :--- | :--- | :--- | :--- | :--- |
| **`yt-dlp`** | YouTube Video & Playlist Extraction | `pip install -U yt-dlp` | `brew install yt-dlp` | `sudo apt install yt-dlp` |
| **`aria2`** | BitTorrent / Magnet Engine | `winget install aria2.aria2` | `brew install aria2` | `sudo apt install aria2` |
| **`ffmpeg`** | Video & Audio Format Merging | `winget install Gyan.FFmpeg` | `brew install ffmpeg` | `sudo apt install ffmpeg` |

*Note: If `ffmpeg` is not installed, FetchDesk automatically selects pre-merged single-file formats (e.g. 720p MP4) so downloads complete cleanly without post-processing failures.*

---

## 🚀 Building & Running

### 1. Build from Source
```powershell
# Clone repository
git clone https://github.com/JOJINJOHN/fetchdesk.git
cd fetchdesk-src

# Build release binary
cargo build --release
```

Binary will be produced at `target/release/fetchdesk` (or `fetchdesk.exe` on Windows).

### 2. Launch Interactive REPL
```powershell
cargo run
```
or run the binary directly:
```powershell
.\target\release\fetchdesk.exe
```

---

## 💻 CLI Commands & Usage

At the `fetchdesk>` interactive prompt:

| Command | Action | Example |
| :--- | :--- | :--- |
| **`y`** | YouTube Video / Playlist | Paste YouTube URL (selects quality 1–13) |
| **`m`** | Magnet Link / `.torrent` Path | Paste `magnet:?xt=urn:...` or local `.torrent` path |
| **`d`** | Direct HTTP/HTTPS File | Paste direct file link (`https://example.com/file.zip`) |
| **`conns <n>`** | Set Parallel Connections | `conns 16` |
| **`out <path>`** | Set Output Directory | `out D:\Downloads` |
| **`history`** | Show Download Log | `history` |
| **`export`** | Export History to CSV | `export` |
| **`help` / `?`** | Display Command List | `help` |
| **`exit` / `q`** | Quit FetchDesk | `exit` |

---

## 📁 Repository Structure

```text
fetchdesk-src/
├── Cargo.toml          # Rust dependencies & package configuration
├── src/
│   ├── main.rs         # REPL prompt & CLI routing
│   ├── downloader.rs   # Direct multi-chunk HTTP downloader engine
│   ├── youtube.rs      # yt-dlp integration & quality selector
│   ├── torrent.rs      # aria2 BitTorrent & Magnet engine
│   ├── disk.rs         # Win32 disk space inspector & drive switcher
│   ├── config.rs       # System configuration & persistence
│   ├── history.rs      # Download history logger & CSV exporter
│   ├── queue.rs        # Concurrent download queue system
│   └── banner.rs       # ASCII banner & terminal UI utilities
└── docs/
    └── index.html      # Animated web dashboard & documentation
```

---

## ⚖️ License & Ethical Usage

Distributed under the **MIT License**. Created by **JOJIN JOHN**.

*Disclaimer: FetchDesk is designed for legitimate software distribution, open-source media download, and ethical security testing. Always verify you have authorization before downloading files.*
