//! Job logging system for detailed transcoding history
//!
//! Logs include: codec specs, system info, performance metrics, timestamps

use crate::job::Job;
use crate::platform;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Detailed job log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLogEntry {
    // Job identification
    pub job_id: String,
    pub timestamp: DateTime<Utc>,
    
    // File information
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub input_size_mb: Option<f64>,
    pub output_size_mb: Option<f64>,
    
    // Codec information
    pub transcode_config: serde_json::Value,
    
    // Job status and timing
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub error_message: Option<String>,
    
    // System information at time of transcode
    pub system_info: SystemSnapshot,
    
    // Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// System information snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub platform: String,
    pub cpu_cores: usize,
    pub available_memory_mb: Option<u64>,
    pub ffmpeg_version: Option<String>,
}

/// Performance metrics for the transcode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub read_speed_mbps: Option<f64>,
    pub write_speed_mbps: Option<f64>,
    pub avg_fps: Option<f32>,
    pub processing_speed: Option<f64>, // As ratio of real-time (e.g., 2.5x)
}

impl JobLogEntry {
    /// Create a log entry from a job
    pub fn from_job(job: &Job) -> Self {
        let input_size_mb = get_file_size_mb(&job.input_path);
        let output_size_mb = get_file_size_mb(&job.output_path);
        
        // Calculate read/write speeds if we have the data
        let performance_metrics = calculate_performance_metrics(
            job,
            input_size_mb,
            output_size_mb,
        );
        
        Self {
            job_id: job.id.to_string(),
            timestamp: Utc::now(),
            input_path: job.input_path.clone(),
            output_path: job.output_path.clone(),
            input_size_mb,
            output_size_mb,
            transcode_config: job.config.clone(),
            status: format!("{:?}", job.status),
            created_at: job.created_at,
            started_at: job.started_at,
            completed_at: job.completed_at,
            duration_seconds: job.duration_seconds(),
            error_message: job.error_message.clone(),
            system_info: SystemSnapshot::capture(),
            performance_metrics,
        }
    }
    
    /// Format as human-readable text
    pub fn format_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&"=".repeat(80));
        output.push_str("\nJOB LOG ENTRY\n");
        output.push_str(&"=".repeat(80));
        output.push_str(&format!("\nJob ID: {}\n", self.job_id));
        output.push_str(&format!("Logged: {}\n", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("Status: {}\n", self.status));
        
        output.push_str("\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\nFILE INFORMATION\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\n");
        output.push_str(&format!("Input:  {}\n", self.input_path.display()));
        if let Some(size) = self.input_size_mb {
            output.push_str(&format!("        {:.2} MB\n", size));
        }
        output.push_str(&format!("Output: {}\n", self.output_path.display()));
        if let Some(size) = self.output_size_mb {
            output.push_str(&format!("        {:.2} MB\n", size));
        }
        
        output.push_str("\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\nCODEC CONFIGURATION\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\n");
        output.push_str(&format!("{}\n", serde_json::to_string_pretty(&self.transcode_config).unwrap_or_default()));
        
        output.push_str("\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\nTIMING INFORMATION\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\n");
        output.push_str(&format!("Created:   {}\n", self.created_at.format("%Y-%m-%d %H:%M:%S UTC")));
        if let Some(started) = self.started_at {
            output.push_str(&format!("Started:   {}\n", started.format("%Y-%m-%d %H:%M:%S UTC")));
        }
        if let Some(completed) = self.completed_at {
            output.push_str(&format!("Completed: {}\n", completed.format("%Y-%m-%d %H:%M:%S UTC")));
        }
        if let Some(duration) = self.duration_seconds {
            output.push_str(&format!("Duration:  {} seconds ({:.2} minutes)\n", duration, duration as f64 / 60.0));
        }
        
        output.push_str("\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\nSYSTEM INFORMATION\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\n");
        output.push_str(&format!("Platform: {}\n", self.system_info.platform));
        output.push_str(&format!("CPU Cores: {}\n", self.system_info.cpu_cores));
        if let Some(mem) = self.system_info.available_memory_mb {
            output.push_str(&format!("Available Memory: {} MB ({:.2} GB)\n", mem, mem as f64 / 1024.0));
        }
        if let Some(ffmpeg) = &self.system_info.ffmpeg_version {
            output.push_str(&format!("FFmpeg: {}\n", ffmpeg));
        }
        
        output.push_str("\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\nPERFORMANCE METRICS\n");
        output.push_str(&"-".repeat(80));
        output.push_str("\n");
        
        if let Some(read_speed) = self.performance_metrics.read_speed_mbps {
            output.push_str(&format!("Read Speed:  {:.2} MB/s\n", read_speed));
        }
        if let Some(write_speed) = self.performance_metrics.write_speed_mbps {
            output.push_str(&format!("Write Speed: {:.2} MB/s\n", write_speed));
        }
        if let Some(fps) = self.performance_metrics.avg_fps {
            output.push_str(&format!("Average FPS: {:.2}\n", fps));
        }
        if let Some(speed) = self.performance_metrics.processing_speed {
            output.push_str(&format!("Processing Speed: {:.2}x realtime\n", speed));
        }
        
        if let Some(err) = &self.error_message {
            output.push_str("\n");
            output.push_str(&"-".repeat(80));
            output.push_str("\nERROR MESSAGE\n");
            output.push_str(&"-".repeat(80));
            output.push_str("\n");
            output.push_str(&format!("{}\n", err));
        }
        
        output.push_str("\n");
        output.push_str(&"=".repeat(80));
        output.push_str("\n\n");
        
        output
    }
}

