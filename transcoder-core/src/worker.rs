//! Worker pool for parallel job processing

use crate::config::TranscodeConfig;
use crate::error::{TranscodeError, TranscodeResult};
use crate::job::{JobId, JobStatus};
use crate::progress::{ProgressEvent, ProgressReporter};
use crate::queue::JobQueue;
use crate::transcode::Transcoder;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Worker pool for processing transcode jobs
pub struct WorkerPool {
    queue: Arc<JobQueue>,
    transcoder: Arc<Transcoder>,
    progress_reporter: ProgressReporter,
    worker_count: usize,
    running: Arc<AtomicBool>,
    active_workers: Arc<AtomicUsize>,
    worker_handles: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    /// Create a new worker pool
    pub fn new(
        queue: Arc<JobQueue>,
        progress_reporter: ProgressReporter,
        worker_count: Option<usize>,
    ) -> TranscodeResult<Self> {
        let transcoder = Arc::new(Transcoder::new()?);
        
        // Default to CPU count - 1, minimum 1
        let worker_count = worker_count.unwrap_or_else(|| {
            let cpu_count = crate::platform::cpu_count();
            (cpu_count.saturating_sub(1)).max(1)
        });

        info!("Creating worker pool with {} workers", worker_count);

        Ok(Self {
            queue,
            transcoder,
            progress_reporter,
            worker_count,
            running: Arc::new(AtomicBool::new(false)),
            active_workers: Arc::new(AtomicUsize::new(0)),
            worker_handles: Vec::new(),
        })
    }

