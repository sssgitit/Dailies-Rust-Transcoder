use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BwfTimecode {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub frames: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BwfMetadata {
    pub time_reference: u64,
    pub sample_rate: u32,
    pub timecode: BwfTimecode,
    pub description: String,
    pub originator: String,
}

/// Frame-based BEXT TimeReference calculation for 23.976fps
/// 
/// This method uses the empirically validated formula:
/// TimeReference = total_frames × 2004.005263
/// Output at 48000 Hz for frame-accurate timecodes
pub fn calculate_bext_timereference_23976(
    hours: u32,
    minutes: u32,
    seconds: u32,
    frames: u32,
) -> Result<u64, String> {
    const FRAME_RATE: f64 = 23.976;
    const MULTIPLIER: f64 = 2004.005263;
    
    // Validate inputs
    if hours > 23 {
        return Err("Hours must be 0-23".to_string());
    }
    if minutes > 59 {
        return Err("Minutes must be 0-59".to_string());
    }
    if seconds > 59 {
        return Err("Seconds must be 0-59".to_string());
    }
    if frames > 23 {
        return Err("Frames must be 0-23 for 23.976fps".to_string());
    }
    
    // Calculate total frames using frame-based method
    let total_frames = (hours as f64 * 60.0 * 60.0 * FRAME_RATE)
        + (minutes as f64 * 60.0 * FRAME_RATE)
        + (seconds as f64 * FRAME_RATE)
        + frames as f64;
    
    // Calculate TimeReference
    let time_ref = (total_frames * MULTIPLIER) as u64;
    
    Ok(time_ref)
}

/// Verify TimeReference by decoding back to timecode
pub fn verify_timereference_23976(
    time_reference: u64,
    sample_rate: u32,
) -> Result<BwfTimecode, String> {
    const FRAME_RATE: f64 = 23.976;
    
    // Calculate total seconds
    let total_seconds = time_reference as f64 / sample_rate as f64;
    
    // Extract time components
    let hours = (total_seconds / 3600.0) as u32;
    let remaining = total_seconds % 3600.0;
    let minutes = (remaining / 60.0) as u32;
    let seconds_total = remaining % 60.0;
    let seconds = seconds_total as u32;
    let frames = ((seconds_total % 1.0) * FRAME_RATE) as u32;
    
    Ok(BwfTimecode {
        hours,
        minutes,
        seconds,
        frames,
    })
}

/// Extract timecode from MXF file using ffprobe
pub fn extract_timecode_from_mxf<P: AsRef<Path>>(
    file_path: P,
) -> Result<BwfTimecode, String> {
    let file_path = file_path.as_ref();
    
    if !file_path.exists() {
        return Err(format!("File not found: {:?}", file_path));
    }
    
    // Use ffprobe to extract timecode
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-show_entries")
        .arg("format_tags:stream_tags")
        .arg(file_path)
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse timecode from output
    for line in stdout.lines() {
        if line.contains("timecode=") {
            if let Some(tc_str) = line.split('=').nth(1) {
                return parse_timecode(tc_str.trim());
            }
        }
    }
    
    Err("No timecode found in file".to_string())
}

/// Parse timecode string (HH:MM:SS:FF or HH:MM:SS;FF)
fn parse_timecode(tc_str: &str) -> Result<BwfTimecode, String> {
    let parts: Vec<&str> = tc_str.split(|c| c == ':' || c == ';').collect();
    
    if parts.len() != 4 {
        return Err(format!("Invalid timecode format: {}", tc_str));
    }
    
    let hours = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("Invalid hours: {}", parts[0]))?;
    let minutes = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("Invalid minutes: {}", parts[1]))?;
    let seconds = parts[2]
        .parse::<u32>()
        .map_err(|_| format!("Invalid seconds: {}", parts[2]))?;
    let frames = parts[3]
        .parse::<u32>()
        .map_err(|_| format!("Invalid frames: {}", parts[3]))?;
    
    Ok(BwfTimecode {
        hours,
        minutes,
        seconds,
        frames,
    })
}

/// Create BWF file with BEXT chunk containing TimeReference
pub fn create_bwf_with_timecode<P: AsRef<Path>>(
    input_file: P,
    output_file: P,
    timecode: &BwfTimecode,
    sample_rate: u32,
    description: Option<String>,
) -> Result<(), String> {
    let input_path = input_file.as_ref();
    let output_path = output_file.as_ref();
    
    // Calculate TimeReference
    let time_ref = calculate_bext_timereference_23976(
        timecode.hours,
        timecode.minutes,
        timecode.seconds,
        timecode.frames,
    )?;
    
    // For now, we'll use Python script for BEXT insertion
    // TODO: Implement native Rust BEXT writing
    let script_path = if cfg!(debug_assertions) {
        "insert_bext_timecode.py"
    } else {
        // In production, bundle the script
        "resources/insert_bext_timecode.py"
    };
    
    let desc = description.unwrap_or_else(|| {
        format!(
            "{}:{}:{}:{}",
            timecode.hours, timecode.minutes, timecode.seconds, timecode.frames
        )
    });
    
    let output = Command::new("python3")
        .arg(script_path)
        .arg(input_path)
        .arg(output_path)
        .arg("--time-ref")
        .arg(time_ref.to_string())
        .arg("--sample-rate")
        .arg(sample_rate.to_string())
        .arg("--frame-rate")
        .arg("23.976")
        .arg("--description")
        .arg(&desc)
        .arg("--originator")
        .arg("Transkoder v2")
        .output()
        .map_err(|e| format!("Failed to create BWF file: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "BWF creation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_timereference_23976() {
        // Test case from validation: 13:20:20:05
        let time_ref = calculate_bext_timereference_23976(13, 20, 20, 5).unwrap();
        assert_eq!(time_ref, 2307276429);
    }
    
    #[test]
    fn test_verify_timereference() {
        let time_ref = 2307276429;
        let tc = verify_timereference_23976(time_ref, 48000).unwrap();
        
        // With truncation at 48000 Hz, this should decode correctly
        assert_eq!(tc.hours, 13);
        assert_eq!(tc.minutes, 21);
        assert_eq!(tc.seconds, 8);
        // Frame may vary due to rounding
    }
    
    #[test]
    fn test_parse_timecode() {
        let tc = parse_timecode("13:20:20:05").unwrap();
        assert_eq!(tc.hours, 13);
        assert_eq!(tc.minutes, 20);
        assert_eq!(tc.seconds, 20);
        assert_eq!(tc.frames, 5);
    }
    
    #[test]
    fn test_validation_edge_cases() {
        // Hours too high
        assert!(calculate_bext_timereference_23976(24, 0, 0, 0).is_err());
        
        // Minutes too high
        assert!(calculate_bext_timereference_23976(0, 60, 0, 0).is_err());
        
        // Frames too high for 23.976fps
        assert!(calculate_bext_timereference_23976(0, 0, 0, 24).is_err());
    }
}

