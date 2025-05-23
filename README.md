# 🚀 HyprDownloader

<div align="center">

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![GTK4](https://img.shields.io/badge/GTK4-4A86CF?style=flat&logo=gtk&logoColor=white)
![Downloads](https://img.shields.io/badge/Downloads-Unlimited-green)
[![GitHub](https://img.shields.io/badge/GitHub-View_Repository-181717?style=flat&logo=github&logoColor=white)](https://github.com/os-guy/HyprDownloader)

</div>

A modern, sleek GTK4 application for downloading high-quality media from various websites. Built with Rust for performance and reliability.

![HyprDownloader Screenshot](https://via.placeholder.com/800x450?text=HyprDownloader+Screenshot)

## ✨ Features

- 🎮 **Intuitive UI**: Clean and modern GTK4 interface
- 🎬 **Video Downloads**: Select your preferred resolution and FPS
- 🎵 **Audio Downloads**: Extract audio in various formats and bitrates
- 🎯 **Smart Format Detection**: Detailed suggestions for optimal format selection
- 🔄 **Real-time Progress**: Live download progress tracking 
- 📂 **Organized Storage**: Automatic categorization into video and audio folders
- 🌐 **Wide Compatibility**: 
  - ✅ **Tested**: YouTube
  - ⚠️ **Untested**: Vimeo, Dailymotion, and others supported by yt-dlp

## 🛠️ Installation

### Prerequisites

- Rust and Cargo
- GTK4 development libraries
- yt-dlp

### Installing Prerequisites

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install libgtk-4-dev build-essential
pip install yt-dlp
```

#### Arch Linux
```bash
# Install dependencies directly with pacman
sudo pacman -S gtk4 base-devel python-pip
pip install yt-dlp

# Or install from AUR (future availability)
# yay -S hyprdownloader
```

#### Fedora
```bash
sudo dnf install gtk4-devel gcc
pip install yt-dlp
```

#### macOS
```bash
brew install gtk4
pip install yt-dlp
```

## 🚀 Building and Running

To build and run the application:

```bash
# Clone the repository
git clone https://github.com/os-guy/HyprDownloader.git
cd HyprDownloader

# Build and run
cargo run

# For release version
cargo run --release
```

## 📖 Usage

1. Enter the URL of the media you want to download
2. Click **Fetch** to retrieve the available formats
3. Switch between **Video** and **Audio** tabs based on what you want to download
4. Select your preferred quality options:
   - For videos: Choose resolution, FPS, and format
   - For audio: Choose bitrate and format
5. The download path is automatically set to organize your downloads
6. Click **Download Media** to start downloading

## 💡 Tips

- Higher resolution doesn't always mean better quality; consider file size too
- MP4 format offers good compatibility across devices
- For audio, MP3 is widely compatible while OPUS offers better quality at smaller sizes

## 📜 License

This project is licensed under the GNU General Public License v3.0 - see the LICENSE file for details. 