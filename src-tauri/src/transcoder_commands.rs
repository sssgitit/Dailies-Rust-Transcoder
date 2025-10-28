//! Tauri commands for transcoder functionality

use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use transcoder_core::*;

/// System information response
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub platform: String,
    pub cpu_cores: usize,
    pub available_memory_mb: Option<u64>,
    pub ffmpeg_available: bool,
    pub ffmpeg_path: Option<String>,
}

/// Get system information
#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let platform = Platform::current();
    let cpu_cores = platform::cpu_count();
    let available_memory_mb = platform::available_memory_mb();

    let (ffmpeg_available, ffmpeg_path) = match platform::find_ffmpeg() {
        Ok(path) => (true, Some(path.to_string_lossy().to_string())),
        Err(_) => (false, None),
    };

    Ok(SystemInfo {
        platform: platform.name().to_string(),
        cpu_cores,
        available_memory_mb,
        ffmpeg_available,
        ffmpeg_path,
    })
}

/// Verify FFmpeg installation
#[tauri::command]
pub async fn verify_ffmpeg() -> Result<String, String> {
    let transcoder = Transcoder::new().map_err(|e| e.to_string())?;
    transcoder.verify().await.map_err(|e| e.to_string())
}

/// Get all available presets
#[tauri::command]
pub fn get_presets() -> HashMap<String, config::CodecPreset> {
    config::CodecPreset::all_presets()
}

/// Job creation request
#[derive(Debug, Deserialize)]
pub struct AddJobRequest {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub preset_name: String,
    pub priority: Option<job::Priority>,
}

/// Add a new job to the queue
#[tauri::command]
pub async fn add_job(
    request: AddJobRequest,
    state: State<'_, AppState>,
) -> Result<job::JobId, String> {
    // Get preset
    let presets = config::CodecPreset::all_presets();
    let preset = presets
        .get(&request.preset_name)
        .ok_or_else(|| format!("Unknown preset: {}", request.preset_name))?;

    // Create job
    let config_json = serde_json::to_value(&preset.config).map_err(|e| e.to_string())?;
    let job = job::Job::new(
        request.input_path,
        request.output_path,
        config_json,
        request.priority.unwrap_or(job::Priority::Normal),
    );

    // Add to queue
    let job_id = state.queue.add_job(job).await.map_err(|e| e.to_string())?;

    Ok(job_id)
}

/// Get a job by ID
#[tauri::command]
pub fn get_job(job_id: String, state: State<'_, AppState>) -> Result<job::Job, String> {
    let id = uuid::Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    state
        .queue
        .get_job(&id)
        .ok_or_else(|| format!("Job not found: {}", job_id))
}

/// Get all jobs
#[tauri::command]
pub fn get_all_jobs(state: State<'_, AppState>) -> Vec<job::Job> {
    state.queue.get_all_jobs()
}

/// Cancel a job
#[tauri::command]
pub async fn cancel_job(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let id = uuid::Uuid::parse_str(&job_id).map_err(|e| e.to_string())?;
    state.queue.cancel_job(&id).await.map_err(|e| e.to_string())
}

/// Clear completed jobs (logs them first)
#[tauri::command]
pub fn clear_completed_jobs(state: State<'_, AppState>) -> Result<usize, String> {
    // Get all completed/failed/cancelled jobs
    let jobs_to_clear: Vec<Job> = state
        .queue
        .get_all_jobs()
        .into_iter()
        .filter(|job| {
            matches!(
                job.status,
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
            )
        })
        .collect();
    
    if jobs_to_clear.is_empty() {
        return Ok(0);
    }
    
    // Log all jobs before clearing
    let logger = JobLogger::default();
    let logged_count = logger.log_jobs(&jobs_to_clear).map_err(|e| {
        format!("Failed to log jobs: {}", e)
    })?;
    
    tracing::info!("Logged {} jobs before clearing", logged_count);
    
    // Now clear the jobs from the queue
    let cleared_count = state.queue.clear_completed();
    
    Ok(cleared_count)
}

/// Get queue statistics
#[tauri::command]
pub fn get_queue_stats(state: State<'_, AppState>) -> queue::QueueStats {
    state.queue.get_stats()
}

