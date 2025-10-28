//! Tauri commands for MXF operations

use crate::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use transcoder_core::mxf::*;

/// MXF rewrap request
#[derive(Debug, Deserialize)]
pub struct MxfRewrapRequest {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub target_wrapping: MxfWrapping,
}

/// Batch rewrap request
#[derive(Debug, Deserialize)]
pub struct BatchRewrapRequest {
    pub files: Vec<(PathBuf, PathBuf)>,
    pub target_wrapping: MxfWrapping,
}

/// Detect MXF wrapping type
#[tauri::command]
pub async fn detect_mxf_wrapping(input_path: String) -> Result<MxfWrapping, String> {
    let rewrapper = MxfRewrapper::new();
    
    if !rewrapper.is_available() {
        return Err("bmxtranswrap not found - please install BMX tools".to_string());
    }
    
    let path = PathBuf::from(input_path);
    rewrapper
        .detect_wrapping(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Rewrap MXF file (clip-wrapped ↔ frame-wrapped)
#[tauri::command]
pub async fn rewrap_mxf(
    request: MxfRewrapRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let rewrapper = MxfRewrapper::new();
    
    if !rewrapper.is_available() {
        return Err("bmxtranswrap not found - please install BMX tools".to_string());
    }
    
    // Create progress callback
    let progress_reporter = state.progress_reporter.clone();
    let job_id = uuid::Uuid::new_v4();
    
    let progress_callback = move |progress: f32| {
        progress_reporter.report(transcoder_core::ProgressEvent::JobProgress {
            job_id,
            progress,
            fps: None,
            eta_seconds: None,
        });
    };
    
    rewrapper
        .rewrap(
            &request.input_path,
            &request.output_path,
            request.target_wrapping,
            progress_callback,
        )
        .await
        .map_err(|e| e.to_string())
}

/// Convert clip-wrapped to frame-wrapped
#[tauri::command]
pub async fn clip_to_frame(
    input_path: String,
    output_path: String,
) -> Result<(), String> {
    let rewrapper = MxfRewrapper::new();
    
    if !rewrapper.is_available() {
        return Err("bmxtranswrap not found - please install BMX tools".to_string());
    }
    
    let input = PathBuf::from(input_path);
    let output = PathBuf::from(output_path);
    
    rewrapper
        .clip_to_frame(&input, &output, |_| {})
        .await
        .map_err(|e| e.to_string())
}

/// Convert frame-wrapped to clip-wrapped
#[tauri::command]
pub async fn frame_to_clip(
    input_path: String,
    output_path: String,
) -> Result<(), String> {
    let rewrapper = MxfRewrapper::new();
    
    if !rewrapper.is_available() {
        return Err("bmxtranswrap not found - please install BMX tools".to_string());
    }
    
    let input = PathBuf::from(input_path);
    let output = PathBuf::from(output_path);
    
    rewrapper
        .frame_to_clip(&input, &output, |_| {})
        .await
        .map_err(|e| e.to_string())
}

/// Batch rewrap multiple files
#[tauri::command]
pub async fn batch_rewrap_mxf(
    request: BatchRewrapRequest,
    state: State<'_, AppState>,
) -> Result<Vec<Result<(), String>>, String> {
    let rewrapper = MxfRewrapper::new();
    
    if !rewrapper.is_available() {
        return Err("bmxtranswrap not found - please install BMX tools".to_string());
    }
    
    let progress_reporter = state.progress_reporter.clone();
    
    let progress_callback = move |current: usize, total: usize, progress: f32| {
        let overall_progress = ((current as f32 - 1.0 + progress / 100.0) / total as f32) * 100.0;
        
        progress_reporter.report(transcoder_core::ProgressEvent::QueueUpdated {
            pending_count: total - current,
            running_count: 1,
            completed_count: current - 1,
        });
    };
    
    let results = rewrapper
        .batch_rewrap(&request.files, request.target_wrapping, progress_callback)
        .await
        .map_err(|e| e.to_string())?;
    
    // Convert results to string errors
    Ok(results
        .into_iter()
        .map(|r| r.map_err(|e| e.to_string()))
        .collect())
}

/// Check if MXF rewrapping is available
#[tauri::command]
pub fn is_mxf_rewrapping_available() -> bool {
    MxfRewrapper::new().is_available()
}

