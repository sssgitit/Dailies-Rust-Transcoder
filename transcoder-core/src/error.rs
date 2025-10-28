//! Error types for the transcoder

use thiserror::Error;

pub type TranscodeResult<T> = Result<T, TranscodeError>;

#[derive(Error, Debug)]
pub enum TranscodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("FFmpeg not found in system PATH")]
    FfmpegNotFound,

    #[error("FFmpeg execution failed: {0}")]
    FfmpegFailed(String),

    #[error("Invalid input file: {0}")]
    InvalidInput(String),

    #[error("Invalid output path: {0}")]
    InvalidOutput(String),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Job already exists: {0}")]
    JobAlreadyExists(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Worker pool error: {0}")]
    WorkerPoolError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Job cancelled")]
    Cancelled,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

