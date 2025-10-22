#!/usr/bin/env python3
"""
BWF BEXT Timecode Calculator and Tester
Tests theory about calculating timecode from BEXT TimeReference
"""

import struct
import sys
from pathlib import Path

def read_bext_chunk(wav_file):
    """
    Read BEXT chunk from BWF (Broadcast Wave Format) file
    """
    with open(wav_file, 'rb') as f:
        # Read RIFF header
        riff = f.read(4)
        if riff != b'RIFF':
            raise ValueError("Not a valid WAV file")
        
        file_size = struct.unpack('<I', f.read(4))[0]
        wave = f.read(4)
        if wave != b'WAVE':
            raise ValueError("Not a valid WAV file")
        
        # Find BEXT and fmt chunks
        bext_data = None
        sample_rate = None
        
        while f.tell() < file_size:
            try:
                chunk_id = f.read(4)
                if len(chunk_id) < 4:
                    break
                    
                chunk_size = struct.unpack('<I', f.read(4))[0]
                
                if chunk_id == b'bext':
                    # Read BEXT chunk
                    bext_data = f.read(chunk_size)
                elif chunk_id == b'fmt ':
                    # Read format chunk to get sample rate
                    fmt_data = f.read(chunk_size)
                    sample_rate = struct.unpack('<I', fmt_data[4:8])[0]
                else:
                    # Skip other chunks
                    f.seek(chunk_size, 1)
                    
                # Align to even boundary
                if chunk_size % 2:
                    f.read(1)
                    
            except Exception as e:
                print(f"Error reading chunk: {e}")
                break
        
        return bext_data, sample_rate

def parse_bext(bext_data):
    """
    Parse BEXT chunk data
    BEXT structure:
    - Description: 256 bytes (ASCII)
    - Originator: 32 bytes (ASCII)
    - OriginatorReference: 32 bytes (ASCII)
    - OriginationDate: 10 bytes (ASCII, YYYY-MM-DD)
    - OriginationTime: 8 bytes (ASCII, HH:MM:SS)
    - TimeReference: 8 bytes (QWORD, little-endian)
    - Version: 2 bytes (WORD)
    - UMID: 64 bytes
    - LoudnessValue: 2 bytes (WORD)
    - LoudnessRange: 2 bytes (WORD)
    - MaxTruePeakLevel: 2 bytes (WORD)
    - MaxMomentaryLoudness: 2 bytes (WORD)
    - MaxShortTermLoudness: 2 bytes (WORD)
    - Reserved: 180 bytes
    - CodingHistory: variable length (ASCII)
    """
    if not bext_data or len(bext_data) < 602:
        raise ValueError("Invalid BEXT data")
    
    description = bext_data[0:256].decode('ascii', errors='ignore').rstrip('\x00')
    originator = bext_data[256:288].decode('ascii', errors='ignore').rstrip('\x00')
    originator_ref = bext_data[288:320].decode('ascii', errors='ignore').rstrip('\x00')
    origination_date = bext_data[320:330].decode('ascii', errors='ignore').rstrip('\x00')
    origination_time = bext_data[330:338].decode('ascii', errors='ignore').rstrip('\x00')
    
    # TimeReference is at byte 338, 8 bytes (64-bit unsigned integer, little-endian)
    time_reference = struct.unpack('<Q', bext_data[338:346])[0]
    
    version = struct.unpack('<H', bext_data[346:348])[0]
    umid = bext_data[348:412].hex()
    
    # Optional fields (version >= 1)
    loudness_value = None
    if len(bext_data) >= 414:
        loudness_value = struct.unpack('<H', bext_data[412:414])[0]
    
    # Coding history starts at byte 602
    coding_history = ''
    if len(bext_data) > 602:
        coding_history = bext_data[602:].decode('ascii', errors='ignore').rstrip('\x00')
    
    return {
        'description': description,
        'originator': originator,
        'originator_reference': originator_ref,
        'origination_date': origination_date,
        'origination_time': origination_time,
        'time_reference': time_reference,
        'version': version,
        'umid': umid,
        'loudness_value': loudness_value,
        'coding_history': coding_history,
    }

