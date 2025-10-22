#!/bin/bash

# unify_mob_id.sh
# Rewrap multiple MXF files to have the same Material Package UID (MOB ID)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if mxf2raw and bmxtranswrap are available
MXF2RAW="/Users/Editor/Downloads/bmx-ebu/build/apps/mxf2raw/mxf2raw"
BMXTRANSWRAP="/Users/Editor/Downloads/bmx-ebu/build/apps/bmxtranswrap/bmxtranswrap"

if [ ! -f "$MXF2RAW" ] || [ ! -f "$BMXTRANSWRAP" ]; then
    echo -e "${RED}Error: BMX tools not found. Please build bmx first.${NC}"
    exit 1
fi

# Usage
usage() {
    echo "Usage: $0 [OPTIONS] <input_files...>"
    echo ""
    echo "Rewrap multiple MXF files to have the same Material Package UID (MOB ID)"
    echo ""
    echo "Options:"
    echo "  -m, --mob-id <umid>     Use this specific MOB ID (64 hex chars)"
    echo "  -r, --reference <file>  Extract MOB ID from this reference file"
    echo "  -o, --output-dir <dir>  Output directory (default: ./unified_output)"
    echo "  -t, --type <type>       Output type: avid, op1a, as11op1a, etc. (default: avid)"
    echo "  -h, --help              Show this help"
    echo ""
    echo "Examples:"
    echo "  # Use MOB ID from first file for all others"
    echo "  $0 video.mxf audio1.mxf audio2.mxf"
    echo ""
    echo "  # Use MOB ID from specific reference file"
    echo "  $0 -r reference.mxf video.mxf audio1.mxf audio2.mxf"
    echo ""
    echo "  # Use custom MOB ID"
    echo "  $0 -m 060a2b340101010101010f00130000006f88c03c... *.mxf"
    exit 1
}

# Parse arguments
MOB_ID=""
REFERENCE_FILE=""
OUTPUT_DIR="./unified_output"
OUTPUT_TYPE="avid"
INPUT_FILES=()

while [[ $# -gt 0 ]]; do
    case $1 in
        -m|--mob-id)
            MOB_ID="$2"
            shift 2
            ;;
        -r|--reference)
            REFERENCE_FILE="$2"
            shift 2
            ;;
        -o|--output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -t|--type)
            OUTPUT_TYPE="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        -*)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            ;;
        *)
            INPUT_FILES+=("$1")
            shift
            ;;
    esac
done

# Validate input files
if [ ${#INPUT_FILES[@]} -eq 0 ]; then
    echo -e "${RED}Error: No input files specified${NC}"
    usage
fi

# Determine MOB ID
if [ -n "$MOB_ID" ]; then
    # Remove any dots or dashes from provided MOB ID
    MOB_ID=$(echo "$MOB_ID" | tr -d '.-')
    echo -e "${GREEN}Using provided MOB ID: ${MOB_ID}${NC}"
elif [ -n "$REFERENCE_FILE" ]; then
    # Extract from reference file
    if [ ! -f "$REFERENCE_FILE" ]; then
        echo -e "${RED}Error: Reference file not found: $REFERENCE_FILE${NC}"
        exit 1
    fi
    echo -e "${YELLOW}Extracting MOB ID from reference file: $REFERENCE_FILE${NC}"
    MOB_ID=$("$MXF2RAW" -i --avid "$REFERENCE_FILE" 2>/dev/null | grep "Material Package UID" | head -1 | awk '{print $NF}' | tr -d '.-')
else
    # Extract from first input file
    echo -e "${YELLOW}Extracting MOB ID from first input file: ${INPUT_FILES[0]}${NC}"
    MOB_ID=$("$MXF2RAW" -i --avid "${INPUT_FILES[0]}" 2>/dev/null | grep "Material Package UID" | head -1 | awk '{print $NF}' | tr -d '.-')
fi

# Validate MOB ID
if [ -z "$MOB_ID" ]; then
    echo -e "${RED}Error: Could not determine MOB ID${NC}"
    exit 1
fi

# Ensure MOB ID is 64 characters
if [ ${#MOB_ID} -ne 64 ]; then
    echo -e "${RED}Error: MOB ID must be 64 hexadecimal characters, got ${#MOB_ID}${NC}"
    echo -e "${RED}MOB ID: $MOB_ID${NC}"
    exit 1
fi

echo -e "${GREEN}Target MOB ID: $MOB_ID${NC}"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Process each file
SUCCESS_COUNT=0
FAILED_COUNT=0

for input_file in "${INPUT_FILES[@]}"; do
    if [ ! -f "$input_file" ]; then
        echo -e "${RED}Warning: File not found: $input_file${NC}"
        ((FAILED_COUNT++))
        continue
    fi
    
    filename=$(basename "$input_file" .mxf)
    
    # Extract current MOB ID for comparison
    CURRENT_MOB_ID=$("$MXF2RAW" -i --avid "$input_file" 2>/dev/null | grep "Material Package UID" | head -1 | awk '{print $NF}' | tr -d '.-' || echo "")
    
    if [ "$CURRENT_MOB_ID" = "$MOB_ID" ]; then
        echo -e "${YELLOW}File already has target MOB ID, copying: $input_file${NC}"
        cp "$input_file" "$OUTPUT_DIR/${filename}_unified.mxf"
        ((SUCCESS_COUNT++))
    else
        echo -e "${YELLOW}Processing: $input_file${NC}"
        echo -e "  Current MOB ID: $CURRENT_MOB_ID"
        echo -e "  New MOB ID:     $MOB_ID"
        
        # Rewrap with unified MOB ID
        if "$BMXTRANSWRAP" -t "$OUTPUT_TYPE" \
            -o "$OUTPUT_DIR/${filename}_unified" \
            --mp-uid "$MOB_ID" \
            "$input_file" 2>&1 | grep -v "^$"; then
            echo -e "${GREEN}  ✓ Success${NC}"
            ((SUCCESS_COUNT++))
        else
            echo -e "${RED}  ✗ Failed${NC}"
            ((FAILED_COUNT++))
        fi
    fi
    echo ""
done

# Summary
echo "========================================"
echo -e "${GREEN}Successfully processed: $SUCCESS_COUNT${NC}"
if [ $FAILED_COUNT -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED_COUNT${NC}"
fi
echo "Output directory: $OUTPUT_DIR"
echo "Target MOB ID: $MOB_ID"
echo "========================================"

