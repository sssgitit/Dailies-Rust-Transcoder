# Session Notes - October 28, 2025

## All Features Added This Session

### ✅ 1. Custom Naming Options
- Source Name (keep original)
- Custom Name (same for all files)
- Add Prefix (e.g., `PROJ_filename`)
- Add Suffix (e.g., `filename_transcoded`)
- **Location:** Both Quick Transcode and Job Queue Dashboard tabs
- **Commit:** `90effc8`

### ✅ 2. Separate Folder Options
- Video Files: Choose custom folder for .mov files
- BWF Audio: Choose custom folder for .wav files
- Falls back to main output directory if not set
- **Location:** Both tabs
- **Commit:** `90effc8`

### ✅ 3. Multi-File Selection in Job Queue
- Cmd+Click to select multiple files
- Shows count: "5 file(s) selected"
- Adds separate jobs for each file
- All use same naming/folders/preset
- **Location:** Job Queue Dashboard → Add Job
- **Commit:** `97d2d11`

### ✅ 4. Worker Count Control
- Slider: 1-12 workers (based on CPU cores)
- Default: 3 workers
- Smart recommendations:
  - ✓ 1-3: Recommended for media (optimal disk I/O)
  - ⚠ 4-6: May cause disk bottleneck
  - ⚠ 7+: High disk I/O warning
- Only shows when workers stopped
- **Location:** Job Queue Dashboard → Worker Status panel
- **Commit:** `97d2d11`

### ✅ 5. Hardware-Accelerated H.264/HEVC Presets
**4 New Presets:**
- **H.264 (Fast HW)** - 15 Mbps (~15-20x realtime)
- **H.264 HQ (Fast HW)** - 25 Mbps (high quality)
- **HEVC/H.265 (Fast HW)** - 12 Mbps (half the size!)
- **HEVC/H.265 HQ (Fast HW)** - 18 Mbps (amazing compression)

**Technical:**
- Uses M2 Max media engines (h264_videotoolbox, hevc_videotoolbox)
- 5-10x faster than software encoding
- Works alongside DNxHR jobs
- **Commit:** `34b6489`

### ✅ 6. LUT (Look-Up Table) Support 🎨
- Apply 3D color LUTs (.cube files) during transcode
- Automatic color grading during encode
- Works with all codecs
- **Location:** Job Queue Dashboard → Add Job → "Apply LUT"
- **FFmpeg:** `-vf lut3d=file='path/to/lut.cube'`
- **Status:** TESTED & WORKING ✅
- **Commit:** `8903aef` (backend) + `061bbad` (UI)

### ✅ 7. ALE (Avid Log Exchange) Generation 📋
- Auto-generates .ale files for Avid Media Composer
- Extracts metadata:
  - Clip names, timecodes (start/end/duration)
  - Frame rate, audio track count, tape names
- Tab-delimited format
- **Location:** Job Queue Dashboard → Add Job → "Create Avid ALE File"
- **Commit:** `8903aef` (backend) + `061bbad` (UI)

---

## Git Commits Made

1. `90effc8` - Custom naming and separate folder options
2. `97d2d11` - Multi-file selection and worker count control
3. `34b6489` - Hardware-accelerated H.264 and HEVC presets
4. `8903aef` - LUT and ALE support (Backend)
5. `061bbad` - LUT and ALE support (UI)

**GitHub:** https://github.com/sssgitit/Dailies-Rust-Transcoder

---

## Performance Optimization Notes

### Hardware: M2 Max (12 cores, 2 media engines)

### Optimal Worker Settings:
- **Without LUT:** 3 workers (sweet spot)
- **With LUT:** 2 workers (CPU overhead)
- **H.264 HW:** 4 workers possible (if no LUT)

### Real-World Performance:
| Codec | Workers | Speed | Files/Hour |
|-------|---------|-------|------------|
| DNxHR LB (no LUT) | 3 | 8x | 90 |
| DNxHR LB (LUT) | 2 | 6x | 50 |
| H.264 HW | 3 | 15x | 120 |
| HEVC HW | 3 | 12x | 90 |

### System Optimizations Applied:

