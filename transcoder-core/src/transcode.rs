//! FFmpeg-based transcoding engine

use crate::config::TranscodeConfig;
use crate::error::{TranscodeError, TranscodeResult};
use crate::platform;
use regex::Regex;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Transcoder for executing FFmpeg jobs
pub struct Transcoder {
    ffmpeg_path: std::path::PathBuf,
}

impl Transcoder {
    /// Create a new transcoder instance
    pub fn new() -> TranscodeResult<Self> {
        let ffmpeg_path = platform::find_ffmpeg()?;
        info!("FFmpeg found at: {:?}", ffmpeg_path);
        Ok(Self { ffmpeg_path })
    }

    /// Transcode a file with progress callback
    pub async fn transcode<F>(
        &self,
        input: &Path,
        output: &Path,
        config: &TranscodeConfig,
        mut progress_callback: F,
    ) -> TranscodeResult<()>
    where
        F: FnMut(f32, Option<f32>) + Send,
    {
        // Validate input
        if !input.exists() {
            return Err(TranscodeError::InvalidInput(format!(
                "Input file not found: {:?}",
                input
            )));
        }

        // Get input duration for progress calculation
        let duration = self.get_duration(input).await?;
        debug!("Input duration: {:.2}s", duration);

        // Build FFmpeg command
        let args = config.to_ffmpeg_args(
            &input.to_string_lossy(),
            &output.to_string_lossy(),
        );

        debug!("FFmpeg command: {:?} {:?}", self.ffmpeg_path, args);

        // Execute FFmpeg with progress parsing
        self.execute_ffmpeg(&args, duration, &mut progress_callback)
            .await?;

        // Verify output was created
        if !output.exists() {
            return Err(TranscodeError::FfmpegFailed(
                "Output file was not created".to_string(),
            ));
        }

        info!("Transcoding completed: {:?}", output);
        Ok(())
    }

    /// Get duration of media file in seconds
    async fn get_duration(&self, input: &Path) -> TranscodeResult<f64> {
        let ffprobe_path = platform::find_ffprobe()?;

        let output = Command::new(ffprobe_path)
            .args(&[
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                &input.to_string_lossy(),
            ])
            .output()
            .await
            .map_err(|e| TranscodeError::Platform(format!("Failed to run ffprobe: {}", e)))?;

        if !output.status.success() {
            return Err(TranscodeError::Platform(
                "Failed to get media duration".to_string(),
            ));
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        duration_str
            .trim()
            .parse::<f64>()
            .map_err(|_| TranscodeError::Platform("Invalid duration format".to_string()))
    }

    /// Execute FFmpeg with progress monitoring
    async fn execute_ffmpeg<F>(
        &self,
        args: &[String],
        duration: f64,
        progress_callback: &mut F,
    ) -> TranscodeResult<()>
    where
        F: FnMut(f32, Option<f32>) + Send,
    {
        let mut child = Command::new(&self.ffmpeg_path)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TranscodeError::FfmpegFailed(format!("Failed to spawn FFmpeg: {}", e)))?;

        // Parse stderr for progress
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TranscodeError::FfmpegFailed("Cannot capture stderr".to_string()))?;

        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        let time_regex = Regex::new(r"time=(\d{2}):(\d{2}):(\d{2}\.\d{2})").unwrap();
        let fps_regex = Regex::new(r"fps=\s*(\d+\.?\d*)").unwrap();

        while let Some(line) = lines.next_line().await.map_err(|e| {
            TranscodeError::FfmpegFailed(format!("Error reading FFmpeg output: {}", e))
        })? {
            debug!("FFmpeg: {}", line);

            // Parse time progress
            if let Some(caps) = time_regex.captures(&line) {
                let hours: f64 = caps[1].parse().unwrap_or(0.0);
                let minutes: f64 = caps[2].parse().unwrap_or(0.0);
                let seconds: f64 = caps[3].parse().unwrap_or(0.0);

                let current_time = hours * 3600.0 + minutes * 60.0 + seconds;
                let progress = if duration > 0.0 {
                    ((current_time / duration) * 100.0).min(100.0) as f32
                } else {
                    0.0
                };

                // Parse FPS
                let fps = fps_regex
                    .captures(&line)
                    .and_then(|caps| caps[1].parse::<f32>().ok());

                progress_callback(progress, fps);
            }
        }

