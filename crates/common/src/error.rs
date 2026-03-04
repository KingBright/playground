//! Unified error types for all crates

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Invalid agent input: {0}")]
    InvalidInput(String),

    #[error("Agent execution failed: {0}")]
    AgentExecutionFailed(String),

    #[error("Memory operation failed: {0}")]
    MemoryError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Environment error: {0}")]
    EnvironmentError(String),

    #[error("Workflow error: {0}")]
    WorkflowError(String),

    #[error("Step failed: {step:?} - {cause:?}")]
    StepFailed { step: String, cause: Box<Error> },

    #[error("Sandbox error: {0}")]
    SandboxError(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Engine error: {0}")]
    EngineError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

// Allow conversion from Box<Error>
impl From<Box<Error>> for Error {
    fn from(err: Box<Error>) -> Self {
        *err
    }
}
