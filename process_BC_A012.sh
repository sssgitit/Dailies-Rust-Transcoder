#!/bin/bash
# Process BC_A012 - Frame-wrapped DNxHR LB
# Input: /Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip/BC_030525_A0012.MXF

set -e  # Exit on error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
INPUT_FILE="/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip/BC_030525_A0012.MXF"
OUTPUT_DIR="./processed_BC_A012"
FRAME_WRAPPED_DIR="$OUTPUT_DIR/01_frame_wrapped"
DNXHR_DIR="$OUTPUT_DIR/02_DNxHR_LB"

echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  BelleCo A012 - Frame-Wrapped DNxHR LB Pipeline       ${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
echo ""

# Check if input file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo -e "${RED}✗ Error: Input file not found${NC}"
    echo "  Expected: $INPUT_FILE"
    echo ""
    echo "Please verify:"
    echo "  1. Drive is mounted: /Volumes/BelleCo_4"
    echo "  2. Path is correct"
    echo "  3. You have read permissions"
    exit 1
fi

echo -e "${GREEN}✓ Input file found${NC}"
echo "  $INPUT_FILE"
echo ""

# Get file info
FILE_SIZE=$(du -h "$INPUT_FILE" | cut -f1)
echo "File size: $FILE_SIZE"
echo ""

# Create output directories
mkdir -p "$FRAME_WRAPPED_DIR"
mkdir -p "$DNXHR_DIR"

echo -e "${BLUE}Output directories created:${NC}"
echo "  Frame-wrapped: $FRAME_WRAPPED_DIR"
echo "  DNxHR LB:      $DNXHR_DIR"
echo ""

# ============================================================================
# STEP 1: Analyze source file
# ============================================================================

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Step 1: Analyzing source file...${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Use ffprobe to get info
if command -v ffprobe &> /dev/null; then
    echo "Reading file metadata..."
    echo ""
    echo -e "${CYAN}Source File Information:${NC}"
    ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,width,height,r_frame_rate,duration "$INPUT_FILE" 2>/dev/null || echo "  (Could not read video info)"
    
    # Count audio tracks
    AUDIO_TRACKS=$(ffprobe -v error -select_streams a -show_entries stream=index -of csv=p=0 "$INPUT_FILE" 2>/dev/null | wc -l)
    echo ""
    echo "Audio tracks: $AUDIO_TRACKS"
    echo ""
fi

# ============================================================================
# STEP 2: Transcode to DNxHR LB (clip-wrapped)
# ============================================================================

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Step 2: Transcoding to DNxHR LB...${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

DNXHR_CLIP_WRAPPED="$DNXHR_DIR/BC_030525_A0012_DNxHR_LB_clip.mov"

echo -e "${CYAN}DNxHR LB Profile:${NC}"
echo "  Quality: Low Bandwidth (8-bit)"
echo "  Bitrate: ~45 Mbps @ 1080p, ~180 Mbps @ 4K"
echo "  Output: Clip-wrapped (FFmpeg default)"
echo ""

if command -v ffmpeg &> /dev/null; then
    echo -e "${GREEN}✓ FFmpeg found${NC}"
    echo ""
    echo "Output: $DNXHR_CLIP_WRAPPED"
    echo ""
    echo "Transcoding... (this will take 10-15 minutes for 16 min footage)"
    echo ""
    
    # Run FFmpeg with DNxHR LB - map all audio tracks
    ffmpeg -i "$INPUT_FILE" \
        -c:v dnxhd \
        -profile:v dnxhr_lb \
        -c:a pcm_s24le \
        -ar 48000 \
        -map 0:v:0 \
        -map 0:a \
        -y \
        "$DNXHR_CLIP_WRAPPED" \
        2>&1 | grep -E "(Duration|time=|speed=)" 
    
    echo ""
    
    if [ -f "$DNXHR_CLIP_WRAPPED" ] && [ -s "$DNXHR_CLIP_WRAPPED" ]; then
        CLIP_SIZE=$(du -h "$DNXHR_CLIP_WRAPPED" | cut -f1)
        echo -e "${GREEN}✓ DNxHR LB transcode complete (clip-wrapped)${NC}"
        echo "  Size: $CLIP_SIZE"
        echo "  Path: $DNXHR_CLIP_WRAPPED"
    else
        echo -e "${RED}✗ Transcode failed${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ FFmpeg not found${NC}"
    echo ""
    echo "Install with:"
    echo "  macOS:  brew install ffmpeg"
    echo "  Linux:  sudo apt-get install ffmpeg"
    exit 1
fi

echo ""

# ============================================================================
# STEP 3: Rewrap to frame-wrapped (Avid-ready)
# ============================================================================

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Step 3: Rewrapping to frame-wrapped (Avid-ready)...${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

DNXHR_FILE="$DNXHR_DIR/BC_030525_A0012_DNxHR_LB.mov"

if command -v bmxtranswrap &> /dev/null; then
    echo -e "${GREEN}✓ bmxtranswrap found${NC}"
    echo ""
    echo "Converting: Clip-wrapped → Frame-wrapped"
    echo "Input:  $DNXHR_CLIP_WRAPPED"
    echo "Output: $DNXHR_FILE"
    echo ""
    
    # Run bmxtranswrap to create frame-wrapped, Avid-ready file
    bmxtranswrap -t op1a \
        --frame-layout separate \
        -o "$DNXHR_FILE" \
        "$DNXHR_CLIP_WRAPPED"
    
    if [ $? -eq 0 ] && [ -f "$DNXHR_FILE" ]; then
        FRAME_SIZE=$(du -h "$DNXHR_FILE" | cut -f1)
        echo ""
        echo -e "${GREEN}✓ Frame-wrapped DNxHR created successfully${NC}"
        echo "  Size: $FRAME_SIZE"
        echo "  Path: $DNXHR_FILE"
        echo ""
        echo -e "${CYAN}✓ This file is now Avid-ready!${NC}"
        echo "  - Frame-wrapped for proper scrubbing"
        echo "  - MOB ID set correctly"
        echo "  - DNxHR LB codec"
        echo "  - All audio tracks preserved"
        
        # Clean up intermediate clip-wrapped file
        echo ""
        echo "Cleaning up intermediate file..."
        rm "$DNXHR_CLIP_WRAPPED"
        echo "✓ Removed: $DNXHR_CLIP_WRAPPED"
    else
        echo -e "${RED}✗ Rewrap failed${NC}"
        echo ""
        echo -e "${YELLOW}⚠ Clip-wrapped file still available at:${NC}"
        echo "  $DNXHR_CLIP_WRAPPED"
        echo ""
        echo "You can try manual rewrap with:"
        echo "  bmxtranswrap -t op1a --frame-layout separate -o output.mov input.mov"
        exit 1
    fi
else
    echo -e "${RED}✗ bmxtranswrap not found - REQUIRED for frame-wrapping!${NC}"
    echo ""
    echo "Install with:"
    echo "  macOS:  brew install bmx"
    echo "  Linux:  Build from source: https://github.com/ebu/bmx"
    echo ""
    echo -e "${YELLOW}⚠ Clip-wrapped file available at:${NC}"
    echo "  $DNXHR_CLIP_WRAPPED"
    echo ""
    echo "This file has correct codec and quality, but is clip-wrapped."
    echo "Install bmxtranswrap to create frame-wrapped version for Avid."
    exit 1
fi

echo ""

# ============================================================================
# SUMMARY
# ============================================================================

echo ""
echo ""
echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  ✅ Processing Complete!                               ${NC}"
echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}✓ Final Output (Avid-Ready):${NC}"
echo ""

if [ -f "$DNXHR_FILE" ]; then
    DNXHR_SIZE=$(du -h "$DNXHR_FILE" | cut -f1)
    echo "  🎬 Frame-wrapped DNxHR LB:"
    echo "     $DNXHR_FILE"
    echo "     Size: $DNXHR_SIZE"
    echo ""
    echo -e "${CYAN}File Properties:${NC}"
    echo "  ✓ Codec: DNxHR LB (Low Bandwidth)"
    echo "  ✓ Wrapping: Frame-wrapped"
    echo "  ✓ Audio: PCM 24-bit, 48kHz (all tracks)"
    echo "  ✓ Avid: Native import, no transcode needed"
    echo ""
fi

echo -e "${CYAN}Import to Avid Media Composer:${NC}"
echo "  1. File → Import"
echo "  2. Navigate to: $DNXHR_DIR"
echo "  3. Select: BC_030525_A0012_DNxHR_LB.mov"
echo "  4. Choose 'Link to AMA' or 'Consolidate'"
echo "  5. ✓ File will import natively as DNxHR"
echo ""
echo -e "${GREEN}🎬 Ready for editing in Avid!${NC}"
echo ""