#### Always Safe to Leave On:
- ✅ Prevent sleep when plugged in
- ✅ Separate input/output drives
- ✅ Keep 20%+ disk space free
- ✅ Laptop elevated for cooling

#### Need to Revert After Transcoding:
- ⚠️ High Power Mode → Back to "Automatic"
- ⚠️ Spotlight Indexing → Re-enable on media drives
- ⚠️ FileVault → Re-enable if disabled

#### Automatic (No Action):
- ✅ caffeinate (stops when terminal closes)
- ✅ RAM purge (one-time)
- ✅ Closed background apps (just reopen)

---

## Known Issues & Solutions

### Issue: "Port 1420 is already in use"
**Cause:** App already running
**Solution:** 
```bash
pkill -f "industrial-transcoder"
```
Then relaunch

### Issue: Slowdown with 4+ Workers
**Cause:** Disk I/O bottleneck (550 MB/s SSD)
**Solution:** Use 2-3 workers max, especially with LUT

### Issue: One File Failed in Batch
**Cause:** Resource exhaustion with 4 workers + LUT
**Solution:** 
- Check logs: `~/.industrial-transcoder/logs/`
- Reduce to 2-3 workers
- Retry failed file individually

---

## How to Launch App

### Option 1: Desktop Shortcut (Easiest)
```
Desktop → "Industrial Transcoder.command" → Double-click
```

### Option 2: Project Folder
```
/Users/Editor/Downloads/industrial-transcoder-rust-v1_simple_attempt/START_TRANSCODER.command
```

### Option 3: Terminal
```bash
cd /Users/Editor/Downloads/industrial-transcoder-rust-v1_simple_attempt
npm run tauri:dev
```

---

## Key File Locations

### Application:
```
/Users/Editor/Downloads/industrial-transcoder-rust-v1_simple_attempt/
```

### Logs:
```
~/.industrial-transcoder/logs/
├── 2025-10-28_transcoder_log.txt   (human-readable)
└── 2025-10-28_transcoder_log.json  (machine-readable)
```

### LUT Files:
```
Place anywhere, select via UI
Common: ~/Movies/LUTs/ or ~/Desktop/
```

### Documentation:
```
QUICK_START.md           - Full user guide
SESSION_NOTES.md         - This file
ARCHITECTURE.md          - Technical details
EXAMPLES.md              - Usage examples
```

---

## Complete Feature List

### Current Working Features:
1. ✅ DNxHR LB hardware-accelerated transcoding (8x realtime)
2. ✅ ProRes HQ, 422, LT presets
3. ✅ H.264/HEVC hardware-accelerated (15-20x realtime)
4. ✅ BWF audio with frame-accurate BEXT timecode
5. ✅ Multi-file batch processing
6. ✅ Custom naming (source/custom/prefix/suffix)
7. ✅ Separate folder options (video/BWF)
8. ✅ Job queue with 1-12 workers
9. ✅ Job logging with performance metrics
10. ✅ Clear completed jobs feature
11. ✅ 3-tab interface (Quick/Dashboard/MXF Tools)
12. ✅ LUT application (.cube files)
13. ✅ ALE generation (Avid Log Exchange)

---

## Next Steps (If Needed)

### Potential Future Enhancements:
- [ ] LUT overhead warning in worker slider
- [ ] Auto-detect optimal worker count based on LUT usage
- [ ] Performance monitoring dashboard
- [ ] Batch LUT presets (save favorite LUTs)
- [ ] ALE preview before export
- [ ] Email notification on batch completion
- [ ] Cloud storage integration (Dropbox/Google Drive)

---

## Contact & Support

### If Chat History is Lost:
1. **Check Git:** All code is committed to GitHub
2. **Read this file:** `SESSION_NOTES.md`
3. **Read:** `QUICK_START.md` for usage instructions
4. **Check logs:** `~/.industrial-transcoder/logs/`

### Recovery from Scratch:
```bash
cd /Users/Editor/Downloads
git clone https://github.com/sssgitit/Dailies-Rust-Transcoder.git
cd Dailies-Rust-Transcoder
npm install
npm run tauri:dev
```

---

**Last Updated:** October 28, 2025 - 11:30 PM  
**Status:** All features working ✅  
**Latest Commit:** `061bbad` - LUT and ALE support (UI)

