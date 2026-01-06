# StreamFlow-Tauri

[![CI/CD](https://github.com/MaskiCoding/StreamFlow/actions/workflows/ci.yml/badge.svg)](https://github.com/MaskiCoding/StreamFlow/actions/workflows/ci.yml)

A modern Twitch stream manager built with Tauri 2.x and Rust. Features include stream quality selection, favorite stream management, real-time status tracking, and a clean, responsive interface.

## Features

- **Stream Management**: Watch Twitch streams with customizable quality settings
- **Quick Access**: Save up to 4 favorite streams for instant access
- **Status Tracking**: Real-time online/offline stream status updates
- **Ad-Free Viewing**: Built-in ad blocking via luminous.dev proxy
- **Modern UI**: Clean, responsive interface with custom window controls
- **Resource Efficient**: Minimal memory and CPU usage thanks to Rust backend
- **Standalone App**: No installation required - just run the executable
- **Cross-Platform Ready**: Built on Tauri for future multi-platform support

## Requirements

### Runtime
- **Streamlink**: `pip install streamlink`
- **VLC Media Player**: Required video player for stream playback

### Development
- **Node.js 20+**
- **Rust 1.84+**
- **Visual Studio Build Tools** (Windows) with C++ workload
- **Git**

## Installation

### Pre-built Releases (Recommended)
Download the latest release from [GitHub Releases](https://github.com/MaskiCoding/StreamFlow/releases).

**Rolling Release**: Always up-to-date builds from the main branch
**Versioned Release**: Stable releases with version tags (v1.0.0, v1.1.0, etc.)

**Available for:**
- Windows x64 (MSVC)

### Building from Source

```bash
# Clone repository
git clone https://github.com/MaskiCoding/StreamFlow-Tauri.git
cd StreamFlow-Tauri

# Install dependencies
npm install

# Development mode
npm run dev

# Production build
npm run build
```

## Usage

1. **Enter a Twitch URL or username**
2. **Select preferred quality** (best, 1080p, 720p, etc.)
3. **Click "Watch Stream"** to start
4. **Use Quick Streams** to save and access favorite channels

## Configuration

Settings are automatically saved to `%APPDATA%\StreamFlow-Tauri` on Windows. Quality preferences and saved streams persist between sessions.

## CI/CD Pipeline

The project uses GitHub Actions for automated builds and releases:

| Job | Description |
|-----|-------------|
| **Auto-fix** | Automatically formats code with Prettier (frontend) and rustfmt (Rust) |
| **Code Quality** | Runs rustfmt check and clippy linting |
| **Build** | Compiles debug and release builds, runs tests |
| **Rolling Release** | Creates artifacts on every push to main |
| **Versioned Release** | Creates GitHub releases when pushing `v*.*.*` tags |

### Creating a Release
```bash
# Tag a new version
git tag v1.2.0
git push origin v1.2.0
```

## Tech Stack

- **Frontend**: Vanilla HTML/CSS/JavaScript
- **Backend**: Rust with Tauri 2.x
- **Plugins**: tauri-plugin-shell, tauri-plugin-process, tauri-plugin-http
- **Streaming**: Streamlink with VLC player
- **Ad Blocking**: luminous.dev TTV LOL proxy

## Troubleshooting

### Stream Won't Start
- Ensure Streamlink is installed: `pip install streamlink`
- Check internet connection
- Try different quality settings (best, 1080p60, 720p60, etc.)
- Verify VLC is installed and accessible in PATH

### Build Issues
- Ensure Node.js 20+ and Rust 1.84+ are installed
- Install Visual Studio Build Tools with C++ workload
- Check GitHub Actions logs for detailed error information

### VLC Not Found
- Install VLC from [videolan.org](https://www.videolan.org/)
- Ensure VLC is in your system PATH

## Changelog

### v1.1.0 - Tauri 2.x Migration & Code Quality
- **Migrated**: Full migration to Tauri 2.x architecture
- **Added**: CI/CD pipeline with auto-formatting and rolling releases
- **Added**: Proper error handling with custom `StreamFlowError` type
- **Added**: Shared types module to eliminate duplication
- **Fixed**: All mutex lock error handling (no more panics)
- **Fixed**: Window management using `WebviewWindow` for Tauri 2.x
- **Fixed**: Plugin initialization for shell, process, and http
- **Improved**: Code organization with modular Rust structure

### v1.0.1 - Bug Fixes and Performance Improvements
- **Fixed**: JavaScript API call bug in status indicator update
- **Fixed**: Removed dead code functions for cleaner codebase
- **Improved**: Optimized autocomplete prevention (reduced resource usage)
- **Improved**: Fixed HTTP user agent to avoid non-existent domains
- **Enhanced**: Better error handling and logging throughout application

### v1.0.0 - Production Release
- Complete modular architecture with ES6 modules
- Comprehensive autocomplete prevention
- Standalone executable distribution
- Professional CI/CD with automatic releases
- Production-ready error handling and performance optimizations

## License

MIT License