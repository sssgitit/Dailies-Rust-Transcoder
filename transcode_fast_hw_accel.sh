#!/bin/bash
# FAST XAVC-I to DNxHR LB with Hardware Acceleration
# Optimized for Apple Silicon (M1 Max) using VideoToolbox

set -e

INPUT="${1:-/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip/BC_030525_A0012.MXF}"
OUTPUT_DIR="${2:-/Users/Editor/Downloads/AVID_READY}"

# Extract filename without extension
BASENAME=$(basename "$INPUT" .MXF)
OUTPUT_FILE="$OUTPUT_DIR/${BASENAME}_DNxHR_LB.mov"

echo "========================================"
echo "⚡ FAST Hardware-Accelerated Transcode"
echo "========================================"
echo ""
echo "Input:  $INPUT"
echo "Output: $OUTPUT_FILE"
echo ""

# Check input exists
if [ ! -f "$INPUT" ]; then
    echo "❌ Error: Input file not found: $INPUT"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Detect codec
CODEC=$(ffprobe -v error -select_streams v:0 -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$INPUT")
echo "📹 Detected codec: $CODEC"
echo ""

# Use VideoToolbox hardware acceleration (works for H.264 and HEVC)
echo "🚀 Using VideoToolbox hardware acceleration for $CODEC"
echo "⏱️  Starting transcode (HW accelerated)..."
echo ""

# Hardware-accelerated transcode
# VideoToolbox decoding (GPU) → DNxHR encoding (CPU multi-threaded)
ffmpeg \
  -hwaccel videotoolbox \
  -i "$INPUT" \
  -c:v dnxhd \
  -profile:v dnxhr_lb \
  -pix_fmt yuv422p \
  -threads 0 \
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
echo "📊 File Properties:"
ffprobe -v error -select_streams v:0 \
  -show_entries stream=codec_name,width,height,r_frame_rate,bit_rate \
  -of default=noprint_wrappers=1 "$OUTPUT_FILE" | \
  sed 's/codec_name=/  Codec: /' | \
  sed 's/width=/  Width: /' | \
  sed 's/height=/  Height: /' | \
  sed 's/r_frame_rate=/  Frame Rate: /' | \
  sed 's/bit_rate=/  Bitrate: /'

AUDIO_TRACKS=$(ffprobe -v error -select_streams a -show_entries stream=index -of csv=p=0 "$OUTPUT_FILE" | wc -l | tr -d ' ')
echo "  Audio Tracks: $AUDIO_TRACKS (PCM 24-bit 48kHz)"
echo ""
echo "✅ Ready for Avid Media Composer!"
echo ""

