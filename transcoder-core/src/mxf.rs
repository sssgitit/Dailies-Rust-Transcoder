//! MXF-specific operations
//! 
//! Handles MXF rewrapping, metadata extraction, and format conversion

use crate::error::{TranscodeError, TranscodeResult};
use crate::platform;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// MXF wrapping mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MxfWrapping {
    /// Clip-wrapped (contiguous essence, one chunk per track)
    ClipWrapped,
    /// Frame-wrapped (interleaved essence, frame-by-frame)
    FrameWrapped,
}

impl MxfWrapping {
    pub fn as_str(&self) -> &'static str {
        match self {
            MxfWrapping::ClipWrapped => "clip",
            MxfWrapping::FrameWrapped => "frame",
        }
    }
}

/// MXF operational pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MxfOperationalPattern {
    /// OP-Atom (used by Avid)
    OPAtom,
    /// OP1a (single item, single package)
    OP1a,
    /// OP1b (single item, ganged packages)
    OP1b,
    /// Unknown pattern
    Unknown(String),
}

/// MXF metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MxfMetadata {
    pub operational_pattern: MxfOperationalPattern,
    pub wrapping: MxfWrapping,
    pub material_package_uid: Option<String>,
    pub duration_frames: Option<u64>,
    pub edit_rate: Option<String>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
}

/// MXF rewrapper using bmxtranswrap
pub struct MxfRewrapper {
    bmxtranswrap_path: Option<PathBuf>,
}

impl MxfRewrapper {
    /// Create a new MXF rewrapper
    pub fn new() -> Self {
        let bmxtranswrap_path = which::which("bmxtranswrap").ok();
        
        if bmxtranswrap_path.is_some() {
            info!("bmxtranswrap found at: {:?}", bmxtranswrap_path);
        } else {
            warn!("bmxtranswrap not found - MXF rewrapping will not be available");
        }
        
        Self { bmxtranswrap_path }
    }

    /// Check if rewrapping is available
    pub fn is_available(&self) -> bool {
        self.bmxtranswrap_path.is_some()
    }

    /// Detect MXF wrapping type
    pub async fn detect_wrapping(&self, input: &Path) -> TranscodeResult<MxfWrapping> {
        if !input.exists() {
            return Err(TranscodeError::InvalidInput(format!(
                "Input file not found: {:?}",
                input
            )));
        }

        // Use ffprobe to detect wrapping
        let ffprobe = platform::find_ffprobe()?;
        
        let output = Command::new(ffprobe)
            .args(&[
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
                &input.to_string_lossy(),
            ])
            .output()
            .await
            .map_err(|e| TranscodeError::Platform(format!("Failed to run ffprobe: {}", e)))?;

        if !output.status.success() {
            return Err(TranscodeError::Platform(
                "Failed to detect MXF wrapping".to_string(),
            ));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse JSON and check for wrapping indicators
        // Clip-wrapped typically has larger packet sizes and fewer packets
        // Frame-wrapped has many small packets
        
        if json_str.contains("\"nb_frames\"") {
            // Try to infer from packet structure
            // This is a heuristic - bmxtranswrap gives more accurate info
            if let Some(bmx_path) = &self.bmxtranswrap_path {
                return self.detect_wrapping_with_bmx(input, bmx_path).await;
            }
        }

        // Default assumption if we can't determine
        warn!("Could not definitively detect wrapping, assuming clip-wrapped");
        Ok(MxfWrapping::ClipWrapped)
    }

    /// Detect wrapping using bmxtranswrap
    async fn detect_wrapping_with_bmx(
        &self,
        input: &Path,
        bmx_path: &Path,
    ) -> TranscodeResult<MxfWrapping> {
        // Run bmxtranswrap with --info flag to get metadata
        let output = Command::new(bmx_path)
            .args(&[
                "--info",
                &input.to_string_lossy(),
            ])
            .output()
            .await
            .map_err(|e| TranscodeError::Platform(format!("Failed to run bmxtranswrap: {}", e)))?;

        let info = String::from_utf8_lossy(&output.stdout);
        
        // Parse bmxtranswrap output for wrapping type
        if info.contains("frame-wrapped") || info.contains("Frame Wrapped") {
            Ok(MxfWrapping::FrameWrapped)
        } else if info.contains("clip-wrapped") || info.contains("Clip Wrapped") {
            Ok(MxfWrapping::ClipWrapped)
        } else {
            warn!("Could not determine wrapping from bmxtranswrap output");
            Ok(MxfWrapping::ClipWrapped)
        }
    }

    /// Rewrap MXF from clip-wrapped to frame-wrapped
    pub async fn rewrap<F>(
        &self,
        input: &Path,
        output: &Path,
        target_wrapping: MxfWrapping,
        mut progress_callback: F,
    ) -> TranscodeResult<()>
    where
        F: FnMut(f32) + Send,
    {
        let bmx_path = self.bmxtranswrap_path.as_ref().ok_or_else(|| {
            TranscodeError::Platform(
                "bmxtranswrap not found - cannot rewrap MXF files".to_string(),
            )
        })?;

        if !input.exists() {
            return Err(TranscodeError::InvalidInput(format!(
                "Input file not found: {:?}",
                input
            )));
        }

        info!("Rewrapping MXF to {:?}: {:?} → {:?}", target_wrapping, input, output);

        // Detect current wrapping
        let current_wrapping = self.detect_wrapping(input).await?;
        
        if current_wrapping == target_wrapping {
            info!("File is already {:?}, no rewrapping needed", target_wrapping);
            // Just copy the file
            tokio::fs::copy(input, output).await.map_err(|e| {
                TranscodeError::Io(e)
            })?;
            progress_callback(100.0);
            return Ok(());
        }

        // Build bmxtranswrap command
        let mut args = vec![
            "-t".to_string(),
            "op1a".to_string(), // Output as OP1a
        ];

        // Set wrapping mode
        match target_wrapping {
            MxfWrapping::FrameWrapped => {
                args.push("--frame-layout".to_string());
                args.push("separate".to_string());
            }
            MxfWrapping::ClipWrapped => {
                args.push("--clip-wrap".to_string());
            }
        }

        args.push("-o".to_string());
        args.push(output.to_string_lossy().to_string());
        args.push(input.to_string_lossy().to_string());

        debug!("bmxtranswrap command: {:?} {:?}", bmx_path, args);

        // Execute bmxtranswrap
        let mut child = Command::new(bmx_path)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                TranscodeError::Platform(format!("Failed to spawn bmxtranswrap: {}", e))
            })?;

