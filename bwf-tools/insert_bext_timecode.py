#!/usr/bin/env python3
"""
Insert BEXT chunk with specific TimeReference into WAV file
"""

import struct
import sys
import subprocess
from pathlib import Path
from datetime import datetime

def create_bext_chunk(time_reference, 
                      description="",
                      originator="Transkoder",
                      originator_ref="",
                      origination_date=None,
                      origination_time=None):
    """
    Create a BEXT chunk with specified TimeReference
    """
    if origination_date is None:
        origination_date = datetime.now().strftime("%Y-%m-%d")
    if origination_time is None:
        origination_time = datetime.now().strftime("%H:%M:%S")
    
    # Build BEXT chunk (minimum 602 bytes)
    bext = bytearray(602)
    
    # Description (256 bytes)
    desc_bytes = description.encode('ascii')[:256]
    bext[0:len(desc_bytes)] = desc_bytes
    
    # Originator (32 bytes)
    orig_bytes = originator.encode('ascii')[:32]
    bext[256:256+len(orig_bytes)] = orig_bytes
    
    # OriginatorReference (32 bytes)
    orig_ref_bytes = originator_ref.encode('ascii')[:32]
    bext[288:288+len(orig_ref_bytes)] = orig_ref_bytes
    
    # OriginationDate (10 bytes, YYYY-MM-DD)
    date_bytes = origination_date.encode('ascii')[:10]
    bext[320:320+len(date_bytes)] = date_bytes
    
    # OriginationTime (8 bytes, HH:MM:SS)
    time_bytes = origination_time.encode('ascii')[:8]
    bext[330:330+len(time_bytes)] = time_bytes
    
    # TimeReference (8 bytes, 64-bit unsigned int, little-endian) - THE KEY VALUE!
    struct.pack_into('<Q', bext, 338, time_reference)
    
    # Version (2 bytes)
    struct.pack_into('<H', bext, 346, 1)  # Version 1
    
    # UMID (64 bytes) - all zeros
    # Reserved (180 bytes) - all zeros
    # Already zeroed in bytearray
    
    return bytes(bext)

def transcode_with_bext(input_file, output_file, time_reference, sample_rate=48048, 
                        description="", originator="Transkoder", frame_rate=25):
    """
    Transcode audio file to BWF with BEXT chunk using ffmpeg
    """
    print(f"Transcoding: {input_file} -> {output_file}")
    print(f"Sample Rate: {sample_rate} Hz")
    print(f"TimeReference: {time_reference:,} samples")
    print()
    
    # Calculate expected timecode for verification
    total_seconds = time_reference / sample_rate
    hours = int(total_seconds // 3600)
    remaining = total_seconds % 3600
    minutes = int(remaining // 60)
    seconds = remaining % 60
    seconds_int = int(seconds)
    frames = int((seconds % 1) * frame_rate)
    expected_tc = f"{hours:02d}:{minutes:02d}:{seconds_int:02d}:{frames:02d}"
    
    print(f"Expected Timecode (@{frame_rate}fps): {expected_tc}")
    print()
    
    # Create temporary WAV file with ffmpeg
    temp_wav = output_file + ".temp.wav"
    
    # Transcode to WAV at specified sample rate
    cmd = [
        'ffmpeg',
        '-i', str(input_file),
        '-ar', str(sample_rate),  # Set sample rate
        '-ac', '2',  # Stereo (adjust as needed)
        '-c:a', 'pcm_s24le',  # 24-bit PCM
        '-y',  # Overwrite
        temp_wav
    ]
    
    print("Step 1: Transcoding to WAV...")
    print(f"Command: {' '.join(cmd)}")
    print()
    
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode != 0:
        print(f"❌ FFmpeg failed:")
        print(result.stderr)
        return False
    
    print("✓ Transcode complete")
    print()
    
    # Now insert BEXT chunk
    print("Step 2: Inserting BEXT chunk...")
    
    try:
        with open(temp_wav, 'rb') as f_in:
            # Read original WAV
            riff = f_in.read(4)
            if riff != b'RIFF':
                print("❌ Not a valid WAV file")
                return False
            
            file_size = struct.unpack('<I', f_in.read(4))[0]
            wave = f_in.read(4)
            
            # Create BEXT chunk
            bext_data = create_bext_chunk(
                time_reference,
                description=description,
                originator=originator
            )
            
            # Write new WAV with BEXT
            with open(output_file, 'wb') as f_out:
                # RIFF header (will update size later)
                f_out.write(b'RIFF')
                f_out.write(struct.pack('<I', 0))  # Placeholder
                f_out.write(b'WAVE')
                
                # Write BEXT chunk first
                f_out.write(b'bext')
                f_out.write(struct.pack('<I', len(bext_data)))
                f_out.write(bext_data)
                
                # Copy remaining chunks from original
                total_size = 4  # 'WAVE'
                total_size += 8 + len(bext_data)  # 'bext' chunk
                
                while True:
                    chunk_id = f_in.read(4)
                    if len(chunk_id) < 4:
                        break
                    
                    chunk_size_data = f_in.read(4)
                    if len(chunk_size_data) < 4:
                        break
                        
                    chunk_size = struct.unpack('<I', chunk_size_data)[0]
                    chunk_data = f_in.read(chunk_size)
                    
                    # Write chunk
                    f_out.write(chunk_id)
                    f_out.write(chunk_size_data)
                    f_out.write(chunk_data)
                    
                    total_size += 8 + chunk_size
                    
                    # Align to even boundary
                    if chunk_size % 2:
                        padding = f_in.read(1)
                        if padding:
                            f_out.write(padding)
                            total_size += 1
                
                # Update RIFF size
                f_out.seek(4)
                f_out.write(struct.pack('<I', total_size))
        
        # Remove temp file
        Path(temp_wav).unlink()
        
        print(f"✓ BEXT chunk inserted")
        print(f"✓ Output file: {output_file}")
        print()
        
        return True
        
    except Exception as e:
        print(f"❌ Error inserting BEXT: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == '__main__':
    import argparse
    
    parser = argparse.ArgumentParser(description='Transcode audio and insert BEXT chunk with TimeReference')
    parser.add_argument('input', help='Input audio file')
    parser.add_argument('output', help='Output BWF file')
    parser.add_argument('-t', '--time-ref', type=int, required=True, 
                       help='TimeReference value (samples since midnight)')
    parser.add_argument('-s', '--sample-rate', type=int, default=48048,
                       help='Sample rate (default: 48048 Hz)')
    parser.add_argument('-d', '--description', default='',
                       help='BEXT description')
    parser.add_argument('-o', '--originator', default='Transkoder',
                       help='BEXT originator')
    parser.add_argument('-f', '--frame-rate', type=float, default=25,
                       help='Frame rate for timecode display (default: 25)')
    
    args = parser.parse_args()
    
    if not Path(args.input).exists():
        print(f"❌ Input file not found: {args.input}")
        sys.exit(1)
    
    success = transcode_with_bext(
        args.input,
        args.output,
        args.time_ref,
        args.sample_rate,
        args.description,
        args.originator,
        args.frame_rate
    )
    
    if success:
        print()
        print("=" * 80)
        print("✅ SUCCESS! Now let's verify...")
        print()
        
        # Verify the result
        import subprocess
        result = subprocess.run([
            sys.executable,
            'test_bwf_timecode.py',
            args.output,
            '-f', str(args.frame_rate)
        ], capture_output=False)
        
        sys.exit(0)
    else:
        sys.exit(1)

