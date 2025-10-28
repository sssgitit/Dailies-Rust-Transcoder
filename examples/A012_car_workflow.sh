#!/bin/bash
# A012 Car Clip - Complete Workflow Example
# This demonstrates a full production pipeline:
# 1. Detect MXF wrapping
# 2. Convert clip-wrapped to frame-wrapped (if needed)
# 3. Transcode to ProRes HQ for editing
# 4. Create proxy for offline editing

set -e  # Exit on error

# Configuration
INPUT_FILE="A012_car.mxf"
OUTPUT_DIR="./processed"
FRAME_WRAPPED_DIR="$OUTPUT_DIR/01_frame_wrapped"
PRORES_DIR="$OUTPUT_DIR/02_prores"
PROXY_DIR="$OUTPUT_DIR/03_proxy"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if input file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo -e "${RED}Error: Input file '$INPUT_FILE' not found${NC}"
    echo ""
    echo "Please place your A012_car.mxf file in this directory"
    echo "Or update INPUT_FILE in this script"
    exit 1
fi

# Create output directories
mkdir -p "$FRAME_WRAPPED_DIR"
mkdir -p "$PRORES_DIR"
mkdir -p "$PROXY_DIR"

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  A012 Car Clip - Full Pipeline ${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Step 1: Detect MXF wrapping type
echo -e "${YELLOW}Step 1: Detecting MXF wrapping type...${NC}"
echo "Input: $INPUT_FILE"
echo ""

# Using ffprobe to check if it's clip-wrapped or frame-wrapped
if command -v ffprobe &> /dev/null; then
    echo "File information:"
    ffprobe -v quiet -show_format -show_streams "$INPUT_FILE" | grep -E "(codec_name|duration|bit_rate|nb_frames)" || true
    echo ""
fi

# Step 2: Check if rewrapping is needed
echo -e "${YELLOW}Step 2: Checking if rewrap to frame-wrapped is needed...${NC}"

# Try to detect wrapping with bmxtranswrap
if command -v bmxtranswrap &> /dev/null; then
    echo "✓ bmxtranswrap found"
    
    # Rewrap to frame-wrapped
    FRAME_WRAPPED_FILE="$FRAME_WRAPPED_DIR/A012_car_frame.mxf"
    
    echo "Converting to frame-wrapped for editing..."
    echo "Output: $FRAME_WRAPPED_FILE"
    
    bmxtranswrap -t op1a \
        --frame-layout separate \
        -o "$FRAME_WRAPPED_FILE" \
        "$INPUT_FILE"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Successfully converted to frame-wrapped${NC}"
        INPUT_FOR_TRANSCODE="$FRAME_WRAPPED_FILE"
    else
        echo -e "${RED}✗ Rewrap failed, using original file${NC}"
        INPUT_FOR_TRANSCODE="$INPUT_FILE"
    fi
else
    echo -e "${YELLOW}⚠ bmxtranswrap not found - skipping rewrap${NC}"
    echo "Install with: brew install bmx (macOS)"
    INPUT_FOR_TRANSCODE="$INPUT_FILE"
fi
echo ""

# Step 3: Transcode to ProRes HQ
echo -e "${YELLOW}Step 3: Transcoding to ProRes HQ for editing...${NC}"
PRORES_FILE="$PRORES_DIR/A012_car_ProResHQ.mov"
echo "Output: $PRORES_FILE"

if command -v ffmpeg &> /dev/null; then
    ffmpeg -i "$INPUT_FOR_TRANSCODE" \
        -c:v prores_ks \
        -profile:v 3 \
        -c:a pcm_s24le \
        -ar 48000 \
        -y \
        "$PRORES_FILE" \
        -progress pipe:1 | grep -oP 'out_time_ms=\K\d+' | \
        while read -r time_ms; do
            # Show progress (simplified)
            printf "\rProgress: %d ms processed..." "$time_ms"
        done
    
    echo ""
    if [ -f "$PRORES_FILE" ]; then
        echo -e "${GREEN}✓ ProRes HQ transcode complete${NC}"
        
        # Show file size
        SIZE=$(du -h "$PRORES_FILE" | cut -f1)
        echo "File size: $SIZE"
    else
        echo -e "${RED}✗ Transcode failed${NC}"
    fi
else
    echo -e "${RED}✗ ffmpeg not found - cannot transcode${NC}"
    echo "Install with: brew install ffmpeg (macOS)"
fi
echo ""

# Step 4: Create proxy for offline editing
echo -e "${YELLOW}Step 4: Creating ProRes LT proxy for offline editing...${NC}"
PROXY_FILE="$PROXY_DIR/A012_car_ProxyLT.mov"
echo "Output: $PROXY_FILE"

if command -v ffmpeg &> /dev/null; then
    ffmpeg -i "$INPUT_FOR_TRANSCODE" \
        -c:v prores_ks \
        -profile:v 1 \
        -s 1280x720 \
        -c:a pcm_s16le \
        -ar 48000 \
        -y \
        "$PROXY_FILE" 2>&1 | tail -n 10
    
    if [ -f "$PROXY_FILE" ]; then
        echo -e "${GREEN}✓ Proxy created successfully${NC}"
        
        # Show file size comparison
        SIZE=$(du -h "$PROXY_FILE" | cut -f1)
        echo "File size: $SIZE (much smaller than master)"
    fi
else
    echo -e "${RED}✗ ffmpeg not found - cannot create proxy${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  Pipeline Complete!            ${NC}"
echo -e "${BLUE}================================${NC}"
echo ""
echo "Output files:"
echo ""

if [ -f "$FRAME_WRAPPED_FILE" ]; then
    echo -e "${GREEN}✓${NC} Frame-wrapped MXF: $FRAME_WRAPPED_FILE"
fi

if [ -f "$PRORES_FILE" ]; then
    echo -e "${GREEN}✓${NC} ProRes HQ (editing): $PRORES_FILE"
fi

if [ -f "$PROXY_FILE" ]; then
    echo -e "${GREEN}✓${NC} ProRes LT (proxy): $PROXY_FILE"
fi

echo ""
echo "Next steps:"
echo "1. Import ProRes HQ file into your editing system"
echo "2. Use ProRes LT proxy for faster offline editing"
echo "3. Relink to ProRes HQ for final output"
echo ""

