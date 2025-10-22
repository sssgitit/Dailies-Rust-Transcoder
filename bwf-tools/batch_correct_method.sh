#!/bin/bash

# Batch create 3 files using THE CORRECT METHOD:
# Frame-based TimeReference + 48000 Hz output

set -e

CLIP_DIR="/Volumes/BelleCo_4/00_BELLECO_S4_OCM/BC4001/030525/SOURCE/FX9/BC_A01_03052025/Untitled/XDROOT/Clip"
OUTPUT_DIR="/Users/Editor/Downloads/Transkoder"
FRAME_RATE=23.976
MULTIPLIER=2004.005263
SAMPLE_RATE=48000  # The correct output rate!

echo "================================================================================"
echo "FINAL METHOD: Frame TimeReference + 48000 Hz Output"
echo "================================================================================"
echo ""

# File definitions
declare -a files=(
    "BC_030525_A0001.MXF:13:20:20:05"
    "BC_030525_A0002.MXF:13:26:35:01"
    "BC_030525_A0003.MXF:13:54:32:04"
)

for file_data in "${files[@]}"; do
    IFS=':' read -r filename h m s f <<< "$file_data"
    
    echo "--------------------------------------------------------------------------------"
    echo "Processing: $filename"
    echo "Source Timecode: $h:$m:$s:$f @ ${FRAME_RATE}fps"
    echo "--------------------------------------------------------------------------------"
    
    # Calculate TimeReference using frame-based method
    h_clean=$((10#$h))
    m_clean=$((10#$m))
    s_clean=$((10#$s))
    f_clean=$((10#$f))
    
    total_frames=$(python3 -c "print(($h_clean*60*60*$FRAME_RATE) + ($m_clean*60*$FRAME_RATE) + ($s_clean*$FRAME_RATE) + $f_clean)")
    time_ref=$(python3 -c "print(int($total_frames * $MULTIPLIER))")
    
    echo "Frame Calculation:"
    echo "  Total Frames: $total_frames"
    echo "  TimeReference: $time_ref (frames × $MULTIPLIER)"
    echo "  Output Sample Rate: $SAMPLE_RATE Hz"
    echo ""
    
    # Create output filename
    base_name="${filename%.MXF}"
    output_file="${OUTPUT_DIR}/${base_name}_FINAL.wav"
    
    # Transcode with BEXT
    python3 "${OUTPUT_DIR}/insert_bext_timecode.py" \
        "${CLIP_DIR}/${filename}" \
        "$output_file" \
        --time-ref "$time_ref" \
        --sample-rate $SAMPLE_RATE \
        --frame-rate $FRAME_RATE \
        --description "Frame method: $h:$m:$s:$f @ 48000 Hz" \
        --originator "Transkoder Final" > /dev/null 2>&1
    
    echo "✅ Created: $(basename "$output_file")"
    echo ""
done

echo "================================================================================"
echo "COMPLETE! All files created with:"
echo "  • Frame-based TimeReference calculation"
echo "  • 48000 Hz output"
echo "================================================================================"

