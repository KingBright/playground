//! Common types and utilities shared across all crates

pub mod agent;
pub mod config;
pub mod error;
pub mod llm;
pub mod memory;
pub mod testing;

// Re-exports
pub use agent::{Agent, AgentCapabilities, AgentInput, AgentOutput};
pub use error::{Error, Result};
pub use memory::{KnowledgeSlice, MemoryType, SearchResult};
