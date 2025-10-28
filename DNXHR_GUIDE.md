# DNxHR Guide - Avid's 4K+ Codec

## Overview

**DNxHR** (DNx High Resolution) is Avid's codec for resolutions beyond HD, supporting 4K, 6K, 8K, and higher.

### DNxHR vs DNxHD

| Feature | DNxHD | DNxHR |
|---------|-------|-------|
| Max Resolution | 1920×1080 (HD) | Unlimited (4K, 8K+) |
| Bit Depth | 8-bit, 10-bit | 8-bit, 10-bit, 12-bit |
| Color Space | 4:2:2 | 4:2:2, 4:4:4 |
| Use Case | HD workflows | 4K/UHD/8K workflows |

**Key Point:** For 4K and above, use **DNxHR**. For HD (1080p), either works.

---

## DNxHR Profiles

Industrial Transcoder supports all five DNxHR profiles:

### 1. **DNxHR LB** (Low Bandwidth)
- **Bit Depth:** 8-bit
- **Quality:** Lowest
- **Bitrate:** ~45 Mbps @ 1080p, ~180 Mbps @ 4K
- **Use Case:** Web delivery, streaming, low storage

### 2. **DNxHR SQ** (Standard Quality) ⭐
- **Bit Depth:** 8-bit
- **Quality:** Good for offline editing
- **Bitrate:** ~100 Mbps @ 1080p, ~400 Mbps @ 4K
- **Use Case:** Offline editing, dailies proxies

### 3. **DNxHR HQ** (High Quality) ⭐⭐
- **Bit Depth:** 8-bit
- **Quality:** High quality, good for most workflows
- **Bitrate:** ~145 Mbps @ 1080p, ~580 Mbps @ 4K
- **Use Case:** Online editing, color grading prep

### 4. **DNxHR HQX** (High Quality 10-bit) ⭐⭐⭐ **[RECOMMENDED]**
- **Bit Depth:** 10-bit
- **Quality:** Very high, minimal banding
- **Bitrate:** ~220 Mbps @ 1080p, ~880 Mbps @ 4K
- **Use Case:** Professional editing, color grading, finishing

### 5. **DNxHR 444** (Highest Quality)
- **Bit Depth:** 10-bit or 12-bit
- **Color Space:** 4:4:4 (full color resolution)
- **Quality:** Maximum quality, lossless-like
- **Bitrate:** ~440 Mbps @ 1080p, ~1760 Mbps @ 4K
- **Use Case:** VFX, chroma keying, final mastering

---

## Usage

### Desktop App

1. **Open Industrial Transcoder**
2. **Add Job**
3. **Select Preset:**
   - "DNxHR HQX" (recommended for 4K)
   - "DNxHR HQ" (8-bit alternative)
   - "DNxHR SQ" (offline/proxy)
4. **Start Transcoding**

### CLI

```bash
# DNxHR HQX (10-bit, recommended)
transcoder transcode \
  --input A012_car.mxf \
  --output A012_car_DNxHR_HQX.mov \
  --preset dnxhr_hqx

# DNxHR HQ (8-bit)
transcoder transcode \
  --input A012_car.mxf \
  --output A012_car_DNxHR_HQ.mov \
  --preset dnxhr_hq

# DNxHR SQ (offline proxy)
transcoder transcode \
  --input A012_car.mxf \
  --output A012_car_DNxHR_SQ.mov \
  --preset dnxhr_sq
```

### TypeScript API

```typescript
import { addJob } from './api/transcoder-api';

// Add DNxHR HQX job
await addJob({
  input_path: '/path/to/A012_car.mxf',
  output_path: '/path/to/A012_car_DNxHR_HQX.mov',
  preset_name: 'DNxHR HQX',
  priority: 'High'
});
```

---

## Complete A012 Car Workflow (with DNxHR)

### Shell Script

```bash
#!/bin/bash
# A012 Car - DNxHR Workflow

INPUT="A012_car.mxf"
OUTPUT_DIR="./processed"

# Step 1: Rewrap to frame-wrapped
bmxtranswrap -t op1a --frame-layout separate \
  -o "$OUTPUT_DIR/A012_car_frame.mxf" \
  "$INPUT"

# Step 2: Transcode to DNxHR HQX (10-bit, 4K)
transcoder transcode \
  --input "$OUTPUT_DIR/A012_car_frame.mxf" \
  --output "$OUTPUT_DIR/A012_car_DNxHR_HQX.mov" \
  --preset dnxhr_hqx

# Step 3: Create DNxHR SQ proxy
transcoder transcode \
  --input "$OUTPUT_DIR/A012_car_frame.mxf" \
  --output "$OUTPUT_DIR/A012_car_DNxHR_SQ.mov" \
  --preset dnxhr_sq

echo "✓ DNxHR workflow complete!"
```

---

## File Sizes & Performance

### A012 Car Example (10 minutes @ 4K 23.976fps)