        // Monitor stderr for progress
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| TranscodeError::Platform("Cannot capture stderr".to_string()))?;

        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await.map_err(|e| {
            TranscodeError::Platform(format!("Error reading bmxtranswrap output: {}", e))
        })? {
            debug!("bmxtranswrap: {}", line);
            
            // bmxtranswrap doesn't provide great progress info
            // We can estimate based on file operations
            // For now, just show activity
        }

        // Wait for completion
        let status = child.wait().await.map_err(|e| {
            TranscodeError::Platform(format!("Failed to wait for bmxtranswrap: {}", e))
        })?;

        if !status.success() {
            return Err(TranscodeError::Platform(format!(
                "bmxtranswrap exited with code: {:?}",
                status.code()
            )));
        }

        // Verify output was created
        if !output.exists() {
            return Err(TranscodeError::Platform(
                "Output file was not created".to_string(),
            ));
        }

        progress_callback(100.0);
        info!("MXF rewrapping completed: {:?}", output);
        Ok(())
    }

    /// Convert clip-wrapped to frame-wrapped (convenience method)
    pub async fn clip_to_frame<F>(
        &self,
        input: &Path,
        output: &Path,
        progress_callback: F,
    ) -> TranscodeResult<()>
    where
        F: FnMut(f32) + Send,
    {
        self.rewrap(input, output, MxfWrapping::FrameWrapped, progress_callback)
            .await
    }

    /// Convert frame-wrapped to clip-wrapped (convenience method)
    pub async fn frame_to_clip<F>(
        &self,
        input: &Path,
        output: &Path,
        progress_callback: F,
    ) -> TranscodeResult<()>
    where
        F: FnMut(f32) + Send,
    {
        self.rewrap(input, output, MxfWrapping::ClipWrapped, progress_callback)
            .await
    }

    /// Batch rewrap multiple files
    pub async fn batch_rewrap<F>(
        &self,
        files: &[(PathBuf, PathBuf)],
        target_wrapping: MxfWrapping,
        mut progress_callback: F,
    ) -> TranscodeResult<Vec<TranscodeResult<()>>>
    where
        F: FnMut(usize, usize, f32) + Send,
    {
        let total = files.len();
        let mut results = Vec::new();

        for (index, (input, output)) in files.iter().enumerate() {
            info!("Processing file {}/{}: {:?}", index + 1, total, input);
            
            let result = self
                .rewrap(
                    input,
                    output,
                    target_wrapping,
                    |progress| {
                        progress_callback(index + 1, total, progress);
                    },
                )
                .await;

            results.push(result);
        }

        Ok(results)
    }
}

impl Default for MxfRewrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mxf_rewrapper_creation() {
        let rewrapper = MxfRewrapper::new();
        
        if rewrapper.is_available() {
            println!("bmxtranswrap is available");
        } else {
            println!("bmxtranswrap is not available - install from https://github.com/ebu/bmx");
        }
    }

    #[test]
    fn test_wrapping_enum() {
        assert_eq!(MxfWrapping::ClipWrapped.as_str(), "clip");
        assert_eq!(MxfWrapping::FrameWrapped.as_str(), "frame");
    }
}

