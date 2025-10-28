# 🖥️ GUI Quick Start Guide

**Status:** Building now (2-3 minutes first time)  
**Will auto-open when ready!**

---

## 🎯 Quick Start (3 Steps)

### 1. Select Preset
- **Default:** "DNxHR LB (Fast)" ⚡ (Hardware accelerated, 6.73x realtime)
- Other options: ProRes HQ, DNxHR HQX, etc.

### 2. Add Files
**Method A: Drag & Drop**
- Drag MXF files into the window

**Method B: Browse**
- Click "Add Job" button
- Navigate to your files
- Select one or multiple MXF files

### 3. Start Processing
- Click "Start Workers" button
- Watch real-time progress!
- 11 jobs will run simultaneously

---

## 📊 What You'll See

### Job Queue
- List of all files to transcode
- Status: Pending → In Progress → Complete
- Real-time progress bars (%)
- Speed indicator (e.g., "6.73x")

### Worker Status
- Active workers: X/11
- Shows which files are transcoding now

### Output Location
- Default: `~/Downloads/transcoded/`
- Creates: `filename_DNxHR_LB.mov`

---

## ⚙️ Settings

### Preset Selector
- **DNxHR LB (Fast)** ⚡ - Fastest, HW accelerated
- ProRes HQ - High quality
- ProRes 422 - Standard quality
- DNxHR HQX - 10-bit 4K
- H.264 High - For delivery

### Options (Per Job)
- **Hardware Acceleration:** ✅ ON (default, keep it on!)
- **Extract BWF:** ☐ OFF (enable for audio-only files)
- **Map All Audio:** ✅ ON (keeps all 8 tracks)

---

## 🚀 Performance

### M2 Max Specs
- **12 cores:** 8 performance + 4 efficiency
- **11 workers:** Maximum parallel jobs
- **VideoToolbox:** Hardware H.264/HEVC decode

### Expected Speed
- Single job: **6.73x realtime**
- 16-minute clip: **~2.4 minutes**
- 11 parallel jobs: **~70x aggregate throughput**

### Example: 11 × 16-minute clips
- **Sequential:** 11 × 15 min = **165 minutes** (2.75 hours)
- **Parallel (HW):** 11 × 2.4 min ÷ 11 = **~30 minutes**
- **Speedup:** **5.5x faster!** ⚡

---

## 📁 Output Files

### Format
- **Container:** QuickTime MOV
- **Video:** DNxHR LB (8-bit, 4:2:2)
- **Audio:** PCM 24-bit 48kHz (all tracks preserved)
- **Bitrate:** ~45 Mbps @ 1080p 24fps

### Avid Compatibility
- ✅ Import via AMA (Link)
- ✅ Import via Consolidate
- ✅ Frame-accurate editing
- ✅ Real-time playback

---

## 🔧 Troubleshooting

### GUI Won't Open
```bash
# Check if it's still building
ps aux | grep tauri

# If hung, restart:
pkill -f "tauri dev"
cd /Users/Editor/Downloads/industrial-transcoder-rust-v1
npx tauri dev
```

### Slow Transcoding
- ✅ Check "Hardware Acceleration" is enabled
- ✅ Verify using "DNxHR LB (Fast)" preset
- ✅ Close other heavy applications

### Missing Audio Tracks
- ✅ Check "Map All Audio" is enabled
- Re-add the job with correct settings

---

## 💡 Pro Tips

### 1. Priority System
- Drag jobs up/down to reorder
- Higher in list = processed first

### 2. Monitor Progress
- Click on a job to see detailed progress
- Speed indicator shows realtime factor
- ETA shows estimated completion time

### 3. Batch Processing
- Add entire folder at once
- All files use same preset
- Progress shown per file

### 4. Pause/Resume
- "Stop Workers" to pause
- "Start Workers" to resume
- Jobs remember progress

---

## 🎬 Workflow Example

### Scenario: Transcode 11 Sony FX9 clips

**Step 1:** Open GUI (auto-opens when ready)

**Step 2:** Select preset
- Choose: "DNxHR LB (Fast)"

**Step 3:** Add files
- Drag all 11 MXF files into window
- OR: Click "Add Job" → Select multiple files

**Step 4:** Verify settings
- Hardware Acceleration: ✅ ON
- Map All Audio: ✅ ON
- Extract BWF: ☐ OFF (unless needed)

**Step 5:** Start
- Click "Start Workers"
- Watch 11 jobs process simultaneously

**Step 6:** Wait
- ~30 minutes for 11 × 16-minute clips
- Go get coffee! ☕

**Step 7:** Import to Avid
- Open Avid Media Composer
- File → Import
- Navigate to output folder
- Link to AMA
- Start editing! 🎬

---

## 📊 Performance Monitor

While transcoding, you'll see:

```
Worker 1: BC_030525_A0012.MXF [████████░░] 87% (6.73x) ETA: 0:23
Worker 2: BC_030525_A0013.MXF [████░░░░░░] 42% (6.81x) ETA: 1:15
Worker 3: BC_030525_A0014.MXF [██░░░░░░░░] 18% (6.65x) ETA: 2:04
...
Worker 11: BC_030525_A0022.MXF [░░░░░░░░░░] 0% (N/A) ETA: --:--

Active: 11/11 workers
Completed: 3/14 jobs
Queue: 0 pending
```

---

## ✅ Ready!

**GUI is building now...**

When it opens:
1. Select "DNxHR LB (Fast)" preset
2. Drag & drop your MXF files
3. Click "Start Workers"
4. Grab coffee! ☕

**Everything is hardware-accelerated and optimized for your M2 Max!** 🚀