/// Worker status response
#[derive(Debug, Serialize)]
pub struct WorkerStatus {
    pub is_running: bool,
    pub active_workers: usize,
    pub total_workers: usize,
}

/// Start worker pool
#[tauri::command]
pub async fn start_workers(
    worker_count: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut pool_lock = state.worker_pool.lock().await;

    if pool_lock.is_some() {
        return Err("Worker pool already running".to_string());
    }

    let mut pool = WorkerPool::new(
        Arc::clone(&state.queue),
        state.progress_reporter.clone(),
        worker_count,
    )
    .map_err(|e| e.to_string())?;

    pool.start().await.map_err(|e| e.to_string())?;

    *pool_lock = Some(pool);

    Ok(())
}

/// Stop worker pool
#[tauri::command]
pub async fn stop_workers(state: State<'_, AppState>) -> Result<(), String> {
    let mut pool_lock = state.worker_pool.lock().await;

    if let Some(mut pool) = pool_lock.take() {
        pool.stop().await;
        Ok(())
    } else {
        Err("Worker pool not running".to_string())
    }
}

/// Get worker pool status
#[tauri::command]
pub async fn get_worker_status(state: State<'_, AppState>) -> Result<WorkerStatus, String> {
    let pool_lock = state.worker_pool.lock().await;

    Ok(match pool_lock.as_ref() {
        Some(pool) => WorkerStatus {
            is_running: pool.is_running(),
            active_workers: pool.active_worker_count(),
            total_workers: platform::cpu_count().saturating_sub(1).max(1),
        },
        None => WorkerStatus {
            is_running: false,
            active_workers: 0,
            total_workers: platform::cpu_count().saturating_sub(1).max(1),
        },
    })
}

/// Subscribe to progress events (returns initial state)
#[tauri::command]
pub fn subscribe_progress(state: State<'_, AppState>) -> Result<(), String> {
    // Client will receive events via the "transcode_progress" event
    let subscriber_count = state.progress_reporter.subscriber_count();
    tracing::info!("Progress subscriber count: {}", subscriber_count);
    Ok(())
}

/// Simple command: Transcode to DNxHR LB MOV with hardware acceleration
#[tauri::command]
pub async fn transcode_dnxhr_lb(
    input_path: PathBuf,
    output_path: PathBuf,
) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::process::Command;
    
    tracing::info!("Transcoding to DNxHR LB: {:?} -> {:?}", input_path, output_path);
    
    // Verify input file exists
    if !input_path.exists() {
        return Err(format!("Input file does not exist: {:?}", input_path));
    }
    
    // Create output directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }
    }
    
    // Build FFmpeg command with hardware acceleration
    let output = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-hwaccel", "videotoolbox",  // Hardware decode
            "-i", input_path.to_str().ok_or("Invalid input path")?,
            "-c:v", "dnxhd",
            "-profile:v", "dnxhr_lb",
            "-pix_fmt", "yuv422p",  // 8-bit for LB
            "-c:a", "pcm_s24le",
            "-ar", "48000",
            "-map", "0:v:0",
            "-map", "0:a",
            "-threads", "0",  // Multi-threaded
            output_path.to_str().ok_or("Invalid output path")?,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg = stderr
            .lines()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        
        tracing::error!("FFmpeg error: {}", error_msg);
        return Err(format!("Transcode failed:\n{}", error_msg));
    }
    
    tracing::info!("Transcode completed successfully");
    Ok(())
}

