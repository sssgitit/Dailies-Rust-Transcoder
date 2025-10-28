//! Industrial Transcoder Core
//!
//! Cross-platform, multi-threaded media transcoding engine
//! Supports macOS, Windows, and Linux

pub mod config;
pub mod error;
pub mod job;
pub mod logger;
pub mod mxf;
pub mod platform;
pub mod progress;
pub mod queue;
pub mod transcode;
pub mod worker;

pub use config::{CodecPreset, TranscodeConfig};
pub use error::{TranscodeError, TranscodeResult};
pub use job::{Job, JobId, JobStatus};
pub use logger::{JobLogger, JobLogEntry};
pub use mxf::{MxfMetadata, MxfRewrapper, MxfWrapping};
pub use platform::Platform;
pub use progress::{ProgressEvent, ProgressReporter};
pub use queue::JobQueue;
pub use transcode::Transcoder;
pub use worker::WorkerPool;

/// Initialize the transcoder system with logging
pub fn init() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}

