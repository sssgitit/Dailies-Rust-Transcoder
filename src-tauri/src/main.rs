// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod mxf_commands;
mod transcoder_commands;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use transcoder_core::*;
use mxf_commands::*;
use transcoder_commands::*;

/// Application state
pub struct AppState {
    pub queue: Arc<JobQueue>,
    pub worker_pool: Arc<Mutex<Option<WorkerPool>>>,
    pub progress_reporter: ProgressReporter,
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Create application state
    let queue = Arc::new(JobQueue::new());
    let progress_reporter = ProgressReporter::new(1000);

    let state = AppState {
        queue: Arc::clone(&queue),
        worker_pool: Arc::new(Mutex::new(None)),
        progress_reporter: progress_reporter.clone(),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // System commands
            get_system_info,
            verify_ffmpeg,
            // Preset commands
            get_presets,
            // Job commands
            add_job,
            get_job,
            get_all_jobs,
            cancel_job,
            clear_completed_jobs,
            get_queue_stats,
            // Worker pool commands
            start_workers,
            stop_workers,
            get_worker_status,
            // Progress subscription
            subscribe_progress,
            // Simple transcode commands
            transcode_dnxhr_lb,
            create_bwf_from_mxf,
            // MXF commands
            detect_mxf_wrapping,
            rewrap_mxf,
            clip_to_frame,
            frame_to_clip,
            batch_rewrap_mxf,
            is_mxf_rewrapping_available,
        ])
        .setup(|app| {
            // Start progress event emitter
            let app_handle = app.handle();
            let state = app.state::<AppState>();
            let mut receiver = state.progress_reporter.subscribe();

            tauri::async_runtime::spawn(async move {
                while let Ok(event) = receiver.recv().await {
                    let _ = app_handle.emit_all("transcode_progress", &event);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