| Format | File Size | Quality | Use Case |
|--------|-----------|---------|----------|
| Original MXF | 10 GB | Source | - |
| DNxHR 444 | 22 GB | Maximum | VFX, mastering |
| **DNxHR HQX** | **11 GB** | **Very High** | **Editing ⭐** |
| DNxHR HQ | 7 GB | High | Editing |
| DNxHR SQ | 5 GB | Good | Offline/proxy |
| ProRes HQ | 25 GB | Very High | Editing |

**Note:** DNxHR HQX offers excellent quality at roughly half the file size of ProRes HQ.

### Transcode Speed (M1 Max, 8 cores)

| Profile | Speed | Time (10 min footage) |
|---------|-------|----------------------|
| DNxHR 444 | ~0.6x | ~17 minutes |
| DNxHR HQX | ~0.8x | ~13 minutes |
| DNxHR HQ | ~1.0x | ~10 minutes |
| DNxHR SQ | ~1.2x | ~8 minutes |

---

## Avid Media Composer Integration

### Import Settings

1. **File → Import**
2. Select DNxHR MOV files
3. **Link to AMA** or **Consolidate/Transcode**
4. DNxHR files link natively (no transcode needed!)

### Project Settings

- **Format:** 3840×2160 23.976p (for 4K)
- **Codec:** DNxHR (auto-detected)
- **Bit Depth:** 10-bit (for HQX)

### Timeline Performance

DNxHR is optimized for Avid:
- **Realtime playback** (no rendering)
- **Fast scrubbing**
- **Efficient editing**
- **Direct export**

---

## Comparison with ProRes

### ProRes vs DNxHR (4K)

| Feature | ProRes HQ | DNxHR HQX |
|---------|-----------|-----------|
| Bit Depth | 10-bit | 10-bit |
| Quality | Excellent | Excellent |
| File Size @ 4K | ~250 MB/min | ~110 MB/min |
| Avid Native | Via plugin | Native ✅ |
| FCP Native | Native ✅ | Via plugin |
| Premiere | Both work | Both work |

**Recommendation:**
- **Avid workflows:** DNxHR HQX
- **FCP workflows:** ProRes HQ
- **Premiere/Resolve:** Either works

---

## Advanced Options

### Custom Resolution

```bash
# Transcode with specific resolution
transcoder transcode \
  --input 8K_source.mxf \
  --output 4K_output.mov \
  --preset dnxhr_hqx \
  --resolution 3840x2160
```

### Custom Frame Rate

```bash
# Force specific frame rate
transcoder transcode \
  --input variable_fps.mp4 \
  --output constant_fps.mov \
  --preset dnxhr_hq \
  --frame-rate 23.976
```

---

## Troubleshooting

### "Codec not found: dnxhd"

**Solution:** Update FFmpeg
```bash
# macOS
brew upgrade ffmpeg

# Linux
sudo apt-get update && sudo apt-get upgrade ffmpeg
```

### File won't import to Avid

**Check:**
1. Is it in MOV container? ✅
2. Audio at 48kHz? ✅
3. Is it actually DNxHR? Check with:
```bash
ffprobe file.mov | grep -i dnxhr
```

### Quality looks poor

**Solutions:**
- Use **DNxHR HQX** (not SQ) for mastering
- Check source file quality
- Ensure correct resolution maintained

### File size too large

**Options:**
1. Use **DNxHR HQ** instead of HQX (8-bit vs 10-bit)
2. Use **DNxHR SQ** for proxies
3. Lower resolution if 4K not needed

---

## Best Practices

### For Dailies
1. **Rewrap MXF** to frame-wrapped
2. **Transcode to DNxHR HQX** for editing
3. **Create DNxHR SQ** for offline

### For Delivery
- Use **DNxHR HQX** or **DNxHR 444** for broadcast
- Check delivery specs (some require specific profiles)

### For Archive
- Keep original source files
- Archive **DNxHR 444** if max quality needed
- Otherwise **DNxHR HQX** is sufficient

---

## Integration with Existing Tools

### Complete Pipeline

```bash
# 1. Rewrap MXF
bmxtranswrap -t op1a --frame-layout separate \
  -o frame_wrapped.mxf clip_wrapped.mxf

# 2. Transcode to DNxHR
transcoder transcode \
  --input frame_wrapped.mxf \
  --output dnxhr.mov \
  --preset dnxhr_hqx

# 3. Add BWF audio timecode (if needed)
python3 bwf-tools/insert_bext_timecode.py \
  audio.wav audio_bext.wav \
  --time-ref 2307276429
```

---

## Resources

- [Avid DNx Codec Specifications](https://www.avid.com/dnxhr)
- [FFmpeg DNxHD/DNxHR Documentation](https://trac.ffmpeg.org/wiki/Encode/DNxHD)

---

**Version:** 2.0.0  
**Updated:** October 2025  
**Recommended Profile:** DNxHR HQX (10-bit) for 4K editing

