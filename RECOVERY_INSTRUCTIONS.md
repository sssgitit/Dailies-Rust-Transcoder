# Recovery Instructions - Industrial Transcoder GUI

**Last Stable Commit:** `b4b9a91` (Oct 28, 2025)  
**Repository:** https://github.com/sssgitit/Dailies-Rust-Transcoder

## If You Need to Restore This Working Version

### Quick Start
```bash
# Clone or pull the latest
cd /Users/Editor/Downloads
git clone https://github.com/sssgitit/Dailies-Rust-Transcoder.git
cd Dailies-Rust-Transcoder

# Install dependencies
npm install

# Run the app
npm run tauri:dev
```

### Current Working Features

#### Tab 1: 🎬 Quick Transcode & BWF
- **Multi-file selection** (Cmd+Click to select multiple files)
- **DNxHR LB transcoding** with hardware acceleration (8x realtime)
- **BWF audio extraction** with frame-accurate BEXT timecode
- **Batch processing** - processes files sequentially
- Files: `src/components/SimpleTranscoder.tsx`

#### Tab 2: 📊 Job Queue Dashboard
- **Job queue system** with priority management
- **Worker pool** control (start/stop workers)
- **Clear Completed button** with automatic detailed logging
- **Performance metrics** tracking
- Logs saved to: `~/.industrial-transcoder/logs/`
- Files: `src/components/TranscoderDashboard.tsx`, `transcoder-core/src/logger.rs`

#### Tab 3: 🎞️ MXF Tools
- MXF rewrap utilities
- Files: `src/components/MxfRewrapTool.tsx`

### Key Files & Structure

```
industrial-transcoder-rust-v1_simple_attempt/
├── src/
│   ├── App.tsx                          # Main app with 3-tab navigation
│   ├── components/
│   │   ├── SimpleTranscoder.tsx         # Tab 1 - Multi-file batch transcode
│   │   ├── TranscoderDashboard.tsx      # Tab 2 - Job queue & Clear Completed
│   │   ├── JobList.tsx                  # Job list display
│   │   └── AddJobDialog.tsx             # Add job form
│   └── api/
│       └── transcoder-api.ts            # TypeScript API for Tauri
│
├── src-tauri/
│   └── src/
│       ├── main.rs                      # Tauri main app
│       ├── transcoder_commands.rs       # All Tauri commands
│       └── mxf_commands.rs              # MXF utilities
│
├── transcoder-core/
│   └── src/
│       ├── logger.rs                    # Job logging system ⭐ NEW
│       ├── job.rs                       # Job definitions
│       ├── queue.rs                     # Job queue with clear_completed
│       ├── worker.rs                    # Worker pool
│       └── transcode.rs                 # FFmpeg wrapper
│
└── package.json                          # NPM dependencies
```

### System Requirements

- **macOS** with Apple Silicon (M1/M2/M3) or Intel
- **FFmpeg 8.0** with VideoToolbox support
- **Rust** (latest stable)
- **Node.js** 18+

### Install FFmpeg (if needed)
```bash
brew install ffmpeg
```

### Verify Setup
```bash
which ffmpeg
ffmpeg -version | grep -i videotoolbox
```

### Build from Scratch
```bash
# Frontend
npm install
npm run build

# Backend
cargo build --release

# Run development mode
npm run tauri:dev

# Build production app
npm run tauri:build
```

## What This Version Has

### ✅ Completed Features
1. **Multi-file batch processing** - Select and process multiple files at once
2. **Tabbed interface** - Three organized tabs for different workflows
3. **Job logging system** - Detailed logs with:
   - Codec specifications
   - System information (CPU, RAM, FFmpeg version)
   - Performance metrics (read/write speeds)
   - File sizes and durations
   - Timestamps
4. **Clear Completed button** - Logs jobs before clearing
5. **Hardware acceleration** - VideoToolbox for 8x realtime encoding
6. **BWF audio** - Frame-accurate BEXT timecode from MXF
7. **Error handling** - Detailed FFmpeg error messages
8. **Progress tracking** - Per-file and overall progress

### 📝 Log Location
```
~/.industrial-transcoder/logs/
├── 2025-10-28_transcoder_log.txt     # Human-readable
└── 2025-10-28_transcoder_log.json    # Machine-readable
```

## Troubleshooting

### App won't start
```bash
# Kill any running instances
pkill -f "industrial-transcoder"

# Check port
lsof -ti:1420 | xargs kill -9

# Restart
npm run tauri:dev
```

### FFmpeg errors
Check the terminal output - errors are now displayed in detail.

### Files not processing
- Verify input files exist
- Check output directory has write permissions
- Look at terminal logs for FFmpeg output

## Contact
- GitHub: https://github.com/sssgitit/Dailies-Rust-Transcoder
- Commit: `b4b9a91`
- Date: October 28, 2025

---

**STABLE VERSION - TESTED AND WORKING** ✅

