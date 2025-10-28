# Getting Started with Industrial Transcoder

**Cross-platform multi-job transcoder for professional media workflows**

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Building from Source](#building-from-source)
- [Platform-Specific Notes](#platform-specific-notes)

---

## Prerequisites

### Required Software

1. **FFmpeg** (Required for transcoding)
   - macOS: `brew install ffmpeg`
   - Linux: `sudo apt-get install ffmpeg`
   - Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)

2. **Rust** (For building)
   - Install from [rustup.rs](https://rustup.rs/)
   - Minimum version: 1.70+

3. **Node.js** (For UI)
   - Install from [nodejs.org](https://nodejs.org/)
   - Recommended: v18+

### Optional Tools

- **BMX Tools** (For MXF workflows)
  - See [MXF_INTEGRATION.md](docs/MXF_INTEGRATION.md)

---

## Installation

### Option 1: Download Pre-built Binary (Coming Soon)

Pre-built binaries will be available for:
- macOS (Intel & Apple Silicon)
- Windows (x64)
- Linux (x64)

### Option 2: Build from Source

See [Building from Source](#building-from-source) below.

---

## Quick Start

### 1. Verify FFmpeg Installation

```bash
# Check if FFmpeg is installed
ffmpeg -version

# If not installed, install it:
# macOS
brew install ffmpeg

# Linux
sudo apt-get install ffmpeg

# Windows: Download from ffmpeg.org
```

### 2. Run the Application

#### Desktop App (Tauri)

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

#### Command Line Interface

```bash
# Build the CLI
cargo build --release -p transcoder-cli

# Transcode a single file
./target/release/transcoder transcode \
  --input input.mxf \
  --output output.mov \
  --preset prores_hq

# List available presets
./target/release/transcoder presets

# Verify FFmpeg installation
./target/release/transcoder verify

# Show system information
./target/release/transcoder info
```

---

## Building from Source

### 1. Clone the Repository

```bash
git clone https://github.com/YOUR_USERNAME/industrial-transcoder-rust-v1.git
cd industrial-transcoder-rust-v1
```

### 2. Install Rust Dependencies

```bash
cargo build --release
```

### 3. Install Node.js Dependencies

```bash
npm install
```

### 4. Build & Run

#### Development Mode

```bash
# Run Tauri app in dev mode
npm run tauri:dev

# Run CLI in dev mode
cargo run -p transcoder-cli -- --help
```

#### Production Build

```bash
# Build Tauri app
npm run tauri:build

# Build CLI
cargo build --release -p transcoder-cli
```

**Output locations:**
- **macOS**: `src-tauri/target/release/bundle/macos/`
- **Windows**: `src-tauri/target/release/bundle/msi/`
- **Linux**: `src-tauri/target/release/bundle/appimage/`
- **CLI**: `target/release/transcoder`

---

## Platform-Specific Notes

### macOS

#### Permissions

On first run, you may need to allow the app in:
- System Preferences → Security & Privacy → General

#### Code Signing (for distribution)

```bash
# Sign the app bundle
codesign --force --deep --sign "Developer ID Application: Your Name" \
  ./src-tauri/target/release/bundle/macos/Industrial\ Transcoder.app
```

#### FFmpeg Installation

```bash
# Install via Homebrew (recommended)
brew install ffmpeg

# Or build from source with custom options
brew install ffmpeg --with-libvpx --with-opus --with-x265
```

### Windows

#### Long Path Support

Enable long paths in Windows 10/11:
1. Run `gpedit.msc`
2. Navigate to: Computer Configuration → Administrative Templates → System → Filesystem
3. Enable "Enable Win32 long paths"

#### FFmpeg Installation

1. Download FFmpeg from [ffmpeg.org](https://ffmpeg.org/download.html)
2. Extract to `C:\ffmpeg`
3. Add `C:\ffmpeg\bin` to PATH:
   - System Properties → Environment Variables → Path → Edit → New

#### Visual C++ Redistributables

Tauri apps require Visual C++ redistributables:
- Download from [Microsoft](https://aka.ms/vs/17/release/vc_redist.x64.exe)

### Linux

#### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  ffmpeg
```

**Fedora:**
```bash
sudo dnf install -y \
  webkit2gtk3-devel \
  openssl-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  ffmpeg
```

**Arch Linux:**
```bash
sudo pacman -S \
  webkit2gtk \
  gtk3 \
  libappindicator-gtk3 \
  librsvg \
  ffmpeg
```

#### AppImage Permissions

```bash
# Make AppImage executable
chmod +x Industrial_Transcoder_2.0.0_amd64.AppImage

# Run
./Industrial_Transcoder_2.0.0_amd64.AppImage
```

---

## Next Steps

- **Read the [Architecture Guide](ARCHITECTURE.md)** to understand the system design
- **Check [EXAMPLES.md](EXAMPLES.md)** for usage examples
- **See [CODEC_PRESETS.md](CODEC_PRESETS.md)** for preset configuration
- **Review [API.md](API.md)** for programmatic usage

---

## Troubleshooting

### FFmpeg Not Found

**Error:** `FFmpeg not found in system PATH`

**Solution:**
1. Install FFmpeg (see prerequisites above)
2. Verify installation: `ffmpeg -version`
3. Restart the application

### Build Errors on Linux

**Error:** `webkit2gtk not found`

**Solution:**
```bash
# Install required system libraries
sudo apt-get install libwebkit2gtk-4.0-dev
```

### Permission Denied on macOS

**Error:** "Industrial Transcoder" cannot be opened because the developer cannot be verified

**Solution:**
1. Open System Preferences → Security & Privacy
2. Click "Open Anyway" for Industrial Transcoder

### Slow Performance

**Tips for optimization:**
- Use SSD for input/output files
- Adjust worker count (default: CPU cores - 1)
- Choose appropriate codec preset (LT vs HQ)
- Close other applications during transcoding

---

## Getting Help

- **Documentation**: See `docs/` directory
- **Issues**: Open an issue on GitHub
- **Discussions**: Start a discussion for questions

---

**Version:** 2.0.0  
**Cross-Platform:** ✅ macOS, Windows, Linux  
**Status:** Production Ready

