#!/usr/bin/env python3
"""
Frame-Based BEXT TimeReference Calculator
Uses the exact method discovered by the user:
- Frame rate: exactly 23.976 (not 24000/1001)
- Multiplier: 2004.005263 samples/frame
- Works with truncation at ~48048 Hz
"""

def calculate_timereference_frame_based(hours, minutes, seconds, frames, 
                                       frame_rate=23.976, 
                                       samples_per_frame=2004.005263):
    """
    Calculate TimeReference using frame-based method
    
    Args:
        hours: Timecode hours
        minutes: Timecode minutes  
        seconds: Timecode seconds
        frames: Timecode frames
        frame_rate: Frame rate (default 23.976 exactly)
        samples_per_frame: Calibrated multiplier (default 2004.005263)
    
    Returns:
        TimeReference in samples
    """
    # Calculate total frames
    total_frames = (hours * 60 * 60 * frame_rate) + \
                   (minutes * 60 * frame_rate) + \
                   (seconds * frame_rate) + \
                   frames
    
    # Calculate TimeReference
    time_ref = total_frames * samples_per_frame
    
    return int(time_ref)

def verify_timereference(time_ref, sample_rate=48048, frame_rate=23.976):
    """
    Verify a TimeReference by decoding it back to timecode
    Uses TRUNCATION method
    """
    total_seconds = time_ref / sample_rate
    
    h = int(total_seconds // 3600)
    remaining = total_seconds % 3600
    m = int(remaining // 60)
    s_total = remaining % 60
    s = int(s_total)
    f = int((s_total % 1) * frame_rate)  # TRUNCATE
    
    return f"{h:02d}:{m:02d}:{s:02d}:{f:02d}"

if __name__ == '__main__':
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Frame-based BEXT TimeReference Calculator',
        epilog='This uses the exact method: frames × 2004.005263 = TimeReference'
    )
    parser.add_argument('timecode', help='Timecode in HH:MM:SS:FF format')
    parser.add_argument('-f', '--frame-rate', type=float, default=23.976,
                       help='Frame rate (default: 23.976)')
    parser.add_argument('-m', '--multiplier', type=float, default=2004.005263,
                       help='Samples per frame multiplier (default: 2004.005263)')
    parser.add_argument('-s', '--sample-rate', type=int, default=48048,
                       help='Sample rate for verification (default: 48048)')
    parser.add_argument('-v', '--verify', action='store_true',
                       help='Verify by decoding back to timecode')
    
    args = parser.parse_args()
    
    # Parse timecode
    try:
        parts = args.timecode.replace(';', ':').split(':')
        if len(parts) != 4:
            raise ValueError
        h, m, s, f = map(int, parts)
    except:
        print(f"Error: Invalid timecode format. Use HH:MM:SS:FF")
        exit(1)
    
    print(f"Frame-Based BEXT Calculator")
    print("=" * 70)
    print(f"Input Timecode: {h:02d}:{m:02d}:{s:02d}:{f:02d} @ {args.frame_rate}fps")
    print()
    
    # Calculate using frame-based method
    print("Calculation:")
    print("-" * 70)
    total_frames = (h * 60 * 60 * args.frame_rate) + \
                   (m * 60 * args.frame_rate) + \
                   (s * args.frame_rate) + f
    
    print(f"Total Frames = ({h}×60×60×{args.frame_rate}) + "
          f"({m}×60×{args.frame_rate}) + ({s}×{args.frame_rate}) + {f}")
    print(f"             = {total_frames:.10f} frames")
    print()
    
    time_ref = calculate_timereference_frame_based(
        h, m, s, f, 
        args.frame_rate, 
        args.multiplier
    )
    
    print(f"TimeReference = {total_frames:.10f} × {args.multiplier}")
    print(f"              = {time_ref:,} samples")
    print()
    
    # Verify
    if args.verify:
        print("Verification:")
        print("-" * 70)
        verified_tc = verify_timereference(time_ref, args.sample_rate, args.frame_rate)
        print(f"At {args.sample_rate} Hz with truncation:")
        print(f"  {time_ref:,} samples → {verified_tc}")
        
        if verified_tc == f"{h:02d}:{m:02d}:{s:02d}:{f:02d}":
            print("  ✅ MATCH!")
        else:
            print(f"  ⚠️  Expected: {h:02d}:{m:02d}:{s:02d}:{f:02d}")
            print(f"  ⚠️  Got:      {verified_tc}")
    
    print()
    print("=" * 70)
    print(f"BEXT TimeReference: {time_ref}")
    print("=" * 70)

