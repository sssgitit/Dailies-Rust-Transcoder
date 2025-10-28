#!/bin/bash
# Simple One-Step Transcode to Avid-Ready DNxHR LB MOV
# No MXF wrapping issues - just a clean QuickTime file!

set -e

INPUT="/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip/BC_030525_A0012.MXF"
OUTPUT_DIR="/Users/Editor/Downloads/AVID_READY"
OUTPUT_FILE="$OUTPUT_DIR/BC_030525_A0012_DNxHR_LB.mov"

echo "========================================"
echo "Transcoding to Avid-Ready DNxHR LB MOV"
echo "========================================"
echo ""
echo "Input:  $INPUT"
echo "Output: $OUTPUT_FILE"
echo ""

# Check input exists
if [ ! -f "$INPUT" ]; then
    echo "❌ Error: Input file not found"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Single-step transcode: Source → DNxHR LB MOV
echo "Transcoding... (this will take 10-15 minutes)"
echo ""

ffmpeg -i "$INPUT" \
  -c:v dnxhd \
  -profile:v dnxhr_lb \
  -pix_fmt yuv422p \
  -c:a pcm_s24le \
  -ar 48000 \
  -map 0:v:0 \
  -map 0:a \
  -y "$OUTPUT_FILE"

echo ""
echo "========================================"
echo "✅ Transcode Complete!"
echo "========================================"
echo ""
ls -lh "$OUTPUT_FILE"
echo ""
echo "File Properties:"
ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,width,height,r_frame_rate \
  -of default=noprint_wrappers=1:nokey=1 "$OUTPUT_FILE" | \
  awk 'NR==1{print "  Codec: " $0} NR==2{print "  Resolution: " $0 "x"} NR==3{print $0} NR==4{print "  Frame Rate: " $0}'

AUDIO_TRACKS=$(ffprobe -v error -select_streams a -show_entries stream=index -of csv=p=0 "$OUTPUT_FILE" | wc -l)
echo "  Audio Tracks: $AUDIO_TRACKS (PCM 24-bit 48kHz)"
echo ""
echo "📝 Import to Avid:"
echo "  1. File → Import"
echo "  2. Select: $OUTPUT_FILE"
echo "  3. Choose 'Link to AMA' (recommended)"
echo ""
echo "✅ Ready for Avid Media Composer!"
echo ""

