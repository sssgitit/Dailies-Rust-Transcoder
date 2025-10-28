//! Job management and status tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

pub type JobId = Uuid;

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Priority level for jobs
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Transcode job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub status: JobStatus,
    pub priority: Priority,
    pub progress: f32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub config: serde_json::Value, // TranscodeConfig as JSON
}

impl Job {
    /// Create a new pending job
    pub fn new(
        input_path: PathBuf,
        output_path: PathBuf,
        config: serde_json::Value,
        priority: Priority,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            input_path,
            output_path,
            status: JobStatus::Pending,
            priority,
            progress: 0.0,
            error_message: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            config,
        }
    }

    /// Mark job as running
    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
        self.progress = 0.0;
    }

    /// Update job progress (0.0 - 100.0)
    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 100.0);
    }

    /// Mark job as completed
    pub fn complete(&mut self) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.progress = 100.0;
    }

    /// Mark job as failed with error message
    pub fn fail(&mut self, error: String) {
        self.status = JobStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error);
    }

    /// Mark job as cancelled
    pub fn cancel(&mut self) {
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    /// Get job duration in seconds
    pub fn duration_seconds(&self) -> Option<i64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some((end - start).num_seconds()),
            _ => None,
        }
    }

    /// Check if job is finished (completed, failed, or cancelled)
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    /// Check if job is active (pending or running)
    pub fn is_active(&self) -> bool {
        matches!(self.status, JobStatus::Pending | JobStatus::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_lifecycle() {
        let mut job = Job::new(
            PathBuf::from("/input.mxf"),
            PathBuf::from("/output.mov"),
            serde_json::json!({}),
            Priority::Normal,
        );

        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.is_active());
        assert!(!job.is_finished());

        job.start();
        assert_eq!(job.status, JobStatus::Running);
        assert!(job.started_at.is_some());

        job.update_progress(50.0);
        assert_eq!(job.progress, 50.0);

        job.complete();
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.is_finished());
        assert_eq!(job.progress, 100.0);
    }

    #[test]
    fn test_job_failure() {
        let mut job = Job::new(
            PathBuf::from("/input.mxf"),
            PathBuf::from("/output.mov"),
            serde_json::json!({}),
            Priority::Normal,
        );

        job.start();
        job.fail("Test error".to_string());

        assert_eq!(job.status, JobStatus::Failed);
        assert!(job.is_finished());
        assert_eq!(job.error_message, Some("Test error".to_string()));
    }
}