def calculate_timecode(time_reference, sample_rate, frame_rate=25, drop_frame=False):
    """
    Calculate timecode from TimeReference and sample rate
    
    Args:
        time_reference: Sample count since midnight
        sample_rate: Audio sample rate (e.g., 48000)
        frame_rate: Video frame rate for timecode (default 25)
        drop_frame: Drop frame timecode (default False)
    
    Returns:
        Dictionary with various timecode representations
    """
    # Calculate total seconds since midnight
    total_seconds = time_reference / sample_rate
    
    # Calculate hours, minutes, seconds
    hours = int(total_seconds // 3600)
    remaining = total_seconds % 3600
    minutes = int(remaining // 60)
    seconds = remaining % 60
    
    # Calculate frames
    frames = int((seconds % 1) * frame_rate)
    seconds_int = int(seconds)
    
    # Format timecode
    drop_char = ';' if drop_frame else ':'
    timecode = f"{hours:02d}:{minutes:02d}:{seconds_int:02d}{drop_char}{frames:02d}"
    
    return {
        'time_reference': time_reference,
        'sample_rate': sample_rate,
        'total_seconds': total_seconds,
        'hours': hours,
        'minutes': minutes,
        'seconds': seconds,
        'frames': frames,
        'timecode': timecode,
        'frame_rate': frame_rate,
    }

def test_theory(wav_file, expected_timecode=None, frame_rate=25):
    """
    Test the BWF timecode calculation theory
    """
    print(f"Testing BWF file: {wav_file}")
    print("=" * 80)
    
    try:
        # Read BEXT chunk
        bext_data, sample_rate = read_bext_chunk(wav_file)
        
        if not bext_data:
            print("❌ No BEXT chunk found in file")
            return False
        
        if not sample_rate:
            print("❌ Could not determine sample rate")
            return False
        
        print(f"✓ Found BEXT chunk ({len(bext_data)} bytes)")
        print(f"✓ Sample Rate: {sample_rate} Hz")
        print()
        
        # Parse BEXT
        bext_info = parse_bext(bext_data)
        
        print("BEXT Metadata:")
        print("-" * 80)
        print(f"Description: {bext_info['description']}")
        print(f"Originator: {bext_info['originator']}")
        print(f"Originator Reference: {bext_info['originator_reference']}")
        print(f"Origination Date: {bext_info['origination_date']}")
        print(f"Origination Time: {bext_info['origination_time']}")
        print(f"TimeReference: {bext_info['time_reference']:,} samples")
        print(f"Version: {bext_info['version']}")
        if bext_info['umid']:
            print(f"UMID: {bext_info['umid'][:32]}...")
        print()
        
        # Calculate timecode
        tc_info = calculate_timecode(
            bext_info['time_reference'],
            sample_rate,
            frame_rate
        )
        
        print("Calculated Timecode:")
        print("-" * 80)
        print(f"Total Seconds from Midnight: {tc_info['total_seconds']:.6f}")
        print(f"Timecode (@{frame_rate}fps): {tc_info['timecode']}")
        print(f"  Hours: {tc_info['hours']:02d}")
        print(f"  Minutes: {tc_info['minutes']:02d}")
        print(f"  Seconds: {int(tc_info['seconds']):02d}")
        print(f"  Frames: {tc_info['frames']:02d}")
        print()
        
        # Test against expected
        if expected_timecode:
            print(f"Expected Timecode: {expected_timecode}")
            print(f"Calculated Timecode: {tc_info['timecode']}")
            
            if tc_info['timecode'] == expected_timecode:
                print("✅ MATCH! Theory is correct!")
                return True
            else:
                print("❌ MISMATCH! Theory needs adjustment")
                print()
                print("Debugging Info:")
                print(f"  TimeReference: {bext_info['time_reference']}")
                print(f"  Sample Rate: {sample_rate}")
                print(f"  Calculation: {bext_info['time_reference']} / {sample_rate} = {tc_info['total_seconds']:.6f} seconds")
                return False
        else:
            print("✅ Calculation complete (no expected timecode to compare)")
            return True
            
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        return False

def manual_test(time_reference, sample_rate, expected_timecode, frame_rate=25):
    """
    Manually test a theory with specific values
    """
    print(f"Manual Test")
    print("=" * 80)
    print(f"Time Reference: {time_reference:,} samples")
    print(f"Sample Rate: {sample_rate:,} Hz")
    print(f"Frame Rate: {frame_rate} fps")
    print(f"Expected Timecode: {expected_timecode}")
    print()
    
    tc_info = calculate_timecode(time_reference, sample_rate, frame_rate)
    
    print("Calculated:")
    print(f"  Total Seconds: {tc_info['total_seconds']:.6f}")
    print(f"  Timecode: {tc_info['timecode']}")
    print()
    
    if tc_info['timecode'] == expected_timecode:
        print("✅ MATCH! Theory is correct!")
        return True
    else:
        print("❌ MISMATCH!")
        print(f"  Expected:   {expected_timecode}")
        print(f"  Calculated: {tc_info['timecode']}")
        return False

if __name__ == '__main__':
    import argparse
    
    parser = argparse.ArgumentParser(description='Test BWF BEXT timecode calculation theory')
    parser.add_argument('file', nargs='?', help='BWF file to test')
    parser.add_argument('-e', '--expected', help='Expected timecode (HH:MM:SS:FF)')
    parser.add_argument('-f', '--frame-rate', type=float, default=25, help='Frame rate (default: 25)')
    parser.add_argument('-m', '--manual', action='store_true', help='Manual test mode')
    parser.add_argument('-t', '--time-ref', type=int, help='Time reference (samples)')
    parser.add_argument('-s', '--sample-rate', type=int, help='Sample rate (Hz)')
    
    args = parser.parse_args()
    
    if args.manual:
        if not args.time_ref or not args.sample_rate or not args.expected:
            print("Manual mode requires: --time-ref, --sample-rate, and --expected")
            sys.exit(1)
        
        success = manual_test(args.time_ref, args.sample_rate, args.expected, args.frame_rate)
        sys.exit(0 if success else 1)
    
    if not args.file:
        print("Error: Please provide a BWF file or use --manual mode")
        parser.print_help()
        sys.exit(1)
    
    if not Path(args.file).exists():
        print(f"Error: File not found: {args.file}")
        sys.exit(1)
    
    success = test_theory(args.file, args.expected, args.frame_rate)
    sys.exit(0 if success else 1)

