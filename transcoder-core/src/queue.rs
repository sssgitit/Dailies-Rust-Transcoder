//! Job queue management with priority support

use crate::error::{TranscodeError, TranscodeResult};
use crate::job::{Job, JobId, JobStatus, Priority};
use dashmap::DashMap;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Wrapper for priority queue ordering
#[derive(Clone)]
struct PriorityJob {
    job_id: JobId,
    priority: Priority,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl PartialEq for PriorityJob {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for PriorityJob {}

impl PartialOrd for PriorityJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityJob {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older jobs first
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => other.created_at.cmp(&self.created_at), // Reverse for FIFO
            other => other,
        }
    }
}

/// Thread-safe job queue with priority support
pub struct JobQueue {
    jobs: Arc<DashMap<JobId, Job>>,
    pending_queue: Arc<RwLock<BinaryHeap<PriorityJob>>>,
}

impl JobQueue {
    /// Create a new job queue
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(DashMap::new()),
            pending_queue: Arc::new(RwLock::new(BinaryHeap::new())),
        }
    }

    /// Add a job to the queue
    pub async fn add_job(&self, mut job: Job) -> TranscodeResult<JobId> {
        let job_id = job.id;

        // Check if job already exists
        if self.jobs.contains_key(&job_id) {
            return Err(TranscodeError::JobAlreadyExists(job_id.to_string()));
        }

        // Validate paths
        if !job.input_path.exists() {
            return Err(TranscodeError::InvalidInput(format!(
                "Input file does not exist: {:?}",
                job.input_path
            )));
        }

        // Set status to pending
        job.status = JobStatus::Pending;

        // Add to pending queue
        let priority_job = PriorityJob {
            job_id,
            priority: job.priority,
            created_at: job.created_at,
        };

        self.pending_queue.write().await.push(priority_job);

        // Store job
        self.jobs.insert(job_id, job);

        Ok(job_id)
    }

    /// Get next pending job (highest priority)
    pub async fn get_next_job(&self) -> Option<JobId> {
        let mut queue = self.pending_queue.write().await;
        
        while let Some(priority_job) = queue.pop() {
            let job_id = priority_job.job_id;
            
            // Check if job still exists and is pending
            if let Some(job) = self.jobs.get(&job_id) {
                if job.status == JobStatus::Pending {
                    return Some(job_id);
                }
            }
        }

        None
    }

    /// Get a job by ID
    pub fn get_job(&self, job_id: &JobId) -> Option<Job> {
        self.jobs.get(job_id).map(|entry| entry.clone())
    }

    /// Update a job
    pub fn update_job(&self, job: Job) -> TranscodeResult<()> {
        let job_id = job.id;
        
        if !self.jobs.contains_key(&job_id) {
            return Err(TranscodeError::JobNotFound(job_id.to_string()));
        }

        self.jobs.insert(job_id, job);
        Ok(())
    }

    /// Remove a job from the queue
    pub fn remove_job(&self, job_id: &JobId) -> TranscodeResult<Job> {
        self.jobs
            .remove(job_id)
            .map(|(_, job)| job)
            .ok_or_else(|| TranscodeError::JobNotFound(job_id.to_string()))
    }

    /// Cancel a job
    pub async fn cancel_job(&self, job_id: &JobId) -> TranscodeResult<()> {
        if let Some(mut job_ref) = self.jobs.get_mut(job_id) {
            job_ref.cancel();
            Ok(())
        } else {
            Err(TranscodeError::JobNotFound(job_id.to_string()))
        }
    }

    /// Get all jobs
    pub fn get_all_jobs(&self) -> Vec<Job> {
        self.jobs
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get jobs by status
    pub fn get_jobs_by_status(&self, status: JobStatus) -> Vec<Job> {
        self.jobs
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get queue statistics
    pub fn get_stats(&self) -> QueueStats {
        let mut stats = QueueStats::default();

        for entry in self.jobs.iter() {
            match entry.value().status {
                JobStatus::Pending => stats.pending_count += 1,
                JobStatus::Running => stats.running_count += 1,
                JobStatus::Completed => stats.completed_count += 1,
                JobStatus::Failed => stats.failed_count += 1,
                JobStatus::Cancelled => stats.cancelled_count += 1,
            }
        }

        stats.total_count = self.jobs.len();
        stats
    }

    /// Clear completed jobs
    pub fn clear_completed(&self) -> usize {
        let completed_ids: Vec<JobId> = self.jobs
            .iter()
            .filter(|entry| {
                matches!(
                    entry.value().status,
                    JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
                )
            })
            .map(|entry| *entry.key())
            .collect();

        let count = completed_ids.len();
        for id in completed_ids {
            self.jobs.remove(&id);
        }

        count
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue statistics
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct QueueStats {
    pub total_count: usize,
    pub pending_count: usize,
    pub running_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub cancelled_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_job(priority: Priority) -> Job {
        Job::new(
            PathBuf::from("/tmp/test_input.mxf"),
            PathBuf::from("/tmp/test_output.mov"),
            serde_json::json!({}),
            priority,
        )
    }

    #[tokio::test]
    async fn test_queue_basic_operations() {
        let queue = JobQueue::new();
        let job = create_test_job(Priority::Normal);
        let job_id = job.id;

        // This will fail because the input file doesn't exist, which is expected
        match queue.add_job(job).await {
            Err(TranscodeError::InvalidInput(_)) => {
                // Expected error - input file doesn't exist
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_queue_stats() {
        let queue = JobQueue::new();
        let stats = queue.get_stats();

        assert_eq!(stats.total_count, 0);
        assert_eq!(stats.pending_count, 0);
    }

    #[test]
    fn test_priority_ordering() {
        let low = PriorityJob {
            job_id: uuid::Uuid::new_v4(),
            priority: Priority::Low,
            created_at: chrono::Utc::now(),
        };

        let high = PriorityJob {
            job_id: uuid::Uuid::new_v4(),
            priority: Priority::High,
            created_at: chrono::Utc::now(),
        };

        assert!(high > low);
    }
}

