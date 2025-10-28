#!/bin/bash
# Interactive Transcode Menu - Use NOW while GUI builds!

set -e

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${CYAN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║   ⚡ SUPER FAST TRANSCODER - M2 Max Edition ⚡   ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Hardware Acceleration: VideoToolbox (6.73x realtime!)${NC}"
echo -e "${GREEN}Parallel Jobs: 11 simultaneous transcodes${NC}"
echo ""

# Get system info
CORES=$(sysctl -n hw.ncpu)
WORKERS=$((CORES - 1))

echo "System: M2 Max with $CORES cores"
echo "Workers: $WORKERS parallel jobs"
echo ""
echo "═══════════════════════════════════════════════════"
echo ""

# Default paths
DEFAULT_INPUT="/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip"
DEFAULT_OUTPUT="/Users/Editor/Downloads/AVID_READY"

# Menu
echo "SELECT TRANSCODE MODE:"
echo ""
echo "  1) Single File (Test/Preview)"
echo "  2) Batch Process Entire Folder (Recommended)"
echo "  3) Custom Input/Output Paths"
echo "  4) Exit"
echo ""
read -p "Choice [1-4]: " choice

case $choice in
    1)
        echo ""
        echo -e "${YELLOW}═══ SINGLE FILE MODE ═══${NC}"
        echo ""
        echo "Default file: BC_030525_A0012.MXF"
        read -p "Use default? (y/n): " use_default
        
        if [[ "$use_default" =~ ^[Yy]$ ]]; then
            INPUT="$DEFAULT_INPUT/BC_030525_A0012.MXF"
        else
            read -p "Enter full path to MXF file: " INPUT
        fi
        
        read -p "Output directory [$DEFAULT_OUTPUT]: " OUTPUT
        OUTPUT=${OUTPUT:-$DEFAULT_OUTPUT}
        
        echo ""
        echo -e "${GREEN}Starting transcode...${NC}"
        echo "Input:  $INPUT"
        echo "Output: $OUTPUT"
        echo ""
        
        ./transcode_fast_hw_accel.sh "$INPUT" "$OUTPUT"
        ;;
        
    2)
        echo ""
        echo -e "${YELLOW}═══ BATCH MODE - $WORKERS PARALLEL JOBS ═══${NC}"
        echo ""
        read -p "Input directory [$DEFAULT_INPUT]: " INPUT_DIR
        INPUT_DIR=${INPUT_DIR:-$DEFAULT_INPUT}
        
        read -p "Output directory [$DEFAULT_OUTPUT]: " OUTPUT_DIR
        OUTPUT_DIR=${OUTPUT_DIR:-$DEFAULT_OUTPUT}
        
        # Count files
        FILE_COUNT=$(find "$INPUT_DIR" -maxdepth 1 \( -name "*.MXF" -o -name "*.mxf" \) 2>/dev/null | wc -l | tr -d ' ')
        
        echo ""
        echo -e "${GREEN}Found $FILE_COUNT MXF files${NC}"
        echo "Processing with $WORKERS parallel workers"
        echo ""
        
        read -p "Start batch transcode? (y/n): " confirm
        
        if [[ "$confirm" =~ ^[Yy]$ ]]; then
            echo ""
            echo -e "${GREEN}🚀 Starting batch transcode...${NC}"
            echo ""
            ./batch_transcode_parallel.sh "$INPUT_DIR" "$OUTPUT_DIR"
        else
            echo "Cancelled."
        fi
        ;;
        
    3)
        echo ""
        echo -e "${YELLOW}═══ CUSTOM PATHS ═══${NC}"
        echo ""
        read -p "Mode (single/batch): " mode
        read -p "Input path: " INPUT_PATH
        read -p "Output directory: " OUTPUT_PATH
        
        if [[ "$mode" == "single" ]]; then
            ./transcode_fast_hw_accel.sh "$INPUT_PATH" "$OUTPUT_PATH"
        else
            ./batch_transcode_parallel.sh "$INPUT_PATH" "$OUTPUT_PATH"
        fi
        ;;
        
    4)
        echo "Goodbye!"
        exit 0
        ;;
        
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✅ TRANSCODE COMPLETE! ✅              ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════╝${NC}"
echo ""
echo "📁 Output location: $OUTPUT_DIR"
echo "🎬 Ready to import into Avid Media Composer!"
echo ""

