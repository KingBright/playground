//! Agent模块
//!
//! Agent实现（Local、Universal、Oracle）

pub mod local;
pub mod oracle;
pub mod universal;

pub use local::{create_chess_player, create_debate_participant, LocalAgent, LocalAgentConfig};
pub use oracle::{OracleProtocol, OracleRequest, OracleResponse, RequestPriority};
pub use universal::{
    create_knowledge_extractor, create_text_analyzer, UniversalAgent, UniversalAgentConfig,
};

use common::{Error, Result};

/// Agent配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub description: Option<String>,
}

/// Agent行为trait
pub trait AgentBehavior: Send + Sync {
    fn execute(&self) -> Result<AgentResult>;
}

/// Agent上下文
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentContext {
    pub session_id: uuid::Uuid,
    pub state: serde_json::Value,
}

/// Agent动作
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentAction {
    pub action_type: String,
    pub parameters: serde_json::Value,
}

/// Agent结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error: Option<String>,
}

/// Agent错误
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("Not authorized: {0}")]
    NotAuthorized(String),
}

impl From<AgentError> for Error {
    fn from(err: AgentError) -> Self {
        Error::AgentExecutionFailed(err.to_string())
    }
}
