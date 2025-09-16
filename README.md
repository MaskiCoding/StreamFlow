# StreamFlow-Tauri

A powerful Twitch stream manager that combines the functionality and user interface of StreamFlow with the resource efficiency and performance of Dorion's Tauri architecture.

## Features

- **Twitch Stream Management**: Watch any Twitch stream with customizable quality settings
- **Quick Stream Slots**: Save up to 4 favorite streams for one-click access  
- **Stream Status Tracking**: Real-time online/offline status for saved streams
- **Ad-Free Streaming**: Built-in ad-blocking using luminous.dev proxy
- **Rose Pine Theme**: Beautiful, consistent UI theme throughout the application
- **Resource Efficient**: Built with Tauri for minimal memory and CPU usage
- **Cross-Platform**: Works on Windows, macOS, and Linux

## Requirements

- **Streamlink**: Required for stream processing
  - Install with: `pip install streamlink`
- **VLC Media Player**: Recommended as the default video player
- **Node.js**: For development (if building from source)
- **Rust**: For development (if building from source)

## Installation

### Pre-built Releases
Download the latest release from the [Releases](https://github.com/MaskiCoding/StreamFlow-Tauri/releases) page.

### Building from Source
1. Clone this repository
2. Install dependencies:
   ```bash
   npm install
   ```
3. Install Rust dependencies:
   ```bash
   cd src-tauri
   cargo fetch
   ```
4. Run in development mode:
   ```bash
   npm run dev
   ```
5. Build for production:
   ```bash
   npm run build
   ```

## Usage

1. **Basic Streaming**:
   - Enter a Twitch URL or username in the main input field
   - Select your preferred quality
   - Click "Watch Stream" to start

2. **Quick Streams**:
   - Click the "+" button on any empty slot to add a favorite stream
   - Click on saved streams to instantly start watching
   - Right-click to remove saved streams

3. **Quality Settings**:
   - Choose from: best, 1080p, 720p, 480p, 360p, 160p, worst
   - Your preference is automatically saved

## Architecture

This application combines two excellent projects:

- **StreamFlow**: Provides the core Twitch streaming functionality and beautiful Rose Pine UI
- **Dorion**: Contributes the efficient Tauri-based architecture for optimal performance

The result is a lightweight, fast, and feature-rich stream manager that uses minimal system resources while providing a premium user experience.

## Supported Platforms

- Windows 10/11
- macOS 10.15+
- Linux (Ubuntu 18.04+, Fedora 32+, Debian 10+)

## Configuration

Settings are automatically saved to:
- **Windows**: `%APPDATA%\StreamFlow-Tauri\settings.json`
- **macOS**: `~/Library/Application Support/StreamFlow-Tauri/settings.json`  
- **Linux**: `~/.local/share/StreamFlow-Tauri/settings.json`

## Troubleshooting

### Stream Won't Start
- Ensure Streamlink is installed: `pip install streamlink`
- Check that the Twitch URL/username is valid
- Try a different quality setting

### No Video Player Opens
- Install VLC Media Player and set it as the default video player
- Alternatively, configure Streamlink to use your preferred media player

### Performance Issues
- The app is designed to be lightweight, but ensure you're not running too many other applications
- Try closing other browser tabs or applications if needed

## Contributing

This project builds upon the excellent work of:
- [StreamFlow](https://github.com/MaskiCoding/StreamFlow) - Original stream manager
- [Dorion](https://github.com/SpikeHD/Dorion) - Tauri architecture and performance optimizations

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- **SpikeHD** for creating Dorion and demonstrating excellent Tauri architecture
- **The Streamlink team** for the powerful streaming backend
- **The Tauri team** for the amazing framework
- **Rose Pine** color scheme for the beautiful theme
