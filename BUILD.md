# Build Guide

**Cross-platform build instructions for Industrial Transcoder**

## Quick Build

```bash
# Clone repository
git clone https://github.com/YOUR_USERNAME/industrial-transcoder-rust-v1.git
cd industrial-transcoder-rust-v1

# Install dependencies
npm install

# Build everything
cargo build --release
npm run build

# Run development version
npm run tauri:dev
```

---

## Prerequisites

### All Platforms

- **Rust** 1.70+ ([rustup.rs](https://rustup.rs/))
- **Node.js** 18+ ([nodejs.org](https://nodejs.org/))
- **FFmpeg** (runtime dependency)

### Platform-Specific

#### macOS
```bash
xcode-select --install
```

#### Windows
- **Visual Studio Build Tools 2019+**
- **WebView2** (usually pre-installed on Windows 10/11)

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

---

## Building Components

### 1. Core Library

```bash
cd transcoder-core
cargo build --release
cargo test --release
```

**Output:** `target/release/libtranscoder_core.rlib`

### 2. CLI Tool

```bash
cargo build --release -p transcoder-cli
```

**Output:** `target/release/transcoder` (or `transcoder.exe` on Windows)

**Usage:**
```bash
./target/release/transcoder --help
```

### 3. Tauri Application

```bash
# Install JS dependencies
npm install

# Development build (fast, with hot reload)
npm run tauri:dev

# Production build (optimized)
npm run tauri:build
```

**Output Locations:**

**macOS:**
- DMG: `src-tauri/target/release/bundle/dmg/`
- App: `src-tauri/target/release/bundle/macos/Industrial Transcoder.app`

**Windows:**
- MSI: `src-tauri/target/release/bundle/msi/`
- EXE: `src-tauri/target/release/Industrial Transcoder.exe`

**Linux:**
- AppImage: `src-tauri/target/release/bundle/appimage/`
- DEB: `src-tauri/target/release/bundle/deb/`

---

## Cross-Compilation

### Building for Other Platforms

#### From macOS to Linux

```bash
# Add target
rustup target add x86_64-unknown-linux-gnu

# Install cross-compilation tools
brew install filosottile/musl-cross/musl-cross

# Build
cargo build --release --target x86_64-unknown-linux-gnu
```

#### From Linux to Windows

```bash
# Add target
rustup target add x86_64-pc-windows-gnu

# Install MinGW
sudo apt-get install mingw-w64

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

#### Using Docker (Recommended for Linux builds)

```bash
# Build in Docker container
docker run --rm \
  -v $PWD:/workspace \
  -w /workspace \
  rust:latest \
  cargo build --release
```

---

## Build Profiles

### Development Build

```bash
cargo build
npm run dev
```

**Characteristics:**
- Fast compilation
- Debug symbols included
- Optimizations disabled
- Larger binary size

### Release Build

```bash
cargo build --release
npm run build
```

**Characteristics:**
- Slower compilation
- Optimizations enabled (Level 3)
- LTO (Link-Time Optimization)
- Stripped symbols
- Smaller binary size

### Custom Profile

Edit `Cargo.toml`:
```toml
[profile.production]
inherits = "release"
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

Build with:
```bash
cargo build --profile production
```

---

## Build Options

### Feature Flags

Currently, no optional features. Future features:

```bash
# Example: Enable GPU acceleration
cargo build --release --features gpu-accel

# Example: Disable BWF tools
cargo build --release --no-default-features
```

### Environment Variables

```bash
# Set Rust log level
export RUST_LOG=info

# Set optimization level
export CARGO_PROFILE_RELEASE_OPT_LEVEL=3

# Enable backtrace on panic
export RUST_BACKTRACE=1

# Tauri-specific
export TAURI_SKIP_DEVSERVER_CHECK=true
```

---

## Platform-Specific Build Instructions

### macOS

#### Universal Binary (Intel + Apple Silicon)

```bash
# Add targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary
lipo -create \
  target/x86_64-apple-darwin/release/transcoder \
  target/aarch64-apple-darwin/release/transcoder \
  -output target/release/transcoder-universal
```

#### Code Signing

```bash
# Sign the binary
codesign --force --sign "Developer ID Application: Your Name" \
  src-tauri/target/release/bundle/macos/Industrial\ Transcoder.app

# Verify signature
codesign --verify --deep --strict --verbose=2 \
  src-tauri/target/release/bundle/macos/Industrial\ Transcoder.app
```

#### Notarization

```bash
# Create DMG
hdiutil create -volname "Industrial Transcoder" \
  -srcfolder src-tauri/target/release/bundle/macos/Industrial\ Transcoder.app \
  -ov -format UDZO Industrial-Transcoder.dmg

# Notarize
xcrun notarytool submit Industrial-Transcoder.dmg \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# Staple
xcrun stapler staple Industrial-Transcoder.dmg
```

### Windows

#### Build with Visual Studio

```bash
# Ensure MSVC toolchain is installed
rustup default stable-msvc

# Build
cargo build --release
npm run tauri:build
```

#### Code Signing

```powershell
# Sign with certificate
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com `
  src-tauri\target\release\Industrial Transcoder.exe

# Create MSI installer
npm run tauri:build
```

#### Bundling FFmpeg (Optional)

```bash
# Download FFmpeg
curl -O https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip

# Extract and bundle
unzip ffmpeg-release-essentials.zip
cp ffmpeg-*/bin/ffmpeg.exe src-tauri/target/release/
```

### Linux

#### Ubuntu/Debian

```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-dev \
  build-essential curl wget libssl-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev

# Build
cargo build --release
npm run tauri:build
```

#### Fedora

```bash
# Install dependencies
sudo dnf install webkit2gtk3-devel openssl-devel \
  gtk3-devel libappindicator-gtk3-devel librsvg2-devel

# Build
cargo build --release
npm run tauri:build
```

#### Creating DEB Package

```bash
# Tauri automatically creates DEB
npm run tauri:build

# Or manually with cargo-deb
cargo install cargo-deb
cargo deb -p transcoder-cli
```

#### Creating RPM Package

```bash
# Install cargo-rpm
cargo install cargo-rpm

# Build RPM
cargo rpm build -p transcoder-cli
```

---

## Optimizing Build Times

### Use `sccache`

```bash
# Install sccache
cargo install sccache

# Configure
export RUSTC_WRAPPER=sccache

# Build (cached)
cargo build --release
```

### Incremental Compilation

```bash
# Enable in Cargo.toml
[profile.dev]
incremental = true
```

### Parallel Compilation

```bash
# Use all CPU cores
export CARGO_BUILD_JOBS=$(nproc)

# Or set in .cargo/config.toml
[build]
jobs = 8
```

### Workspace Optimization

```toml
# In workspace Cargo.toml
[profile.release]
codegen-units = 1  # Better optimization, slower compile
```

---

## Continuous Integration

### GitHub Actions

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install dependencies (Linux)
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.0-dev
      
      - name: Build
        run: |
          cargo build --release
          npm install
          npm run tauri:build
      
      - name: Test
        run: cargo test --release
```

---

## Troubleshooting

### Common Build Errors

#### `linker 'cc' not found`

**Solution:**
```bash
# macOS
xcode-select --install

# Linux
sudo apt-get install build-essential

# Windows
# Install Visual Studio Build Tools
```

#### `webkit2gtk not found`

**Solution (Linux):**
```bash
sudo apt-get install libwebkit2gtk-4.0-dev
```

#### `failed to run custom build command for 'openssl-sys'`

**Solution:**
```bash
# Install OpenSSL development files
# Ubuntu/Debian
sudo apt-get install libssl-dev

# Fedora
sudo dnf install openssl-devel

# macOS
brew install openssl
```

#### Out of Memory

**Solution:**
```bash
# Reduce parallel jobs
export CARGO_BUILD_JOBS=2

# Or add swap space (Linux)
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

---

## Clean Build

```bash
# Clean Rust build artifacts
cargo clean

# Clean Node modules
rm -rf node_modules
npm install

# Clean everything
cargo clean
rm -rf node_modules dist target
npm install
```

---

## Build Size Analysis

```bash
# Analyze binary size
cargo install cargo-bloat
cargo bloat --release

# Check dependencies
cargo tree

# Optimize for size
[profile.release]
opt-level = 'z'  # Optimize for size
lto = true
strip = true
```

---

## Next Steps

- **Deploy:** See deployment documentation
- **Test:** Run integration tests
- **Package:** Create installers for distribution

---

**Questions?** Open an issue on GitHub.

**Version:** 2.0.0  
**Cross-Platform:** ✅ macOS, Windows, Linux

