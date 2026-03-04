//! Workflow模块
//!
//! Workflow引擎和脚本执行

pub mod engine;

pub use engine::{
    ExecutionResult, SandboxLimits, StepResult, StepStatus, WorkflowConfig, WorkflowEngine,
    WorkflowResult, WorkflowScript, WorkflowStep,
};

use common::{Error, Result};
use serde::{Deserialize, Serialize};

/// Workflow错误
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Script execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    #[error("Timeout")]
    Timeout,

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
}

impl From<WorkflowError> for Error {
    fn from(err: WorkflowError) -> Self {
        Error::WorkflowError(err.to_string())
    }
}
