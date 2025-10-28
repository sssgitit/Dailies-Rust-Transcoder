# MXF Rewrap Guide: Clip-Wrapped vs Frame-Wrapped

## The Problem

**You have clip-wrapped MXF files, but your editing system needs frame-wrapped files.**

This is a common issue in professional video workflows, especially when working with:
- Avid Media Composer
- Adobe Premiere Pro
- Final Cut Pro
- DaVinci Resolve

## Understanding the Difference

### Clip-Wrapped (Contiguous)
```
┌─────────────────────────────────────┐
│  Video Track:  [VVVVVVVVVVVVVVVVV] │
│  Audio Track:  [AAAAAAAAAAAAAAAAAA] │
└─────────────────────────────────────┘
```
- Essence stored in large contiguous chunks
- One chunk per track
- **Pros:** More efficient for playback, smaller overhead
- **Cons:** Harder to edit frame-by-frame

### Frame-Wrapped (Interleaved)
```
┌─────────────────────────────────────┐
│  [V][A][V][A][V][A][V][A][V][A]... │
└─────────────────────────────────────┘
```
- Essence interleaved frame-by-frame
- Video and audio alternating
- **Pros:** Easier for editing systems, frame-accurate
- **Cons:** Slightly larger file size, more overhead

## When You Need Frame-Wrapped

### Editing Systems Prefer Frame-Wrapped
- **Avid Media Composer** - Requires frame-wrapped for OP1a
- **Premiere Pro** - Better compatibility with frame-wrapped
- **Final Cut Pro** - Prefers frame-wrapped for scrubbing
- **DaVinci Resolve** - More stable with frame-wrapped

### Signs Your Files Are Clip-Wrapped

❌ **Symptoms:**
- "Could not open file" errors in editing software
- Audio sync issues during playback
- Slow scrubbing performance
- Dropouts when playing backwards
- Cannot insert markers at specific frames

## Solution: Rewrap with Industrial Transcoder

### Desktop App

1. **Open MXF Rewrap Tool**
   - Launch Industrial Transcoder
   - Navigate to "MXF Tools" tab

2. **Select Input File**
   - Click "Browse" for input
   - Select your clip-wrapped MXF file
   - Tool will auto-detect wrapping type

3. **Choose Target Wrapping**
   - Select "Frame-Wrapped" (🎬)
   - Auto-generates output filename

4. **Rewrap**
   - Click "🔄 Rewrap MXF File"
   - Wait for completion

### Command Line

```bash
# Single file: Clip → Frame
transcoder mxf-rewrap \
  --input footage.mxf \
  --output footage_frame.mxf \
  --target frame

# Single file: Frame → Clip
transcoder mxf-rewrap \
  --input footage.mxf \
  --output footage_clip.mxf \
  --target clip

# Batch processing
transcoder mxf-batch-rewrap \
  --input-dir ./clip_wrapped/ \
  --output-dir ./frame_wrapped/ \
  --target frame
```

### TypeScript API

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Detect current wrapping
const wrapping = await invoke('detect_mxf_wrapping', {
  inputPath: '/path/to/file.mxf'
});

console.log('Current wrapping:', wrapping);
// Output: "ClipWrapped" or "FrameWrapped"

// Rewrap to frame-wrapped
await invoke('clip_to_frame', {
  inputPath: '/path/to/input.mxf',
  outputPath: '/path/to/output.mxf'
});

// Or use the full rewrap command
await invoke('rewrap_mxf', {
  request: {
    input_path: '/path/to/input.mxf',
    output_path: '/path/to/output.mxf',
    target_wrapping: 'FrameWrapped'  // or 'ClipWrapped'
  }
});
```

### Batch Script (Shell)

```bash
#!/bin/bash
# rewrap_all.sh - Convert all clip-wrapped MXF files to frame-wrapped

INPUT_DIR="./footage"
OUTPUT_DIR="./frame_wrapped"

mkdir -p "$OUTPUT_DIR"