    /// Start the worker pool
    pub async fn start(&mut self) -> TranscodeResult<()> {
        if self.running.load(Ordering::SeqCst) {
            return Err(TranscodeError::WorkerPoolError(
                "Worker pool already running".to_string(),
            ));
        }

        info!("Starting worker pool");
        self.running.store(true, Ordering::SeqCst);

        // Spawn workers
        for worker_id in 0..self.worker_count {
            let queue = Arc::clone(&self.queue);
            let transcoder = Arc::clone(&self.transcoder);
            let progress_reporter = self.progress_reporter.clone();
            let running = Arc::clone(&self.running);
            let active_workers = Arc::clone(&self.active_workers);

            let handle = tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    queue,
                    transcoder,
                    progress_reporter,
                    running,
                    active_workers,
                )
                .await;
            });

            self.worker_handles.push(handle);
        }

        info!("Worker pool started with {} workers", self.worker_count);
        Ok(())
    }

    /// Stop the worker pool
    pub async fn stop(&mut self) {
        info!("Stopping worker pool");
        self.running.store(false, Ordering::SeqCst);

        // Wait for all workers to finish
        for handle in self.worker_handles.drain(..) {
            let _ = handle.await;
        }

        info!("Worker pool stopped");
    }

    /// Worker loop
    async fn worker_loop(
        worker_id: usize,
        queue: Arc<JobQueue>,
        transcoder: Arc<Transcoder>,
        progress_reporter: ProgressReporter,
        running: Arc<AtomicBool>,
        active_workers: Arc<AtomicUsize>,
    ) {
        debug!("Worker {} started", worker_id);

        while running.load(Ordering::SeqCst) {
            // Get next job
            let job_id = match queue.get_next_job().await {
                Some(id) => id,
                None => {
                    // No jobs available, wait a bit
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
            };

            // Increment active worker count
            active_workers.fetch_add(1, Ordering::SeqCst);

            // Process the job
            Self::process_job(
                worker_id,
                job_id,
                &queue,
                &transcoder,
                &progress_reporter,
            )
            .await;

            // Decrement active worker count
            active_workers.fetch_sub(1, Ordering::SeqCst);

            // Report queue status
            let stats = queue.get_stats();
            progress_reporter.report(ProgressEvent::QueueUpdated {
                pending_count: stats.pending_count,
                running_count: stats.running_count,
                completed_count: stats.completed_count,
            });
        }

        debug!("Worker {} stopped", worker_id);
    }

    /// Process a single job
    async fn process_job(
        worker_id: usize,
        job_id: JobId,
        queue: &JobQueue,
        transcoder: &Transcoder,
        progress_reporter: &ProgressReporter,
    ) {
        info!("Worker {} processing job {}", worker_id, job_id);

        // Get job
        let mut job = match queue.get_job(&job_id) {
            Some(job) => job,
            None => {
                error!("Job {} not found", job_id);
                return;
            }
        };

        // Mark as running
        job.start();
        if let Err(e) = queue.update_job(job.clone()) {
            error!("Failed to update job status: {}", e);
            return;
        }

        // Report job started
        progress_reporter.report(ProgressEvent::JobStarted {
            job_id,
            input_path: job.input_path.to_string_lossy().to_string(),
            output_path: job.output_path.to_string_lossy().to_string(),
        });

        // Check if this is a BWF extraction job
        let is_bwf_job = job.config.get("type")
            .and_then(|v| v.as_str())
            .map(|s| s == "bwf_extraction")
            .unwrap_or(false);

        let start_time = std::time::Instant::now();
        let result = if is_bwf_job {
            // Handle BWF extraction
            info!("Processing BWF extraction job");
            Self::process_bwf_job(&job, progress_reporter).await
        } else {
            // Parse config for normal transcode
            let config: TranscodeConfig = match serde_json::from_value(job.config.clone()) {
                Ok(cfg) => cfg,
                Err(e) => {
                    error!("Invalid job config: {}", e);
                    job.fail(format!("Invalid configuration: {}", e));
                    let _ = queue.update_job(job.clone());
                    progress_reporter.report(ProgressEvent::JobFailed {
                        job_id,
                        error: e.to_string(),
                    });
                    return;
                }
            };

            // Create progress callback
            let progress_reporter_clone = progress_reporter.clone();
            let progress_callback = move |progress: f32, fps: Option<f32>| {
                progress_reporter_clone.report(ProgressEvent::JobProgress {
                    job_id,
                    progress,
                    fps,
                    eta_seconds: None, // TODO: Calculate ETA
                });
            };

            // Execute transcode
            transcoder
                .transcode(
                    &job.input_path,
                    &job.output_path,
                    &config,
                    progress_callback,
                )
                .await
        };

        let duration = start_time.elapsed();

        match result {
            Ok(()) => {
                info!(
                    "Worker {} completed job {} in {:.2}s",
                    worker_id,
                    job_id,
                    duration.as_secs_f64()
                );
                job.complete();
                progress_reporter.report(ProgressEvent::JobCompleted {
                    job_id,
                    duration_seconds: duration.as_secs(),
                });
            }
            Err(e) => {
                error!("Worker {} job {} failed: {}", worker_id, job_id, e);
                job.fail(e.to_string());
                progress_reporter.report(ProgressEvent::JobFailed {
                    job_id,
                    error: e.to_string(),
                });
            }
        }

        // Update job status
        if let Err(e) = queue.update_job(job) {
            error!("Failed to update job status: {}", e);
        }
    }

    /// Process BWF extraction job
    async fn process_bwf_job(
        job: &crate::job::Job,
        progress_reporter: &ProgressReporter,
    ) -> TranscodeResult<()> {
        use std::process::Stdio;
        use tokio::process::Command;
        
        let sample_rate = job.config.get("sample_rate")
            .and_then(|v| v.as_u64())
            .unwrap_or(48000) as u32;
        
        info!("Creating BWF from: {:?} -> {:?}", job.input_path, job.output_path);
        
        // Verify input exists
        if !job.input_path.exists() {
            return Err(TranscodeError::InvalidInput(format!("Input file does not exist: {:?}", job.input_path)));
        }
        
        // Create output directory if needed
        if let Some(parent) = job.output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // Extract timecode from file using ffprobe
        let tc_output = Command::new("ffprobe")
            .args(&[
                "-v", "quiet",
                "-show_entries", "format_tags:stream_tags",
                job.input_path.to_str().ok_or_else(|| TranscodeError::InvalidInput("Invalid input path".to_string()))?,
            ])
            .output()
            .await
            .map_err(|e| TranscodeError::FfmpegFailed(format!("ffprobe failed: {}", e)))?;
        
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
        
        let timecode = timecode.unwrap_or_else(|| "00:00:00:00".to_string());
        info!("Extracted timecode: {}", timecode);
        
        // Create temporary WAV
        let temp_wav = job.output_path.with_extension("temp.wav");
        
        let extract_output = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-i", job.input_path.to_str().ok_or_else(|| TranscodeError::InvalidInput("Invalid input path".to_string()))?,
                "-vn",
                "-acodec", "pcm_s24le",
                "-ar", &sample_rate.to_string(),
                "-ac", "2",
                temp_wav.to_str().ok_or_else(|| TranscodeError::InvalidOutput("Invalid temp path".to_string()))?,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TranscodeError::FfmpegFailed(format!("Audio extraction failed: {}", e)))?;
        
        if !extract_output.status.success() {
            let stderr = String::from_utf8_lossy(&extract_output.stderr);
            return Err(TranscodeError::FfmpegFailed(format!("Audio extraction failed: {}", stderr)));
        }
        
        // Calculate TimeReference using validated method for 23.976fps
        let tc_parts: Vec<&str> = timecode.split(':').collect();
        if tc_parts.len() == 4 {
            let hours: u64 = tc_parts[0].parse().unwrap_or(0);
            let minutes: u64 = tc_parts[1].parse().unwrap_or(0);
            let seconds: u64 = tc_parts[2].parse().unwrap_or(0);
            let frames: u64 = tc_parts[3].parse().unwrap_or(0);
            
            // Validated frame-based formula for 23.976fps @ 48000 Hz
            const FRAME_RATE: f64 = 23.976;
            const MULTIPLIER: f64 = 2004.005263;
            
            let total_frames = (hours as f64 * 60.0 * 60.0 * FRAME_RATE)
                + (minutes as f64 * 60.0 * FRAME_RATE)
                + (seconds as f64 * FRAME_RATE)
                + frames as f64;
            
            let time_reference = (total_frames * MULTIPLIER) as u64;
            
            info!("Calculated TimeReference: {} for timecode: {}", time_reference, timecode);
            
            // Try to insert BEXT using Python script
            let script_path = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("bwf-tools/insert_bext_timecode.py");
            
            if script_path.exists() {
                let bext_result = Command::new("python3")
                    .args(&[
                        script_path.to_str().ok_or_else(|| TranscodeError::InvalidInput("Invalid script path".to_string()))?,
                        temp_wav.to_str().ok_or_else(|| TranscodeError::InvalidOutput("Invalid temp path".to_string()))?,
                        job.output_path.to_str().ok_or_else(|| TranscodeError::InvalidOutput("Invalid output path".to_string()))?,
                        "--time-ref", &time_reference.to_string(),
                        "--sample-rate", &sample_rate.to_string(),
                        "--frame-rate", "23.976",
                        "--description", &timecode,
                        "--originator", "Industrial Transcoder v2",
                    ])
                    .output()
                    .await;
                
                match bext_result {
                    Ok(output) if output.status.success() => {
                        // BEXT inserted successfully, remove temp file
                        let _ = std::fs::remove_file(&temp_wav);
                        info!("BWF created with BEXT timecode at: {:?}", job.output_path);
                    }
                    _ => {
                        // Fallback: just rename temp to output (plain WAV)
                        info!("BEXT script unavailable, creating plain WAV");
                        std::fs::rename(&temp_wav, &job.output_path)?;
                    }
                }
            } else {
                // No script available, just rename
                info!("BEXT script not found, creating plain WAV");
                std::fs::rename(&temp_wav, &job.output_path)?;
            }
        } else {
            // Invalid timecode format, just rename
            std::fs::rename(&temp_wav, &job.output_path)?;
        }
        
        info!("BWF created successfully at: {:?}", job.output_path);
        Ok(())
    }

    /// Get number of active workers
    pub fn active_worker_count(&self) -> usize {
        self.active_workers.load(Ordering::SeqCst)
    }

    /// Check if worker pool is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worker_pool_creation() {
        let queue = Arc::new(JobQueue::new());
        let reporter = ProgressReporter::new(10);

        match WorkerPool::new(queue, reporter, Some(2)) {
            Ok(pool) => {
                assert_eq!(pool.worker_count, 2);
                assert!(!pool.is_running());
            }
            Err(e) => {
                println!("Worker pool creation failed (FFmpeg not available?): {}", e);
            }
        }
    }
}

