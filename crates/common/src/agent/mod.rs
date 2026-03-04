//! Core Agent trait and types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for an agent
pub type AgentId = String;

/// Input to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// The input data (text, JSON, etc.)
    pub data: serde_json::Value,

    /// Optional context from environment or other agents
    pub context: HashMap<String, serde_json::Value>,

    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

impl AgentInput {
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            data,
            context: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.insert(key, value);
        self
    }
}

/// Output from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// The output data
    pub data: serde_json::Value,

    /// Optional metadata
    pub metadata: HashMap<String, String>,

    /// Whether the agent needs help from another agent (Oracle Protocol)
    pub need_help: Option<OracleRequest>,
}

/// Oracle Protocol request - when an agent needs help from another agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleRequest {
    /// The target agent to ask for help
    pub target_agent: String,

    /// The query/prompt to send to the target agent
    pub query: String,
}

/// Capabilities that an agent can provide
pub type AgentCapabilities = Vec<String>;

/// Core trait that all agents must implement
#[async_trait]
pub trait Agent: Send + Sync {
    /// Invoke the agent with input and produce output
    async fn invoke(&self, input: AgentInput) -> crate::Result<AgentOutput>;

    /// Get the agent's name/identifier
    fn name(&self) -> &str;

    /// Get the agent's capabilities
    fn capabilities(&self) -> AgentCapabilities;

    /// Inject context into the agent (for Knowledge Slice mounting)
    fn inject_context(&mut self, context: serde_json::Value) {
        // Default implementation does nothing
        let _ = context;
    }
}
