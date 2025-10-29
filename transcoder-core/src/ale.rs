//! Avid Log Exchange (ALE) file generation
//! Creates ALE files for importing media into Avid Media Composer

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// ALE file generator
pub struct AleGenerator {
    entries: Vec<AleEntry>,
}

/// Single entry in an ALE file
#[derive(Debug, Clone)]
pub struct AleEntry {
    pub name: String,
    pub tape: String,
    pub start_tc: String,
    pub end_tc: String,
    pub duration: String,
    pub fps: String,
    pub audio_tracks: u32,
    pub video_tracks: u32,
}

impl AleGenerator {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry from a media file
    pub fn add_from_file(&mut self, file_path: &Path) -> Result<(), String> {
        let entry = Self::extract_metadata(file_path)?;
        self.entries.push(entry);
        Ok(())
    }

    /// Extract metadata from a media file using ffprobe
    fn extract_metadata(file_path: &Path) -> Result<AleEntry, String> {
        // Get file name without extension
        let name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Extract tape name (usually from filename, e.g., "A001" from "BC_030525_A0012")
        let tape = name
            .split('_')
            .find(|s| s.starts_with('A') || s.starts_with('B') || s.starts_with('C'))
            .unwrap_or("A001")
            .to_string();

        // Get timecode using ffprobe
        let start_tc = Self::get_timecode(file_path)?;

        // Get duration and calculate end timecode
        let (duration, end_tc) = Self::get_duration_and_end_tc(file_path, &start_tc)?;

        // Get frame rate
        let fps = Self::get_frame_rate(file_path)?;

        // Get audio track count
        let audio_tracks = Self::get_audio_track_count(file_path)?;

        Ok(AleEntry {
            name,
            tape,
            start_tc,
            end_tc,
            duration,
            fps,
            audio_tracks,
            video_tracks: 1, // Assume 1 video track
        })
    }

    /// Extract timecode from file
    fn get_timecode(file_path: &Path) -> Result<String, String> {
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-select_streams", "v:0",
                "-show_entries", "format_tags=timecode:stream_tags=timecode",
                "-of", "default=noprint_wrappers=1:nokey=1",
                file_path.to_str().ok_or("Invalid path")?,
            ])
            .output()
            .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

        let tc = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        if tc.is_empty() {
            Ok("00:00:00:00".to_string())
        } else {
            Ok(tc)
        }
    }

    /// Get duration and calculate end timecode
    fn get_duration_and_end_tc(file_path: &Path, start_tc: &str) -> Result<(String, String), String> {
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-show_entries", "format=duration",
                "-of", "default=noprint_wrappers=1:nokey=1",
                file_path.to_str().ok_or("Invalid path")?,
            ])
            .output()
            .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

        let duration_secs: f64 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(0.0);

        // Convert duration to timecode format (HH:MM:SS:FF at 23.976fps)
        let total_frames = (duration_secs * 23.976) as i64;
        let duration_tc = Self::frames_to_timecode(total_frames, 23.976);

        // Calculate end timecode (simplified - assumes drop-frame not needed)
        let start_frames = Self::timecode_to_frames(start_tc, 23.976);
        let end_frames = start_frames + total_frames;
        let end_tc = Self::frames_to_timecode(end_frames, 23.976);

        Ok((duration_tc, end_tc))
    }

    /// Get frame rate from file
    fn get_frame_rate(file_path: &Path) -> Result<String, String> {
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-select_streams", "v:0",
                "-show_entries", "stream=r_frame_rate",
                "-of", "default=noprint_wrappers=1:nokey=1",
                file_path.to_str().ok_or("Invalid path")?,
            ])
            .output()
            .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

        let fps_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Parse fraction (e.g., "24000/1001" -> "23.976")
        if let Some((num, den)) = fps_str.split_once('/') {
            if let (Ok(n), Ok(d)) = (num.parse::<f64>(), den.parse::<f64>()) {
                return Ok(format!("{:.3}", n / d));
            }
        }

        Ok("23.976".to_string()) // Default
    }

    /// Get audio track count
    fn get_audio_track_count(file_path: &Path) -> Result<u32, String> {
        let output = Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-select_streams", "a",
                "-show_entries", "stream=index",
                "-of", "csv=p=0",
                file_path.to_str().ok_or("Invalid path")?,
            ])
            .output()
            .map_err(|e| format!("Failed to run ffprobe: {}", e))?;

        let count = String::from_utf8_lossy(&output.stdout)
            .lines()
            .count() as u32;

        Ok(count)
    }

    /// Convert frames to timecode string
    fn frames_to_timecode(total_frames: i64, fps: f64) -> String {
        let fps_int = fps.round() as i64;
        let frames = total_frames % fps_int;
        let seconds = (total_frames / fps_int) % 60;
        let minutes = (total_frames / fps_int / 60) % 60;
        let hours = total_frames / fps_int / 3600;

        format!("{:02}:{:02}:{:02}:{:02}", hours, minutes, seconds, frames)
    }

    /// Convert timecode string to frames
    fn timecode_to_frames(tc: &str, fps: f64) -> i64 {
        let parts: Vec<&str> = tc.split(':').collect();
        if parts.len() != 4 {
            return 0;
        }

        let hours: i64 = parts[0].parse().unwrap_or(0);
        let minutes: i64 = parts[1].parse().unwrap_or(0);
        let seconds: i64 = parts[2].parse().unwrap_or(0);
        let frames: i64 = parts[3].parse().unwrap_or(0);

        let fps_int = fps.round() as i64;
        (hours * 3600 + minutes * 60 + seconds) * fps_int + frames
    }

    /// Write ALE file
    pub fn write_to_file(&self, output_path: &Path) -> Result<(), String> {
        let mut file = File::create(output_path)
            .map_err(|e| format!("Failed to create ALE file: {}", e))?;

        // Write header
        writeln!(file, "Heading").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "FIELD_DELIM\tTABS").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "VIDEO_FORMAT\t1080p").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "AUDIO_FORMAT\t48kHz").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "FPS\t23.976").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;

        // Write column headers
        writeln!(file, "Column").map_err(|e| format!("Write error: {}", e))?;
        writeln!(file, "Name\tTape\tStart\tEnd\tDuration\tTracks\tFPS")
            .map_err(|e| format!("Write error: {}", e))?;
        writeln!(file).map_err(|e| format!("Write error: {}", e))?;

        // Write data
        writeln!(file, "Data").map_err(|e| format!("Write error: {}", e))?;
        for entry in &self.entries {
            writeln!(
                file,
                "{}\t{}\t{}\t{}\t{}\t{}A\t{}",
                entry.name,
                entry.tape,
                entry.start_tc,
                entry.end_tc,
                entry.duration,
                entry.audio_tracks,
                entry.fps
            )
            .map_err(|e| format!("Write error: {}", e))?;
        }

        Ok(())
    }
}

impl Default for AleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

