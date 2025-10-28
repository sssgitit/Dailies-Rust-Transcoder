#!/bin/bash
# Quick version - Frame-wrapped DNxHR LB for BC_A012
# Minimal output, just the essentials

INPUT="/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip/BC_030525_A0012.MXF"
OUTPUT_DIR="./processed_BC_A012"

mkdir -p "$OUTPUT_DIR"/{01_frame_wrapped,02_DNxHR_LB}

echo "🎬 Processing BC_030525_A0012..."
echo ""

# Check file exists
if [ ! -f "$INPUT" ]; then
    echo "❌ File not found: $INPUT"
    exit 1
fi

echo "✓ Input file found ($(du -h "$INPUT" | cut -f1))"

# Step 1: Rewrap
echo "🔄 Rewrapping to frame-wrapped..."
FRAME_OUT="$OUTPUT_DIR/01_frame_wrapped/BC_030525_A0012_frame.mxf"

if command -v bmxtranswrap &> /dev/null; then
    bmxtranswrap -t op1a --frame-layout separate -o "$FRAME_OUT" "$INPUT" 2>&1 | tail -5
    [ -f "$FRAME_OUT" ] && echo "✓ Frame-wrapped: $FRAME_OUT" || echo "✗ Rewrap failed"
    INPUT_FOR_ENCODE="$FRAME_OUT"
else
    echo "⚠ bmxtranswrap not found, using original"
    INPUT_FOR_ENCODE="$INPUT"
fi

# Step 2: Transcode to DNxHR LB
echo ""
echo "🎬 Transcoding to DNxHR LB..."
DNXHR_OUT="$OUTPUT_DIR/02_DNxHR_LB/BC_030525_A0012_DNxHR_LB.mov"

if command -v ffmpeg &> /dev/null; then
    ffmpeg -i "$INPUT_FOR_ENCODE" \
        -c:v dnxhd -profile:v dnxhr_lb \
        -c:a pcm_s24le -ar 48000 \
        -y "$DNXHR_OUT" \
        2>&1 | grep "time=" | tail -1
    
    echo ""
    if [ -f "$DNXHR_OUT" ]; then
        echo "✅ DONE: $DNXHR_OUT ($(du -h "$DNXHR_OUT" | cut -f1))"
    else
        echo "❌ Transcode failed"
        exit 1
    fi
else
    echo "❌ FFmpeg not found"
    exit 1
fi

echo ""
echo "Ready for Avid! 🎉"

