//! Engine System - Agent仿真运行时
//!
//! 基于ECS (Entity Component System) 模式的Agent模拟引擎

pub mod agent;
pub mod api;
pub mod environment;
pub mod session;
pub mod workflow;

// Re-exports
pub use environment::{
    Environment, EnvironmentError, EnvironmentSchema, EnvironmentState, RendererScript,
    StateValidator,
};

pub use session::{
    Session, SessionConfig, SessionError, SessionManager, SessionSnapshot, SessionStatus,
};

pub use agent::{
    create_chess_player, create_debate_participant, LocalAgent, LocalAgentConfig, OracleProtocol,
    OracleRequest, OracleResponse, RequestPriority,
};

pub use workflow::{
    ExecutionResult, SandboxLimits, WorkflowConfig, WorkflowEngine, WorkflowError, WorkflowResult,
    WorkflowScript, WorkflowStep,
};

pub use api::{EngineApi, EngineApiState};

/// Engine版本
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Engine初始化
pub fn init() -> common::Result<()> {
    tracing::info!("Engine v{} initializing", ENGINE_VERSION);
    Ok(())
}
