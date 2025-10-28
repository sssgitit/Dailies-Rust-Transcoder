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

        // Parse config
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
        let start_time = std::time::Instant::now();
        let result = transcoder
            .transcode(
                &job.input_path,
                &job.output_path,
                &config,
                progress_callback,
            )
            .await;

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