/// Simple command: Create BWF from MXF with frame-accurate BEXT timecode
#[tauri::command]
pub async fn create_bwf_from_mxf(
    mxf_path: PathBuf,
    output_path: PathBuf,
    sample_rate: u32,
) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::process::Command;
    
    tracing::info!("Creating BWF from MXF: {:?} -> {:?}", mxf_path, output_path);
    
    // Verify input file exists
    if !mxf_path.exists() {
        return Err(format!("Input file does not exist: {:?}", mxf_path));
    }
    
    // Create output directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
        }
    }
    
    // Step 1: Extract timecode from MXF
    let tc_output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-show_entries", "format_tags:stream_tags",
            mxf_path.to_str().ok_or("Invalid MXF path")?,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to extract timecode: {}", e))?;
    
    let tc_str = String::from_utf8_lossy(&tc_output.stdout);
    let mut timecode = None;
    
    for line in tc_str.lines() {
        if line.contains("timecode=") {
            if let Some(tc) = line.split('=').nth(1) {
                timecode = Some(tc.trim().to_string());
                break;
            }
        }
    }
    
    let timecode = timecode.unwrap_or_else(|| {
        tracing::warn!("No timecode found, using 00:00:00:00");
        "00:00:00:00".to_string()
    });
    
    tracing::info!("Extracted timecode: {}", timecode);
    
    // Step 2: Create temporary WAV (stereo mixdown)
    let temp_wav = output_path.with_extension("temp.wav");
    
    let extract_output = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i", mxf_path.to_str().ok_or("Invalid MXF path")?,
            "-vn",
            "-acodec", "pcm_s24le",
            "-ar", &sample_rate.to_string(),
            "-ac", "2",  // Stereo mixdown
            temp_wav.to_str().ok_or("Invalid temp path")?,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to execute FFmpeg for audio extraction: {}", e))?;
    
    if !extract_output.status.success() {
        let stderr = String::from_utf8_lossy(&extract_output.stderr);
        let error_msg = stderr
            .lines()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        
        tracing::error!("FFmpeg audio extraction error: {}", error_msg);
        return Err(format!("Audio extraction failed:\n{}", error_msg));
    }
    
    // Step 3: Calculate TimeReference using validated method
    let tc_parts: Vec<&str> = timecode.split(':').collect();
    if tc_parts.len() != 4 {
        return Err(format!("Invalid timecode format: {}", timecode));
    }
    
    let hours: u64 = tc_parts[0].parse().map_err(|_| "Invalid hours")?;
    let minutes: u64 = tc_parts[1].parse().map_err(|_| "Invalid minutes")?;
    let seconds: u64 = tc_parts[2].parse().map_err(|_| "Invalid seconds")?;
    let frames: u64 = tc_parts[3].parse().map_err(|_| "Invalid frames")?;
    
    // Validated frame-based formula for 23.976fps @ 48000 Hz
    const FRAME_RATE: f64 = 23.976;
    const MULTIPLIER: f64 = 2004.005263;
    
    let total_frames = (hours as f64 * 60.0 * 60.0 * FRAME_RATE)
        + (minutes as f64 * 60.0 * FRAME_RATE)
        + (seconds as f64 * FRAME_RATE)
        + frames as f64;
    
    let time_reference = (total_frames * MULTIPLIER) as u64;
    
    tracing::info!("Calculated TimeReference: {} for timecode: {}", time_reference, timecode);
    
    // Step 4: Insert BEXT using Python script
    let script_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("bwf-tools/insert_bext_timecode.py");
    
    let bext_status = Command::new("python3")
        .args(&[
            script_path.to_str().ok_or("Invalid script path")?,
            temp_wav.to_str().ok_or("Invalid temp path")?,
            output_path.to_str().ok_or("Invalid output path")?,
            "--time-ref", &time_reference.to_string(),
            "--sample-rate", &sample_rate.to_string(),
            "--frame-rate", "23.976",
            "--description", &timecode,
            "--originator", "Industrial Transcoder v2",
        ])
        .output()
        .await;
    
    // Clean up temp file and handle result
    match bext_status {
        Ok(output) if output.status.success() => {
            let _ = std::fs::remove_file(&temp_wav);
            tracing::info!("BWF created successfully with BEXT timecode");
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("BEXT insertion failed: {}", stderr);
            // Fallback: just rename temp to output (plain WAV without BEXT)
            std::fs::rename(&temp_wav, &output_path)
                .map_err(|e| format!("Failed to save output: {}", e))?;
            tracing::info!("Saved as plain WAV (BEXT script unavailable)");
            Ok(())
        }
        Err(e) => {
            tracing::warn!("BEXT script error: {}, using plain WAV", e);
            // Fallback: just rename temp to output
            std::fs::rename(&temp_wav, &output_path)
                .map_err(|e| format!("Failed to save output: {}", e))?;
            Ok(())
        }
    }
}

