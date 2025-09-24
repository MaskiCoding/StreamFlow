# StreamFlow-Tauri

A modern Twitch stream manager built with Tauri. Features include stream quality selection, favorite stream management, real-time status tracking, and a clean, responsive interface.

## Features

- **Stream Management**: Watch Twitch streams with customizable quality settings
- **Quick Access**: Save up to 4 favorite streams for instant access
- **Status Tracking**: Real-time online/offline stream status updates
- **Ad-Free Viewing**: Built-in ad blocking
- **Modern UI**: Clean, responsive interface
- **Resource Efficient**: Minimal memory and CPU usage
- **Standalone App**: No installation required - just run the executable
- **Autocomplete Prevention**: No browser suggestions in input fields

## Requirements

### Runtime
- **Streamlink**: `pip install streamlink`
- **VLC Media Player**: Recommended video player

### Development
- **Node.js 18+**
- **Rust 1.70+**
- **Git**

## Installation

### Pre-built Releases (Recommended)
Download the latest release from [GitHub Releases](https://github.com/MaskiCoding/StreamFlow-Tauri/releases).

**Available for:**
- Windows (x64)

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

Settings are automatically saved locally. Quality preferences and saved streams persist between sessions.

## CI/CD

The project uses GitHub Actions for automated builds:
- **Trigger**: Version tags automatically create releases
- **Platform**: Windows (x64)
- **Output**: Standalone executable files

## Troubleshooting

### Stream Won't Start
- Ensure Streamlink is installed: `pip install streamlink`
- Check internet connection
- Try different quality settings
- Verify VLC is installed and set as default player

### Build Issues
- Ensure Node.js 18+ and Rust 1.70+ are installed
- Check GitHub Actions logs for detailed error information

## Changelog

### v1.0.1 - Bug Fixes and Performance Improvements
- **Fixed**: JavaScript API call bug in status indicator update
- **Fixed**: Removed dead code functions for cleaner codebase
- **Improved**: Optimized autocomplete prevention (reduced resource usage)
- **Improved**: Fixed HTTP user agent to avoid non-existent domains
- **Enhanced**: Better error handling and logging throughout application
- **Verified**: All API functions properly linked and functional

### v1.0.0 - Production Release
- Complete modular architecture with ES6 modules
- Comprehensive autocomplete prevention
- Standalone executable distribution
- Professional CI/CD with automatic releases
- Production-ready error handling and performance optimizations

## License

MIT License