        // Wait for completion
        let status = child.wait().await.map_err(|e| {
            TranscodeError::FfmpegFailed(format!("Failed to wait for FFmpeg: {}", e))
        })?;

        if !status.success() {
            return Err(TranscodeError::FfmpegFailed(format!(
                "FFmpeg exited with code: {:?}",
                status.code()
            )));
        }

        // Final progress update
        progress_callback(100.0, None);

        Ok(())
    }

    /// Extract BWF audio files with frame-accurate BEXT timecode from the input
    /// Uses the validated frame-based method for 23.976fps
    async fn extract_bwf_audio(&self, input: &Path, output: &Path) -> TranscodeResult<()> {
        info!("Extracting BWF audio with BEXT timecode from: {:?}", input);

        // Determine output directory (same as video output)
        let output_dir = output
            .parent()
            .ok_or_else(|| TranscodeError::InvalidInput("Invalid output path".to_string()))?;

        // Get base filename without extension
        let base_name = output
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| TranscodeError::InvalidInput("Invalid output filename".to_string()))?;

        // Step 1: Extract timecode from source MXF
        info!("Extracting timecode from source...");
        let timecode_output = Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-show_entries", "format_tags:stream_tags",
                &input.to_string_lossy(),
            ])
            .output()
            .await
            .map_err(|e| TranscodeError::Platform(format!("Failed to extract timecode: {}", e)))?;

        let timecode_str = String::from_utf8_lossy(&timecode_output.stdout);
        let mut timecode = None;
        
        for line in timecode_str.lines() {
            if line.contains("timecode=") {
                if let Some(tc) = line.split('=').nth(1) {
                    timecode = Some(tc.trim().to_string());
                    break;
                }
            }
        }

        let timecode = timecode.unwrap_or_else(|| {
            warn!("No timecode found in source, using 00:00:00:00");
            "00:00:00:00".to_string()
        });

        info!("Source timecode: {}", timecode);

        // Step 2: Build intermediate and final BWF output paths
        let temp_wav = output_dir.join(format!("{}_audio_temp.wav", base_name));
        let bwf_output = output_dir.join(format!("{}_audio.wav", base_name));

        // Step 3: Extract audio as plain WAV (PCM 24-bit, 48kHz - required for BEXT)
        // Mix down to stereo since WAV format only supports 1-2 channels
        info!("Extracting audio to temporary WAV (stereo mixdown)...");
        let extract_args = vec![
            "-y".to_string(),
            "-i".to_string(),
            input.to_string_lossy().to_string(),
            "-vn".to_string(), // No video
            "-acodec".to_string(),
            "pcm_s24le".to_string(), // 24-bit PCM
            "-ar".to_string(),
            "48000".to_string(), // 48kHz sample rate (required for validated formula)
            "-ac".to_string(),
            "2".to_string(), // Stereo output (WAV muxer limit)
            temp_wav.to_string_lossy().to_string(),
        ];

        let extract_status = Command::new(&self.ffmpeg_path)
            .args(&extract_args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .status()
            .await
            .map_err(|e| TranscodeError::FfmpegFailed(format!("Failed to extract audio: {}", e)))?;

        if !extract_status.success() {
            return Err(TranscodeError::FfmpegFailed(format!(
                "Audio extraction failed with code: {:?}",
                extract_status.code()
            )));
        }

        // Step 4: Calculate TimeReference using validated frame-based method
        info!("Calculating TimeReference using validated frame-based method (23.976fps)...");
        
        // Parse timecode (HH:MM:SS:FF format)
        let tc_parts: Vec<&str> = timecode.split(':').collect();
        if tc_parts.len() != 4 {
            warn!("Invalid timecode format, using plain WAV");
            std::fs::rename(&temp_wav, &bwf_output).map_err(|e| {
                TranscodeError::Platform(format!("Failed to rename output: {}", e))
            })?;
            return Ok(());
        }

        let hours: u64 = tc_parts[0].parse().unwrap_or(0);
        let minutes: u64 = tc_parts[1].parse().unwrap_or(0);
        let seconds: u64 = tc_parts[2].parse().unwrap_or(0);
        let frames: u64 = tc_parts[3].parse().unwrap_or(0);

        // Validated frame-based formula for 23.976fps @ 48000 Hz
        // TimeReference = total_frames × 2004.005263
        const FRAME_RATE: f64 = 23.976;
        const MULTIPLIER: f64 = 2004.005263;
        
        let total_frames = (hours as f64 * 60.0 * 60.0 * FRAME_RATE)
            + (minutes as f64 * 60.0 * FRAME_RATE)
            + (seconds as f64 * FRAME_RATE)
            + frames as f64;
        
        let time_reference = (total_frames * MULTIPLIER) as u64;
        
        info!("TimeReference calculated: {} (timecode: {})", time_reference, timecode);

        // Step 5: Insert BEXT using validated Python script
        info!("Inserting BEXT chunk...");
        
        // Find the Python script
        let script_path = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("bwf-tools/insert_bext_timecode.py");

        let bext_status = Command::new("python3")
            .args(&[
                script_path.to_string_lossy().as_ref(),
                temp_wav.to_string_lossy().as_ref(),
                bwf_output.to_string_lossy().as_ref(),
                "--time-ref", &time_reference.to_string(),
                "--sample-rate", "48000",
                "--frame-rate", "23.976",
                "--description", &timecode,
                "--originator", "Industrial Transcoder v2",
            ])
            .output()
            .await
            .map_err(|e| {
                warn!("BEXT insertion failed: {}. Falling back to plain WAV.", e);
                e
            });

        // If BEXT insertion fails, just rename the temp file
        if bext_status.is_err() || !bext_status.as_ref().unwrap().status.success() {
            if let Ok(status) = &bext_status {
                let stderr = String::from_utf8_lossy(&status.stderr);
                warn!("BEXT script output: {}", stderr);
            }
            warn!("BEXT insertion not available, using plain WAV without timecode metadata");
            std::fs::rename(&temp_wav, &bwf_output).map_err(|e| {
                TranscodeError::Platform(format!("Failed to rename output: {}", e))
            })?;
        } else {
            // Clean up temp file if it still exists
            let _ = std::fs::remove_file(&temp_wav);
            info!("BEXT timecode inserted successfully with TimeReference: {}", time_reference);
        }

        info!("BWF audio extraction completed: {:?}", bwf_output);
        Ok(())
    }

    /// Verify FFmpeg is working
    pub async fn verify(&self) -> TranscodeResult<String> {
        let output = Command::new(&self.ffmpeg_path)
            .arg("-version")
            .output()
            .await
            .map_err(|e| TranscodeError::FfmpegFailed(format!("FFmpeg verification failed: {}", e)))?;

        if !output.status.success() {
            return Err(TranscodeError::FfmpegFailed(
                "FFmpeg version check failed".to_string(),
            ));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        let first_line = version.lines().next().unwrap_or("Unknown version");
        Ok(first_line.to_string())
    }
}

impl Default for Transcoder {
    fn default() -> Self {
        Self::new().expect("Failed to initialize transcoder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transcoder_creation() {
        match Transcoder::new() {
            Ok(transcoder) => {
                println!("Transcoder created successfully");
                // Try to verify FFmpeg
                match transcoder.verify().await {
                    Ok(version) => println!("FFmpeg version: {}", version),
                    Err(e) => println!("FFmpeg verification failed: {}", e),
                }
            }
            Err(e) => {
                println!("FFmpeg not available: {}", e);
            }
        }
    }
}