for file in "$INPUT_DIR"/*.mxf; do
  basename=$(basename "$file")
  output="$OUTPUT_DIR/$basename"
  
  echo "Rewrapping: $basename"
  
  transcoder mxf-rewrap \
    --input "$file" \
    --output "$output" \
    --target frame
  
  if [ $? -eq 0 ]; then
    echo "✓ Success: $basename"
  else
    echo "✗ Failed: $basename"
  fi
done

echo "Batch rewrap complete!"
```

## Requirements

### BMX Tools (Required)

MXF rewrapping uses `bmxtranswrap` from the BMX project.

#### Install on macOS
```bash
brew install bmx
```

#### Install on Linux (Ubuntu/Debian)
```bash
# Install dependencies
sudo apt-get install git cmake build-essential \
  uuid-dev libexpat1-dev libmxf-dev

# Clone and build
git clone https://github.com/ebu/bmx.git
cd bmx
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build .
sudo make install
```

#### Install on Windows
Download pre-built binaries from:
https://github.com/ebu/bmx/releases

Add to system PATH.

### Verify Installation

```bash
bmxtranswrap --version
```

Or in the app:
```typescript
const available = await invoke('is_mxf_rewrapping_available');
console.log('MXF rewrapping available:', available);
```

## Technical Details

### What Happens During Rewrapping

1. **Parse MXF header** - Read metadata and structure
2. **Identify essence locations** - Find video/audio data
3. **Re-index content** - Build new index tables
4. **Rewrite essence** - Reorganize data on disk
5. **Update metadata** - Adjust descriptors for new layout

### Performance

| File Size | Time (Clip→Frame) | Time (Frame→Clip) |
|-----------|-------------------|-------------------|
| 1 GB      | ~15-30 seconds    | ~10-20 seconds    |
| 10 GB     | ~2-5 minutes      | ~1-3 minutes      |
| 100 GB    | ~20-50 minutes    | ~10-30 minutes    |

**Note:** Times vary based on disk speed and system performance. SSD recommended.

### File Size Changes

- **Clip → Frame:** Typically +2-5% due to index overhead
- **Frame → Clip:** Typically -2-5% due to reduced index size

## Common Issues & Solutions

### Error: "bmxtranswrap not found"

**Solution:** Install BMX tools (see Requirements above)

### Error: "Could not detect wrapping"

**Solution:** File may be corrupted or not a valid MXF file. Try:
```bash
ffmpeg -i file.mxf -c copy file_fixed.mxf
```

### Error: "Permission denied"

**Solution:** Check file permissions
```bash
chmod 644 input.mxf
chmod 755 output_directory/
```

### Output file is corrupted

**Causes:**
- Disk full during rewrap
- Process interrupted
- Bad source file

**Solution:**
- Verify source file integrity
- Ensure sufficient disk space (2x input file size)
- Re-run rewrap operation

### Editing software still can't open file

**Check:**
1. Is file actually frame-wrapped now?
   ```bash
   bmxtranswrap --info output.mxf | grep -i wrap
   ```

2. Does your editing software support MXF OP1a?
   - Avid: Yes (native)
   - Premiere: Yes
   - FCP: Requires plugin
   - Resolve: Yes

3. Try rewrapping to a different operational pattern:
   ```bash
   transcoder mxf-rewrap \
     --input file.mxf \
     --output file_op1a.mxf \
     --target frame \
     --op-pattern op1a
   ```

## Integration with Existing Tools

The MXF rewrap tool integrates with your existing MXF workflow:

### 1. Unified MOB IDs First
```bash
# Step 1: Unify MOB IDs (for multi-track files)
./mxf-tools/scripts/unify_mob_id.sh video.mxf audio.mxf

# Step 2: Rewrap to frame-wrapped
transcoder mxf-rewrap \
  --input video.mxf \
  --output video_frame.mxf \
  --target frame
```

### 2. Rewrap Then Transcode
```bash
# Step 1: Rewrap MXF
transcoder mxf-rewrap \
  --input source.mxf \
  --output source_frame.mxf \
  --target frame

# Step 2: Transcode to ProRes
transcoder transcode \
  --input source_frame.mxf \
  --output output.mov \
  --preset prores_hq
```

## Best Practices

### For Dailies Workflow

1. **Receive camera files** (usually clip-wrapped)
2. **Rewrap to frame-wrapped** for editing
3. **Transcode to ProRes** if needed
4. **Deliver to editorial**

```bash
# Automated dailies pipeline
for file in camera/*.mxf; do
  # Rewrap
  transcoder mxf-rewrap \
    --input "$file" \
    --output "frame_wrapped/$(basename "$file")" \
    --target frame
  
  # Transcode
  transcoder transcode \
    --input "frame_wrapped/$(basename "$file")" \
    --output "prores/$(basename "$file" .mxf).mov" \
    --preset prores_hq
done
```

### For Archive

- Keep **clip-wrapped** for long-term storage (smaller size)
- Convert to **frame-wrapped** only when needed for editing

### For Delivery

- Check delivery specs
- Some broadcast specs require specific wrapping
- Frame-wrapped is generally safer for compatibility

## Performance Tips

1. **Use SSD** - Much faster than HDD for rewrapping
2. **Batch processing** - Rewrap multiple files in parallel
3. **Sufficient RAM** - Minimum 8GB recommended
4. **Close other apps** - Free up system resources

## Further Reading

- [BMX Documentation](https://github.com/ebu/bmx)
- [MXF Standards (SMPTE)](https://www.smpte.org/)
- [Avid MXF Workflows](https://www.avid.com/)

---

**Need Help?** Open an issue on GitHub

**Version:** 2.0.0  
**Works with:** MXF OP-Atom, OP1a, OP1b  
**Cross-Platform:** ✅ macOS, Windows, Linux

