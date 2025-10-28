# Simple Two-Tool Workflow

## Overview
Use **Industrial Transcoder** for fast video, and your **validated BWF tools** for audio.

---

## Step 1: Video Transcode (GUI)

**Tool:** Industrial Transcoder GUI  
**Speed:** 7-8x realtime on M2 Max

1. Open Industrial Transcoder
2. Click "Add Job"
3. Select input MXF file
4. Choose output directory
5. Preset: "DNxHR LB (Fast)" (default)
6. Click "Add Job"
7. Click "Start Workers"

**Output:** `filename_transcoded.mov` (DNxHR LB, Avid-ready)

---

## Step 2: BWF Audio (Command Line)

**Tool:** Your validated bwf-tools  
**Method:** Frame-accurate BEXT timecode @ 23.976fps

### Extract Audio + Add BEXT Timecode

```bash
cd /Users/Editor/Downloads/industrial-transcoder-rust-v1/bwf-tools

# Extract audio from source MXF
INPUT="/path/to/source.MXF"
OUTPUT="/path/to/output_audio.wav"

# Get timecode from source
TIMECODE=$(ffprobe -v quiet -show_entries format_tags:stream_tags "$INPUT" | grep timecode= | cut -d= -f2)

echo "Source timecode: $TIMECODE"

# Calculate TimeReference using validated method
TIME_REF=$(python3 frame_based_bext_calculator.py "$TIMECODE" | grep "BEXT TimeReference:" | awk '{print $3}')

# Extract audio and insert BEXT
python3 insert_bext_timecode.py \
  "$INPUT" \
  "$OUTPUT" \
  --time-ref "$TIME_REF" \
  --sample-rate 48000 \
  --frame-rate 23.976 \
  --description "$TIMECODE" \
  --originator "Industrial Transcoder v2"
```

---

## Batch Processing

### Process Multiple Files

```bash
cd /Users/Editor/Downloads/industrial-transcoder-rust-v1

# 1. Add all video files to GUI queue
# 2. Start workers (processes all in parallel)

# 3. While video transcodes, prepare BWF script:
for file in /path/to/sources/*.MXF; do
  basename=$(basename "$file" .MXF)
  
  # Extract timecode
  TC=$(ffprobe -v quiet -show_entries format_tags:stream_tags "$file" | grep timecode= | cut -d= -f2)
  
  # Calculate TimeReference
  TIME_REF=$(python3 bwf-tools/frame_based_bext_calculator.py "$TC" | grep "BEXT TimeReference:" | awk '{print $3}')
  
  # Create BWF
  python3 bwf-tools/insert_bext_timecode.py \
    "$file" \
    "/output/dir/${basename}_audio.wav" \
    --time-ref "$TIME_REF" \
    --sample-rate 48000 \
    --frame-rate 23.976 \
    --description "$TC" \
    --originator "Industrial Transcoder v2"
done
```

---

## Why This Works

### Separation of Concerns
- **Video:** Complex, benefits from GUI + parallel processing
- **Audio:** Simple, linear process with validated tools

### Validated Components
- **Industrial Transcoder:** Hardware-accelerated video encode
- **BWF Tools:** Frame-accurate BEXT timecode (tested & validated)

### Performance
- Both can run simultaneously
- Video: 7-8x realtime with 11 workers
- Audio: Minimal CPU, I/O bound

---

## Output Files

After both steps:
```
output_dir/
  ├── clip_001_transcoded.mov     (DNxHR LB video + 8ch audio)
  ├── clip_001_audio.wav           (Stereo BWF with BEXT timecode)
  ├── clip_002_transcoded.mov
  ├── clip_002_audio.wav
  └── ...
```

---

## Import to Avid

1. **For video editing:** Import the `.mov` files directly via AMA or consolidate
2. **For audio post:** Use the `.wav` BWF files with frame-accurate timecode
3. **Both have matching timecode** for perfect sync

---

## Troubleshooting

### Video Transcode Issues
- Check: Output directory has space
- Check: FFmpeg available (`which ffmpeg`)
- Check: Source file is readable

### BWF Audio Issues
- Check: Python 3 available
- Check: Source timecode exists (`ffprobe` output)
- Check: bwf-tools scripts are executable

---

## Performance Tips

### For Maximum Speed

1. **Output to fast drive** (SSD preferred)
2. **Use 4-6 workers** if reading from slow network drive
3. **Process BWF in parallel** while video transcodes
4. **Monitor disk space** (DNxHR LB ≈ 45 Mbps = ~320 MB/min)

### For Best Quality

- Source: Ensure original MXF files are not corrupted
- Video: DNxHR LB is production-ready for Avid
- Audio: BWF maintains full 48kHz 24-bit quality

---

**Version:** 2.0  
**Last Updated:** October 28, 2025  
**Status:** Production Ready ✅

