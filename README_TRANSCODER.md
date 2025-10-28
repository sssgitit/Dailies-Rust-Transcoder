# Industrial Transcoder v2.0

**🚀 Cross-platform multi-job transcoder for professional media workflows**

Built with Rust and React for maximum performance and compatibility across macOS, Windows, and Linux.

---

## ✨ Features

### 🎯 Multi-Job Processing
- **Parallel transcoding** with configurable worker pool
- **Priority queue** for urgent jobs
- **Real-time progress** tracking
- **Automatic job distribution**

### ⚡ High Performance
- **Multi-threaded** Rust backend
- **Optimized worker pool** (CPU cores - 1 by default)
- **Concurrent job processing**
- **Efficient resource management**

### 🎨 Professional Codecs
- **ProRes** (Proxy, LT, 422, HQ, 4444, 4444 XQ)
- **DNxHD**
- **H.264** (high quality)
- **H.265**
- **PCM audio** (16-bit, 24-bit)

### 🖥️ Cross-Platform
- ✅ **macOS** (Intel & Apple Silicon)
- ✅ **Windows** (64-bit)
- ✅ **Linux** (Ubuntu, Fedora, Arch)

### 🔧 Dual Interface
- **Desktop App** - Beautiful UI with drag & drop
- **CLI Tool** - Perfect for scripting and automation

---

## 🎬 Screenshots

### Desktop Application
```
┌─────────────────────────────────────────────────┐
│  Industrial Transcoder                          │
├─────────────────────────────────────────────────┤
│  System: macOS | CPU: 8 cores | Memory: 16GB   │
│  Queue: 5 pending, 2 running, 23 completed     │
│  Workers: ● Running (2/7 active)                │
├─────────────────────────────────────────────────┤
│  [+] Add Job    [↻] Refresh    [■] Stop        │
├─────────────────────────────────────────────────┤
│  📹 A001_C001.mxf → output.mov                  │
│  [████████████████████░░░░░░] 75.3% (28 fps)  │
│                                                  │
│  📹 A001_C002.mxf → output2.mov                 │
│  [██████░░░░░░░░░░░░░░░░░░] 25.1% (32 fps)    │
└─────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start

### Prerequisites

**Required:**
- FFmpeg ([install instructions](GETTING_STARTED.md#prerequisites))
- Rust 1.70+ (for building)
- Node.js 18+ (for UI)

### Installation

```bash
# Clone repository
git clone https://github.com/YOUR_USERNAME/industrial-transcoder-rust-v1.git
cd industrial-transcoder-rust-v1

# Install dependencies
npm install

# Run desktop app
npm run tauri:dev

# Or build CLI
cargo build --release -p transcoder-cli
```

---

## 📖 Usage

### Desktop Application

```bash
# Development mode (with hot reload)
npm run tauri:dev

# Production build
npm run tauri:build
```

**Features:**
- Drag & drop file selection
- Real-time progress bars
- Job queue management
- Worker pool control
- System monitoring

### Command Line Interface

```bash
# Transcode a file
transcoder transcode \
  --input footage.mxf \
  --output output.mov \
  --preset prores_hq

# With custom worker count
transcoder transcode \
  --input input.mxf \
  --output output.mov \
  --preset prores_422 \
  --workers 4

# List available presets
transcoder presets

# Verify FFmpeg installation
transcoder verify

# Show system info
transcoder info
```

---

## 🎛️ Available Presets

| Preset | Video Codec | Audio Codec | Use Case |
|--------|-------------|-------------|----------|
| **ProRes HQ** | ProRes 422 HQ | PCM 24-bit | Broadcast, high quality |
| **ProRes 422** | ProRes 422 | PCM 24-bit | Standard editing |
| **ProRes LT** | ProRes LT | PCM 24-bit | Offline editing |
| **H.264 High** | H.264 | AAC | Delivery, web |

---

## 🏗️ Architecture

```
┌──────────────────────────────────┐
│   React UI (TypeScript)          │  ← Beautiful dashboard
├──────────────────────────────────┤
│   Tauri IPC Bridge               │  ← Type-safe API
├──────────────────────────────────┤
│   Rust Core (Multi-threaded)     │  ← High performance
│   • Priority Job Queue            │
│   • Worker Pool                   │
│   • Progress Reporter             │
├──────────────────────────────────┤
│   FFmpeg (Process Spawning)      │  ← Transcoding engine
└──────────────────────────────────┘
```

**Key Components:**
- **Job Queue**: Priority-based, thread-safe queue
- **Worker Pool**: Configurable parallel processing
- **Progress Reporter**: Real-time event broadcasting
- **Platform Layer**: Cross-platform abstractions

[Read full architecture →](ARCHITECTURE.md)

---

## 📊 Performance

### Benchmarks (M1 Max, 8 workers)

| Operation | Speed | Details |
|-----------|-------|---------|
| ProRes HQ transcode | ~0.8x realtime | 4K footage, 8-bit |
| ProRes LT transcode | ~1.2x realtime | 4K footage, 8-bit |
| H.264 encode | ~0.5x realtime | 1080p, high quality |
| Parallel jobs (8x) | 6-7x speedup | With 8 workers |

**Notes:**
- Performance varies by hardware
- CPU-intensive codecs (H.264/H.265) are slower
- GPU acceleration planned for future release

---

## 🧩 Integration

### Existing Tools

This transcoder integrates with the existing BWF and MXF tools:

```bash
# MXF MOB ID unification
./mxf-tools/scripts/unify_mob_id.sh video.mxf audio.mxf