impl SystemSnapshot {
    /// Capture current system information
    pub fn capture() -> Self {
        let platform = platform::Platform::current();
        
        Self {
            platform: platform.name().to_string(),
            cpu_cores: platform::cpu_count(),
            available_memory_mb: platform::available_memory_mb(),
            ffmpeg_version: get_ffmpeg_version(),
        }
    }
}

/// Get file size in MB
fn get_file_size_mb(path: &Path) -> Option<f64> {
    fs::metadata(path).ok().map(|meta| {
        meta.len() as f64 / (1024.0 * 1024.0)
    })
}

/// Calculate performance metrics
fn calculate_performance_metrics(
    job: &Job,
    input_size_mb: Option<f64>,
    output_size_mb: Option<f64>,
) -> PerformanceMetrics {
    let duration = job.duration_seconds().map(|d| d as f64);
    
    let read_speed_mbps = match (input_size_mb, duration) {
        (Some(size), Some(dur)) if dur > 0.0 => Some(size / dur),
        _ => None,
    };
    
    let write_speed_mbps = match (output_size_mb, duration) {
        (Some(size), Some(dur)) if dur > 0.0 => Some(size / dur),
        _ => None,
    };
    
    PerformanceMetrics {
        read_speed_mbps,
        write_speed_mbps,
        avg_fps: None, // Could be extracted from FFmpeg output in the future
        processing_speed: None, // Could be calculated from media duration vs transcode time
    }
}

/// Get FFmpeg version
fn get_ffmpeg_version() -> Option<String> {
    use std::process::Command;
    
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8(output.stdout).ok()
        })
        .and_then(|version_str| {
            version_str.lines().next().map(|s| s.to_string())
        })
}

/// Job logger for writing logs to disk
pub struct JobLogger {
    log_dir: PathBuf,
}

impl JobLogger {
    /// Create a new job logger
    pub fn new(log_dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let log_dir = log_dir.into();
        
        // Create log directory if it doesn't exist
        fs::create_dir_all(&log_dir)?;
        
        Ok(Self { log_dir })
    }
    
    /// Get default log directory (in user's home or current dir)
    pub fn default_log_dir() -> PathBuf {
        if let Some(home) = home::home_dir() {
            home.join(".industrial-transcoder").join("logs")
        } else {
            PathBuf::from("transcoder_logs")
        }
    }
    
    /// Log a job
    pub fn log_job(&self, job: &Job) -> std::io::Result<PathBuf> {
        let entry = JobLogEntry::from_job(job);
        
        // Create log file path: YYYY-MM-DD_transcoder_log.txt
        let date_str = entry.timestamp.format("%Y-%m-%d");
        let log_file = self.log_dir.join(format!("{}_transcoder_log.txt", date_str));
        
        // Append to daily log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?;
        
        writeln!(file, "{}", entry.format_text())?;
        
        // Also write JSON log
        let json_log_file = self.log_dir.join(format!("{}_transcoder_log.json", date_str));
        let mut entries: Vec<JobLogEntry> = if json_log_file.exists() {
            let json_str = fs::read_to_string(&json_log_file)?;
            serde_json::from_str(&json_str).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        entries.push(entry);
        
        let json_str = serde_json::to_string_pretty(&entries)?;
        fs::write(&json_log_file, json_str)?;
        
        Ok(log_file)
    }
    
    /// Log multiple jobs
    pub fn log_jobs(&self, jobs: &[Job]) -> std::io::Result<usize> {
        let mut count = 0;
        
        for job in jobs {
            if let Ok(_) = self.log_job(job) {
                count += 1;
            }
        }
        
        Ok(count)
    }
}

impl Default for JobLogger {
    fn default() -> Self {
        let log_dir = Self::default_log_dir();
        Self::new(log_dir).expect("Failed to create default log directory")
    }
}

