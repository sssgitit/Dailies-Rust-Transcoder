# Examples - Industrial Transcoder

## A012 Car Workflow Examples

These examples demonstrate processing the A012 car clips through a complete production pipeline.

### Files

1. **`A012_car_workflow.sh`** - Shell script for single file processing
2. **`A012_car_batch.ts`** - TypeScript/Tauri batch processing

---

## Shell Script Example

### Single File Processing

```bash
# Make executable
chmod +x A012_car_workflow.sh

# Run the workflow
./A012_car_workflow.sh
```

**What it does:**
1. ✅ Detects MXF wrapping type
2. 🔄 Converts clip-wrapped → frame-wrapped (if needed)
3. 📹 Transcodes to ProRes HQ for editing
4. 🎬 Creates ProRes LT proxy for offline editing

**Output structure:**
```
processed/
├── 01_frame_wrapped/
│   └── A012_car_frame.mxf
├── 02_prores/
│   └── A012_car_ProResHQ.mov
└── 03_proxy/
    └── A012_car_ProxyLT.mov
```

### Batch Processing

For multiple files:

```bash
#!/bin/bash
# Process all A012 clips

for file in footage/A012_car_*.mxf; do
  INPUT_FILE="$file" ./A012_car_workflow.sh
done
```

---

## TypeScript/Tauri Example

### Setup

```bash
# Install dependencies (if not already done)
npm install

# Run in development mode
npm run tauri:dev
```

### Usage

```typescript
import { processA012CarClips } from './examples/A012_car_batch';

// Process all A012 car clips
await processA012CarClips();
```

**What it does:**
1. 🔄 Batch rewraps all MXF files to frame-wrapped
2. 📹 Adds transcode jobs (ProRes HQ + ProRes LT)
3. ⚙️ Starts 4 workers for parallel processing
4. 📊 Shows real-time progress
5. ✅ Displays final summary

**Features:**
- Parallel processing (4 workers)
- Priority queue (ProRes HQ = High, Proxy = Normal)
- Real-time progress tracking
- Error handling and retry logic
- Final summary with statistics

---

## Desktop App Workflow

### For A012 Car Clips

1. **Open Industrial Transcoder**

2. **MXF Rewrap Tab:**
   - Click "Browse" → Select `A012_car.mxf`
   - Auto-detects: "Clip-Wrapped"
   - Target: "Frame-Wrapped" (🎬)
   - Click "🔄 Rewrap MXF File"

3. **Transcode Tab:**
   - Click "+ Add Job"
   - Input: `A012_car_frame.mxf`
   - Preset: "ProRes HQ"
   - Priority: "High"
   - Click "Add Job"

4. **Repeat for Proxy:**
   - Add another job
   - Same input file
   - Preset: "ProRes LT"
   - Priority: "Normal"

5. **Start Workers:**
   - Click "Start Workers"
   - Watch real-time progress!

---

## Production Workflow

### Recommended Pipeline for A012 Clips

```
📹 Camera Cards (A012_car_*.mxf)
    ↓
🔄 Rewrap to Frame-Wrapped
    ↓
📹 Transcode to ProRes HQ (editing master)
    ↓
🎬 Create ProRes LT Proxy (offline)
    ↓
✅ Import to NLE (Avid/Premiere/FCP)
```

### File Naming Convention

```
A012_car_01.mxf         → Original (clip-wrapped)
A012_car_01_frame.mxf   → Frame-wrapped
A012_car_01_ProResHQ.mov → Editing master
A012_car_01_ProxyLT.mov  → Offline proxy
```

---

## Performance Estimates

### Single A012 Clip (10GB, 10 minutes @ 4K)

| Step | Time | Output Size |
|------|------|-------------|
| Rewrap (clip→frame) | ~2-3 min | 10.2 GB |
| ProRes HQ transcode | ~15-20 min | 25 GB |
| ProRes LT proxy | ~8-10 min | 8 GB |
| **Total** | **~25-33 min** | **43.2 GB** |

### Batch (5 clips, 4 workers)

- **Sequential:** ~2-3 hours
- **Parallel (4 workers):** ~40-60 minutes
- **Speedup:** ~3-4x faster

---

## Troubleshooting

### "bmxtranswrap not found"

```bash
# macOS
brew install bmx

# Linux
sudo apt-get install bmx

# Or build from source
git clone https://github.com/ebu/bmx.git
cd bmx && mkdir build && cd build
cmake .. && make && sudo make install
```

### "FFmpeg not found"

```bash
# macOS
brew install ffmpeg

# Linux
sudo apt-get install ffmpeg

# Windows
# Download from ffmpeg.org
```

### Out of Disk Space

Each transcode can be 2-3x the original file size. Ensure:
- **50-100 GB free** for single clip
- **200-500 GB free** for batch processing

### Slow Performance

Tips:
- Use **SSD** (not HDD)
- Close other apps
- Reduce worker count if system is overloaded
- Check available RAM (min 8GB recommended)

---

## Integration with Existing Tools

### After MXF Rewrap

```bash
# Step 1: Rewrap
./A012_car_workflow.sh

# Step 2: Unify MOB IDs (if needed)
./mxf-tools/scripts/unify_mob_id.sh \
  processed/01_frame_wrapped/A012_car_frame.mxf

# Step 3: Add BWF timecode (for audio)
python3 bwf-tools/insert_bext_timecode.py \
  audio.wav output.wav \
  --time-ref 2307276429
```

---

## Next Steps

1. **Customize presets** - Edit `transcoder-core/src/config.rs`
2. **Create templates** - Save common workflows
3. **Automate** - Use watch folders or cron jobs
4. **Monitor** - Check logs for errors

---

## Questions?

- Check [GETTING_STARTED.md](../GETTING_STARTED.md)
- Read [MXF_REWRAP_GUIDE.md](../MXF_REWRAP_GUIDE.md)
- Open an issue on GitHub

---

**Happy Transcoding! 🎬**

