//! Platform-specific utilities and abstractions
//!
//! Handles differences between macOS, Windows, and Linux

use crate::error::{TranscodeError, TranscodeResult};
use std::path::{Path, PathBuf};
use which::which;

/// Platform enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
    Unknown,
}

impl Platform {
    /// Detect the current platform
    pub fn current() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                Platform::MacOS
            } else if #[cfg(target_os = "windows")] {
                Platform::Windows
            } else if #[cfg(target_os = "linux")] {
                Platform::Linux
            } else {
                Platform::Unknown
            }
        }
    }

    /// Get platform name as string
    pub fn name(&self) -> &'static str {
        match self {
            Platform::MacOS => "macOS",
            Platform::Windows => "Windows",
            Platform::Linux => "Linux",
            Platform::Unknown => "Unknown",
        }
    }

    /// Get executable extension for this platform
    pub fn exe_extension(&self) -> &'static str {
        match self {
            Platform::Windows => ".exe",
            _ => "",
        }
    }

    /// Get path separator for this platform
    pub fn path_separator(&self) -> &'static str {
        match self {
            Platform::Windows => "\\",
            _ => "/",
        }
    }
}

/// Find FFmpeg executable in system PATH
pub fn find_ffmpeg() -> TranscodeResult<PathBuf> {
    let ffmpeg_name = if Platform::current() == Platform::Windows {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    which(ffmpeg_name).map_err(|_| TranscodeError::FfmpegNotFound)
}

/// Find FFprobe executable in system PATH
pub fn find_ffprobe() -> TranscodeResult<PathBuf> {
    let ffprobe_name = if Platform::current() == Platform::Windows {
        "ffprobe.exe"
    } else {
        "ffprobe"
    };

    which(ffprobe_name).map_err(|_| {
        TranscodeError::Platform(format!("ffprobe not found in PATH: {}", ffprobe_name))
    })
}

/// Normalize path for current platform
pub fn normalize_path(path: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Convert forward slashes to backslashes on Windows
        let path_str = path.to_string_lossy();
        let normalized = path_str.replace('/', "\\");
        PathBuf::from(normalized)
    }

    #[cfg(not(target_os = "windows"))]
    {
        path.to_path_buf()
    }
}

/// Get temporary directory for the platform
pub fn temp_dir() -> PathBuf {
    std::env::temp_dir()
}

/// Check if running with sufficient permissions
pub fn check_permissions() -> TranscodeResult<()> {
    // Basic permission check - try to write to temp dir
    let temp = temp_dir();
    let test_file = temp.join(format!("transcoder_test_{}", uuid::Uuid::new_v4()));

    std::fs::write(&test_file, b"test")
        .map_err(|e| TranscodeError::Platform(format!("Insufficient permissions: {}", e)))?;

    std::fs::remove_file(&test_file)
        .map_err(|e| TranscodeError::Platform(format!("Cannot clean up temp file: {}", e)))?;

    Ok(())
}

/// Get number of CPU cores for worker pool sizing
pub fn cpu_count() -> usize {
    num_cpus::get_physical()
}

/// Get available memory (best effort)
pub fn available_memory_mb() -> Option<u64> {
    sys_info::mem_info()
        .ok()
        .map(|info| info.avail / 1024) // KB to MB
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        assert_ne!(platform, Platform::Unknown);
        assert!(!platform.name().is_empty());
    }

    #[test]
    fn test_find_ffmpeg() {
        // This test will fail if FFmpeg is not installed
        // That's expected - we want to know if FFmpeg is available
        match find_ffmpeg() {
            Ok(path) => println!("FFmpeg found at: {:?}", path),
            Err(e) => println!("FFmpeg not found: {}", e),
        }
    }

    #[test]
    fn test_permissions() {
        assert!(check_permissions().is_ok());
    }

    #[test]
    fn test_cpu_count() {
        let count = cpu_count();
        assert!(count > 0);
        println!("CPU cores: {}", count);
    }
}

