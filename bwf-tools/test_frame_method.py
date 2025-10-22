#!/usr/bin/env python3
"""
Comprehensive test of the frame-based BEXT calculation method
"""

import subprocess
import sys
from pathlib import Path

def calculate_frame_based(h, m, s, f, frame_rate=23.976, samples_per_frame=2004.005263):
    """Frame-based calculation"""
    total_frames = (h * 60 * 60 * frame_rate) + (m * 60 * frame_rate) + (s * frame_rate) + f
    time_ref = int(total_frames * samples_per_frame)
    return time_ref

def calculate_standard(h, m, s, f, frame_rate=23.976, sample_rate=48000):
    """Standard time-based calculation"""
    total_seconds = h*3600 + m*60 + s + (f / frame_rate)
    time_ref = int(total_seconds * sample_rate)
    return time_ref

def verify_truncate(time_ref, sample_rate=48048, frame_rate=23.976):
    """Decode TimeReference using truncation"""
    total_seconds = time_ref / sample_rate
    h = int(total_seconds // 3600)
    remaining = total_seconds % 3600
    m = int(remaining // 60)
    s_total = remaining % 60
    s = int(s_total)
    f = int((s_total % 1) * frame_rate)
    return (h, m, s, f)

def verify_round(time_ref, sample_rate=48000, frame_rate=23.976):
    """Decode TimeReference using rounding"""
    total_seconds = time_ref / sample_rate
    h = int(total_seconds // 3600)
    remaining = total_seconds % 3600
    m = int(remaining // 60)
    s_total = remaining % 60
    s = int(s_total)
    f = round((s_total % 1) * frame_rate)
    return (h, m, s, f)

def test_timecode(h, m, s, f, frame_rate=23.976):
    """Test a single timecode"""
    original = (h, m, s, f)
    
    # Calculate using frame method
    time_ref_frame = calculate_frame_based(h, m, s, f, frame_rate)
    
    # Calculate using standard method at 48000 Hz
    time_ref_std_48k = calculate_standard(h, m, s, f, frame_rate, 48000)
    
    # Calculate using standard method at 48048 Hz
    time_ref_std_48048 = calculate_standard(h, m, s, f, frame_rate, 48048)
    
    # Verify frame method with truncation at 48048 Hz
    decoded_frame = verify_truncate(time_ref_frame, 48048, frame_rate)
    
    # Verify standard 48k with rounding
    decoded_std_48k = verify_round(time_ref_std_48k, 48000, frame_rate)
    
    # Verify standard 48048 with truncation
    decoded_std_48048 = verify_truncate(time_ref_std_48048, 48048, frame_rate)
    
    return {
        'original': original,
        'frame_method': {
            'time_ref': time_ref_frame,
            'decoded': decoded_frame,
            'match': decoded_frame == original
        },
        'standard_48k': {
            'time_ref': time_ref_std_48k,
            'decoded': decoded_std_48k,
            'match': decoded_std_48k == original
        },
        'standard_48048': {
            'time_ref': time_ref_std_48048,
            'decoded': decoded_std_48048,
            'match': decoded_std_48048 == original
        }
    }

def format_tc(tc_tuple):
    """Format timecode tuple as string"""
    return f"{tc_tuple[0]:02d}:{tc_tuple[1]:02d}:{tc_tuple[2]:02d}:{tc_tuple[3]:02d}"

def run_tests():
    """Run comprehensive tests"""
    
    test_cases = [
        # Original test case
        (13, 20, 20, 5),
        
        # Hour boundaries
        (0, 0, 0, 0),      # Midnight
        (1, 0, 0, 0),      # 1 hour
        (23, 59, 59, 23),  # Near midnight
        
        # Various timecodes
        (10, 30, 15, 10),
        (5, 45, 30, 15),
        (12, 0, 0, 12),
        (18, 25, 45, 20),
        
        # Frame boundaries
        (1, 2, 3, 0),      # Frame 0
        (1, 2, 3, 23),     # Frame 23 (last frame)
        (1, 2, 3, 1),      # Frame 1
        
        # Edge cases
        (0, 0, 1, 0),      # 1 second
        (0, 1, 0, 0),      # 1 minute
        (0, 0, 0, 1),      # 1 frame
    ]
    
    print("=" * 80)
    print("FRAME-BASED BEXT METHOD - COMPREHENSIVE TEST")
    print("=" * 80)
    print()
    
    results = {
        'frame_method': {'pass': 0, 'fail': 0},
        'standard_48k': {'pass': 0, 'fail': 0},
        'standard_48048': {'pass': 0, 'fail': 0}
    }
    
    failures = []
    
    for i, (h, m, s, f) in enumerate(test_cases, 1):
        result = test_timecode(h, m, s, f)
        original_tc = format_tc(result['original'])
        
        print(f"Test {i:2d}: {original_tc}")
        print("-" * 80)
        
        # Frame method
        fm = result['frame_method']
        status = "✅ PASS" if fm['match'] else "❌ FAIL"
        print(f"  Frame Method (48048 Hz, truncate):  {fm['time_ref']:,}")
        print(f"    Decoded: {format_tc(fm['decoded'])} {status}")
        
        if fm['match']:
            results['frame_method']['pass'] += 1
        else:
            results['frame_method']['fail'] += 1
            failures.append({
                'test': i,
                'timecode': original_tc,
                'method': 'Frame Method',
                'decoded': format_tc(fm['decoded'])
            })
        
        # Standard 48k
        s48k = result['standard_48k']
        status = "✅ PASS" if s48k['match'] else "❌ FAIL"
        print(f"  Standard Method (48000 Hz, round):  {s48k['time_ref']:,}")
        print(f"    Decoded: {format_tc(s48k['decoded'])} {status}")
        
        if s48k['match']:
            results['standard_48k']['pass'] += 1
        else:
            results['standard_48k']['fail'] += 1
        
        # Standard 48048
        s48048 = result['standard_48048']
        status = "✅ PASS" if s48048['match'] else "❌ FAIL"
        print(f"  Standard Method (48048 Hz, truncate): {s48048['time_ref']:,}")
        print(f"    Decoded: {format_tc(s48048['decoded'])} {status}")
        
        if s48048['match']:
            results['standard_48048']['pass'] += 1
        else:
            results['standard_48048']['fail'] += 1
        
        print()
    
    # Summary
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print()
    
    total_tests = len(test_cases)
    
    print(f"Frame Method (48048 Hz, truncate):")
    print(f"  ✅ Passed: {results['frame_method']['pass']}/{total_tests}")
    print(f"  ❌ Failed: {results['frame_method']['fail']}/{total_tests}")
    print()
    
    print(f"Standard Method (48000 Hz, round):")
    print(f"  ✅ Passed: {results['standard_48k']['pass']}/{total_tests}")
    print(f"  ❌ Failed: {results['standard_48k']['fail']}/{total_tests}")
    print()
    
    print(f"Standard Method (48048 Hz, truncate):")
    print(f"  ✅ Passed: {results['standard_48048']['pass']}/{total_tests}")
    print(f"  ❌ Failed: {results['standard_48048']['fail']}/{total_tests}")
    print()
    
    if failures:
        print("=" * 80)
        print("FAILURES DETAIL")
        print("=" * 80)
        for fail in failures:
            print(f"Test {fail['test']}: {fail['timecode']}")
            print(f"  Method: {fail['method']}")
            print(f"  Decoded as: {fail['decoded']}")
            print()
    
    # Conclusion
    print("=" * 80)
    print("CONCLUSION")
    print("=" * 80)
    
    if results['frame_method']['fail'] == 0:
        print("✅ FRAME METHOD IS CONSISTENT!")
        print("   Works perfectly for all tested timecodes.")
        return True
    else:
        print("⚠️  FRAME METHOD HAS ISSUES")
        print(f"   Failed {results['frame_method']['fail']} out of {total_tests} tests.")
        return False

if __name__ == '__main__':
    success = run_tests()
    sys.exit(0 if success else 1)