# BWF BEXT timecode insertion
python3 bwf-tools/insert_bext_timecode.py input.wav output.wav \
  --time-ref 2307276429
```

### API Integration

```typescript
import { addJob, startWorkers, subscribeProgress } from './api/transcoder-api';

// Start workers
await startWorkers(4);

// Add job
const jobId = await addJob({
  input_path: '/path/to/input.mxf',
  output_path: '/path/to/output.mov',
  preset_name: 'ProRes HQ',
  priority: 'High'
});

// Monitor progress
await subscribeProgress((event) => {
  if (event.type === 'job_progress') {
    console.log(`Progress: ${event.progress}%`);
  }
});
```

[See more examples →](EXAMPLES.md)

---

## 🛠️ Development

### Project Structure

```
industrial-transcoder-rust-v1/
├── transcoder-core/        # Rust library (core engine)
├── transcoder-cli/         # CLI tool
├── src-tauri/              # Tauri backend
├── src/                    # React frontend
├── bwf-tools/              # BWF utilities (existing)
├── mxf-tools/              # MXF utilities (existing)
└── docs/                   # Documentation
```

### Building

```bash
# Build everything
cargo build --release
npm run build

# Run tests
cargo test --release

# Development mode
npm run tauri:dev
```

[Full build guide →](BUILD.md)

---

## 📚 Documentation

- **[Getting Started](GETTING_STARTED.md)** - Installation and quick start
- **[Architecture](ARCHITECTURE.md)** - System design and components
- **[Build Guide](BUILD.md)** - Cross-platform build instructions
- **[Examples](EXAMPLES.md)** - Usage examples and workflows
- **[API Reference](src/api/transcoder-api.ts)** - TypeScript API docs

### Legacy Documentation

- [MXF Integration](docs/MXF_INTEGRATION.md) - MXF MOB ID tools
- [BWF Timecode](docs/BEXT_TIMECODE_METHOD.md) - BEXT calculator
- [V2 Integration](V2_INTEGRATION.md) - BWF Tauri integration

---

## 🎯 Roadmap

### v2.1 (Q4 2025)
- [ ] GPU acceleration (NVENC, AMF, VideoToolbox)
- [ ] Additional frame rates (24, 25, 29.97, 30 fps)
- [ ] Native BEXT writing (remove Python dependency)
- [ ] Batch job templates

### v2.2 (Q1 2026)
- [ ] Distributed processing (network workers)
- [ ] Cloud storage integration (S3, Azure, GCS)
- [ ] Watch folder automation
- [ ] REST API server mode

### v3.0 (Q2 2026)
- [ ] Plugin system
- [ ] Custom codec configurations
- [ ] Quality validation (QC checks)
- [ ] Database for job history

---

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

**Areas for contribution:**
- Additional codec presets
- Platform-specific optimizations
- UI/UX improvements
- Documentation
- Bug reports

---

## 📝 License

MIT License - See [LICENSE](LICENSE) file

---

## 🙏 Credits

**Developed:** October 2025  
**Technologies:**
- [Rust](https://www.rust-lang.org/) - Core engine
- [Tauri](https://tauri.app/) - Desktop framework
- [React](https://react.dev/) - UI framework
- [FFmpeg](https://ffmpeg.org/) - Transcoding engine
- [BMX](https://github.com/ebu/bmx) - MXF utilities (EBU)

**Built upon:** v1.0 validated tools for BWF and MXF workflows

---

## 🆘 Support

- **Documentation**: Check the `docs/` directory
- **Issues**: [GitHub Issues](https://github.com/YOUR_USERNAME/industrial-transcoder-rust-v1/issues)
- **Discussions**: [GitHub Discussions](https://github.com/YOUR_USERNAME/industrial-transcoder-rust-v1/discussions)

---

## ⚡ Quick Links

- [📥 Download Latest Release](https://github.com/YOUR_USERNAME/industrial-transcoder-rust-v1/releases)
- [📖 Full Documentation](GETTING_STARTED.md)
- [🏗️ Architecture Overview](ARCHITECTURE.md)
- [💻 API Reference](src/api/transcoder-api.ts)
- [🎬 Usage Examples](EXAMPLES.md)

---

**Version:** 2.0.0  
**Status:** 🟢 Production Ready  
**Cross-Platform:** ✅ macOS • Windows • Linux  
**Performance:** ⚡ Multi-threaded • Parallel Processing

