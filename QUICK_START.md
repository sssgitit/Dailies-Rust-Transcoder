# 🚀 Quick Start - Industrial Transcoder

**Last Stable Version:** `8993ce5` (Oct 28, 2025)  
**Status:** ✅ FULLY WORKING - BWF with correct BEXT timecode

## 🎯 Fastest Way to Open the App

### Option 1: Double-Click Launch (Easiest!)

1. **Navigate to:** `/Users/Editor/Downloads/industrial-transcoder-rust-v1_simple_attempt/`
2. **Double-click:** `START_TRANSCODER.command`
3. **Wait** for app to compile and open (30-60 seconds first time)

That's it! The app will open automatically.

### Option 2: From Terminal

```bash
cd /Users/Editor/Downloads/industrial-transcoder-rust-v1_simple_attempt
npm run tauri:dev
```

---

## ✅ What's Working Right Now

### Tab 1: 🎬 Quick Transcode & BWF
- ✅ Multi-file batch processing (Cmd+Click to select multiple)
- ✅ DNxHR LB hardware-accelerated (8x realtime)
- ✅ BWF audio with frame-accurate BEXT timecode
- ✅ Sequential processing of all selected files

### Tab 2: 📊 Job Queue Dashboard  
- ✅ Add jobs with "+ Add Job" button
- ✅ **BWF checkbox option** - Creates both video + BWF audio!
- ✅ BEXT timecode correctly embedded (e.g., 21:15:35:19 = TimeRef 3677394066)
- ✅ Worker pool with 11 workers (automatic)
- ✅ **"🗑️ Clear Completed"** button - Logs jobs before clearing
- ✅ Detailed job logs saved to `~/.industrial-transcoder/logs/`

### Tab 3: 🎞️ MXF Tools
- MXF rewrap utilities

---

## 📂 Where Everything Is Saved

### Output Files
- Video: Same directory as input, with `_transcoded.mov` suffix
- BWF Audio: Same directory as input, with `_transcoded.wav` suffix

### Job Logs
```
~/.industrial-transcoder/logs/
├── 2025-10-28_transcoder_log.txt     # Human-readable
└── 2025-10-28_transcoder_log.json    # Machine-readable JSON
```

**Logs include:**
- Codec specs (DNxHR LB, ProRes, etc.)
- System info (CPU, RAM, FFmpeg version)
- Performance (read/write speeds: ~98 MB/s read, ~44 MB/s write)
- File sizes and durations
- BEXT TimeReference values
- Error messages (if any)

---

## 🎬 Typical Workflow

### For Quick Batch Processing (Tab 1):
1. Click **"🎬 Quick Transcode & BWF"** tab
2. Click **"Select Files"**
3. **Cmd+Click** to select multiple MXF files
4. Choose output directory
5. Check ✅ **"DNxHR LB QuickTime"** and/or **"BWF Audio"**
6. Click **"▶ Start Transcode"**
7. Watch progress - processes files one by one

### For Managed Queue Processing (Tab 2):
1. Click **"📊 Job Queue Dashboard"** tab
2. Click **"Start Workers"** (if not already running)
3. Click **"+ Add Job"**
4. Select input file and output directory
5. Choose preset (DNxHR LB, ProRes HQ, etc.)
6. **Check ✅ "Also Create BWF Audio (WAV)"** if you want BWF
7. Set priority (Low, Normal, High, Urgent)
8. Click **"Add Job"**
9. Repeat for more files - they'll queue up
10. When done, click **"🗑️ Clear Completed"** to log and clear

---

## 🔧 Technical Details

### System Specs (Your Machine):
- **CPU:** Apple M2 Max (12 cores)
- **Workers:** 11 (cores - 1)
- **Hardware Acceleration:** VideoToolbox
- **FFmpeg:** 8.0

### Performance:
- **Encoding speed:** 8x realtime (2 minutes for 16-minute file)
- **Disk read:** ~98 MB/s
- **Disk write:** ~44 MB/s  
- **Optimal concurrent jobs:** 2-3 (due to media engines + disk I/O)

### BWF BEXT Timecode:
- **Frame rate:** 23.976 fps
- **Formula:** TimeReference = Total_Frames × 2004.005263
- **Example:** 21:15:35:19 → TimeReference = 3677394066
- **Script:** `bwf-tools/insert_bext_timecode.py` (found automatically)

---

## 🆘 Troubleshooting

### App won't start?
```bash
# Kill any running instances
pkill -f "industrial-transcoder"

# Restart
npm run tauri:dev
```

### Port already in use?
```bash
lsof -ti:1420 | xargs kill -9
```

### Need to reinstall?
```bash
cd /Users/Editor/Downloads/industrial-transcoder-rust-v1_simple_attempt
rm -rf node_modules
npm install
npm run tauri:dev
```

### BWF timecode showing 1:00:00:00?
- Check terminal for "Found BEXT script at..." message
- Should see: `../bwf-tools/insert_bext_timecode.py`
- If not found, the script path needs fixing

---

## 📥 Restore from GitHub

If you need to get back to this exact working version:

```bash
cd /Users/Editor/Downloads
rm -rf industrial-transcoder-rust-v1_simple_attempt
git clone https://github.com/sssgitit/Dailies-Rust-Transcoder.git industrial-transcoder-rust-v1_simple_attempt
cd industrial-transcoder-rust-v1_simple_attempt
npm install
npm run tauri:dev
```

Or just pull latest:
```bash
cd /Users/Editor/Downloads/industrial-transcoder-rust-v1_simple_attempt
git pull
npm run tauri:dev
```

---

## 📌 Bookmark This!

**Main folder:** `/Users/Editor/Downloads/industrial-transcoder-rust-v1_simple_attempt/`  
**Launch script:** `START_TRANSCODER.command` (just double-click!)  
**GitHub:** https://github.com/sssgitit/Dailies-Rust-Transcoder  
**Latest commit:** `8993ce5` - "WORKING VERSION - BWF with correct BEXT timecode"

---

**This is the stable, tested, working version!** ✅🎉

