#!/bin/bash
# PARALLEL BATCH TRANSCODE with Hardware Acceleration
# Processes multiple files simultaneously using all available cores

set -e

# Configuration
INPUT_DIR="${1:-/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip}"
OUTPUT_DIR="${2:-/Users/Editor/Downloads/AVID_READY_BATCH}"
MAX_JOBS=11  # 12 cores - 1 for system

echo "========================================"
echo "⚡ PARALLEL BATCH TRANSCODE"
echo "========================================"
echo ""
echo "Input:  $INPUT_DIR"
echo "Output: $OUTPUT_DIR"
echo "Max Parallel Jobs: $MAX_JOBS"
echo ""

# Check input directory exists
if [ ! -d "$INPUT_DIR" ]; then
    echo "❌ Error: Input directory not found: $INPUT_DIR"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/logs"

# Count MXF files
FILE_COUNT=$(find "$INPUT_DIR" -maxdepth 1 -name "*.MXF" -o -name "*.mxf" | wc -l | tr -d ' ')
echo "📁 Found $FILE_COUNT MXF files to transcode"
echo ""

if [ "$FILE_COUNT" -eq 0 ]; then
    echo "⚠️  No MXF files found in $INPUT_DIR"
    exit 0
fi

# Function to transcode a single file
transcode_file() {
    local INPUT_FILE="$1"
    local OUTPUT_DIR="$2"
    local BASENAME=$(basename "$INPUT_FILE" .MXF)
    local OUTPUT_FILE="$OUTPUT_DIR/${BASENAME}_DNxHR_LB.mov"
    local LOG_FILE="$OUTPUT_DIR/logs/${BASENAME}.log"
    
    echo "🎬 Starting: $BASENAME" | tee -a "$LOG_FILE"
    
    # Transcode with VideoToolbox hardware acceleration
    # VideoToolbox decoding (GPU) → DNxHR encoding (CPU multi-threaded)
    ffmpeg \
      -hwaccel videotoolbox \
      -i "$INPUT_FILE" \
      -c:v dnxhd \
      -profile:v dnxhr_lb \
      -pix_fmt yuv422p \
      -threads 0 \
      -c:a pcm_s24le \
      -ar 48000 \
      -map 0:v:0 \
      -map 0:a \
      -y "$OUTPUT_FILE" \
      >> "$LOG_FILE" 2>&1
    
    if [ $? -eq 0 ]; then
        FILE_SIZE=$(ls -lh "$OUTPUT_FILE" | awk '{print $5}')
        echo "✅ Complete: $BASENAME ($FILE_SIZE)" | tee -a "$LOG_FILE"
        return 0
    else
        echo "❌ Failed: $BASENAME (see $LOG_FILE)" | tee -a "$LOG_FILE"
        return 1
    fi
}

export -f transcode_file

# Start time
START_TIME=$(date +%s)

echo "🚀 Starting parallel transcode with $MAX_JOBS workers..."
echo ""

# Use GNU parallel or xargs for parallel processing
if command -v parallel &> /dev/null; then
    # GNU parallel (best option)
    find "$INPUT_DIR" -maxdepth 1 \( -name "*.MXF" -o -name "*.mxf" \) | \
        parallel -j $MAX_JOBS transcode_file {} "$OUTPUT_DIR"
elif command -v xargs &> /dev/null; then
    # xargs fallback (good)
    find "$INPUT_DIR" -maxdepth 1 \( -name "*.MXF" -o -name "*.mxf" \) | \
        xargs -P $MAX_JOBS -I {} bash -c "transcode_file '{}' '$OUTPUT_DIR'"
else
    # Sequential fallback (slow)
    echo "⚠️  No parallel tool found. Processing sequentially..."
    for file in "$INPUT_DIR"/*.MXF "$INPUT_DIR"/*.mxf; do
        [ -f "$file" ] && transcode_file "$file" "$OUTPUT_DIR"
    done
fi

# End time
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
MINUTES=$((ELAPSED / 60))
SECONDS=$((ELAPSED % 60))

echo ""
echo "========================================"
echo "✅ BATCH COMPLETE!"
echo "========================================"
echo ""
echo "⏱️  Total time: ${MINUTES}m ${SECONDS}s"
echo "📁 Output directory: $OUTPUT_DIR"
echo ""

# Summary
SUCCESS_COUNT=$(find "$OUTPUT_DIR" -name "*_DNxHR_LB.mov" | wc -l | tr -d ' ')
echo "📊 Summary:"
echo "  Total files: $FILE_COUNT"
echo "  Successful: $SUCCESS_COUNT"
echo "  Failed: $((FILE_COUNT - SUCCESS_COUNT))"
echo ""

if [ $SUCCESS_COUNT -gt 0 ]; then
    echo "✅ Files ready for Avid:"
    ls -lh "$OUTPUT_DIR"/*.mov
fi

echo ""
echo "📋 Logs available in: $OUTPUT_DIR/logs/"

