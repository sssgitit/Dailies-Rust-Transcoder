#!/bin/bash
# Finish BC_A012 - Create Avid OP-Atom MXF from DNxHR MOV

set -e

export DYLD_LIBRARY_PATH=/Users/Editor/Downloads/dnxhr-bmx/lib:/Users/Editor/Downloads/local/lib

SOURCE_MOV="/Users/Editor/Downloads/industrial-transcoder-rust-v1/processed_BC_A012/02_DNxHR_LB/BC_030525_A0012_DNxHR_LB_clip.mov"
OUTPUT_DIR="/Users/Editor/Downloads/industrial-transcoder-rust-v1/processed_BC_A012/03_AVID_OPATOM"

echo "========================================"
echo "Creating Avid OP-Atom MXF files"
echo "From: $SOURCE_MOV"
echo "========================================"
echo ""

rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/temp_essence"

# Step 1: Extract raw DNxHR video
echo "Step 1: Extracting raw DNxHR video essence..."
ffmpeg -i "$SOURCE_MOV" \
    -map 0:v:0 \
    -c:v copy \
    -f rawvideo \
    -y "$OUTPUT_DIR/temp_essence/video.dnxhr" 2>&1 | tail -5

echo ""
echo "✓ Video extracted"
echo ""

# Step 2: Extract all audio tracks as WAV
echo "Step 2: Extracting audio tracks..."
for i in {0..7}; do
    echo "  Extracting audio track $i..."
    ffmpeg -i "$SOURCE_MOV" \
        -map 0:a:$i \
        -c:a pcm_s24le \
        -y "$OUTPUT_DIR/temp_essence/audio_a$i.wav" 2>&1 | tail -2
done

echo ""
echo "✓ All 8 audio tracks extracted"
echo ""

# Step 3: Create Avid OP-Atom MXF with raw2bmx
echo "Step 3: Creating Avid OP-Atom MXF files with raw2bmx..."
cd "$OUTPUT_DIR"

/Users/Editor/Downloads/dnxhr-bmx/bin/raw2bmx \
    -t avid \
    -o BC_030525_A0012 \
    -f 24000/1001 \
    -y 21:15:35:19 \
    --tape "BC_030525_A0012_SOURCE" \
    --width 1920 \
    --height 1080 \
    --signal-std none \
    --color-siting cositing \
    --vc3_dnxhr_lb temp_essence/video.dnxhr \
    --wave temp_essence/audio_a0.wav \
    --wave temp_essence/audio_a1.wav \
    --wave temp_essence/audio_a2.wav \
    --wave temp_essence/audio_a3.wav \
    --wave temp_essence/audio_a4.wav \
    --wave temp_essence/audio_a5.wav \
    --wave temp_essence/audio_a6.wav \
    --wave temp_essence/audio_a7.wav

echo ""
echo "✅ Avid OP-Atom MXF files created!"
echo ""
ls -lh BC_030525_A0012*.mxf
echo ""
echo "✅ Files ready for Avid in: $OUTPUT_DIR"
echo ""
echo "Clean up temp files? (y/n)"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    rm -rf temp_essence
    echo "✓ Temp files removed"
fi

