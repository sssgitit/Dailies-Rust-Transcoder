//! Progress reporting and event handling

use crate::job::JobId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Progress event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    JobStarted {
        job_id: JobId,
        input_path: String,
        output_path: String,
    },
    JobProgress {
        job_id: JobId,
        progress: f32,
        fps: Option<f32>,
        eta_seconds: Option<u64>,
    },
    JobCompleted {
        job_id: JobId,
        duration_seconds: u64,
    },
    JobFailed {
        job_id: JobId,
        error: String,
    },
    JobCancelled {
        job_id: JobId,
    },
    QueueUpdated {
        pending_count: usize,
        running_count: usize,
        completed_count: usize,
    },
}

/// Progress reporter using broadcast channels
#[derive(Clone)]
pub struct ProgressReporter {
    sender: Arc<broadcast::Sender<ProgressEvent>>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Send a progress event
    pub fn report(&self, event: ProgressEvent) {
        // Ignore send errors (no receivers listening)
        let _ = self.sender.send(event);
    }

    /// Subscribe to progress events
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressEvent> {
        self.sender.subscribe()
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_reporter() {
        let reporter = ProgressReporter::new(10);
        let mut receiver = reporter.subscribe();

        let job_id = uuid::Uuid::new_v4();
        
        reporter.report(ProgressEvent::JobStarted {
            job_id,
            input_path: "input.mxf".to_string(),
            output_path: "output.mov".to_string(),
        });

        match receiver.recv().await {
            Ok(ProgressEvent::JobStarted { job_id: id, .. }) => {
                assert_eq!(id, job_id);
            }
            _ => panic!("Expected JobStarted event"),
        }
    }

    #[test]
    fn test_subscriber_count() {
        let reporter = ProgressReporter::new(10);
        assert_eq!(reporter.subscriber_count(), 0);

        let _sub1 = reporter.subscribe();
        assert_eq!(reporter.subscriber_count(), 1);

        let _sub2 = reporter.subscribe();
        assert_eq!(reporter.subscriber_count(), 2);
    }
}

