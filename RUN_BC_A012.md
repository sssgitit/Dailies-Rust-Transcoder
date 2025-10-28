# Process BC_A012 Clip

## Quick Start

### Option 1: Detailed Output (Recommended)

```bash
cd /Users/Editor/Downloads/industrial-transcoder-rust-v1

# Make executable
chmod +x process_BC_A012.sh

# Run it!
./process_BC_A012.sh
```

**Shows:**
- ✅ Detailed progress
- ✅ File information
- ✅ Step-by-step status
- ✅ Verification

### Option 2: Quick Version (Less Output)

```bash
chmod +x process_BC_A012_quick.sh
./process_BC_A012_quick.sh
```

**Shows only:**
- Essential status updates
- Final output path

---

## What It Does

**Input:**
```
/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip/BC_030525_A0012.MXF
```

**Steps:**
1. 🔄 **Rewrap** - Clip-wrapped → Frame-wrapped
2. 🎬 **Transcode** - MXF → DNxHR LB MOV

**Output:**
```
processed_BC_A012/
├── 01_frame_wrapped/
│   └── BC_030525_A0012_frame.mxf
└── 02_DNxHR_LB/
    └── BC_030525_A0012_DNxHR_LB.mov  ← Import this to Avid!
```

---

## Requirements

### Must Have:
- ✅ **FFmpeg** - For transcoding
  ```bash
  brew install ffmpeg
  ```

### Optional (but recommended):
- ⭐ **BMX Tools** - For MXF rewrapping
  ```bash
  brew install bmx
  ```
  
  *If not installed, script will skip rewrap and use original file*

---

## Expected Results

### File Sizes (Approximate)

For a 10-minute 4K clip:

| File | Size |
|------|------|
| Original MXF | ~10 GB |
| Frame-wrapped MXF | ~10.2 GB |
| **DNxHR LB MOV** | **~2 GB** ✅ |

DNxHR LB is much smaller - perfect for editing!

### Processing Time

On M1 Max (8 cores):
- **Rewrap:** ~2-3 minutes
- **Transcode:** ~8-12 minutes
- **Total:** ~10-15 minutes

---

## Troubleshooting

### "File not found"

**Check:**
1. Is the drive mounted?
   ```bash
   ls /Volumes/BelleCo_4
   ```

2. Is the path correct?
   ```bash
   ls -la "/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip/BC_030525_A0012.MXF"
   ```

3. Do you have read permissions?
   ```bash
   # Should show file info
   file "/Volumes/BelleCo_4/...path.../BC_030525_A0012.MXF"
   ```

### "bmxtranswrap not found"

**Install BMX:**
```bash
brew install bmx
```

**Or skip rewrap:**
The script will continue with the original file. It may work fine in Avid, but frame-wrapped is recommended.

### "FFmpeg not found"

**Install FFmpeg:**
```bash
brew install ffmpeg
```

**Verify:**
```bash
ffmpeg -version
```

### "Permission denied"

**Make executable:**
```bash
chmod +x process_BC_A012.sh
```

### Slow performance

**Tips:**
- Close other applications
- Use SSD if possible (not external HDD)
- Check available disk space (need 2-3x file size)

---

## Import to Avid

1. **Open Avid Media Composer**
2. **File → Import**
3. **Navigate to:**
   ```
   processed_BC_A012/02_DNxHR_LB/
   ```
4. **Select:**
   ```
   BC_030525_A0012_DNxHR_LB.mov
   ```
5. **Import Options:**
   - Link to AMA ✅ (recommended)
   - Or Consolidate/Transcode

6. **Done!** File links natively as DNxHR

---

## Next Steps

### For multiple clips:

**Batch process all A012 clips:**
```bash
for file in /Volumes/BelleCo_4/.../XDROOT/Clip/BC_030525_A*.MXF; do
    INPUT_FILE="$file" ./process_BC_A012.sh
done
```

**Or use the transcoder queue:**
```bash
# Build the transcoder
cargo build --release

# Add multiple jobs
transcoder transcode --input A0012.mxf --output A0012.mov --preset dnxhr_lb
transcoder transcode --input A0013.mxf --output A0013.mov --preset dnxhr_lb
# ... etc
```

---

## Questions?

- Check `DNXHR_GUIDE.md` for codec details
- Check `MXF_REWRAP_GUIDE.md` for rewrapping info
- Check `GETTING_STARTED.md` for general help

---

**Ready to process!** 🚀

Run: `./process_BC_A012.sh`

