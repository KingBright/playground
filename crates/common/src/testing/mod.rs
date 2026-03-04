//! Testing framework with mocks, builders, and in-memory implementations
//!
//! This module provides comprehensive testing utilities including:
//! - Mock Agent implementations
//! - Test data builders
//! - In-memory storage backends
//! - Test helpers and fixtures

use crate::agent::{Agent, AgentCapabilities, AgentInput, AgentOutput};
use crate::memory::{KnowledgeSlice, MemoryType, SearchResult, TimeRange};
use crate::{Error, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock Agent for testing
#[derive(Clone)]
pub struct MockAgent {
    pub name: String,
    pub capabilities: AgentCapabilities,
    pub response: Option<serde_json::Value>,
    pub delay_ms: u64,
}

impl MockAgent {
    /// Create a new MockAgent with default settings
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            capabilities: vec!["test".to_string()],
            response: None,
            delay_ms: 0,
        }
    }

    /// Set the response that this agent will return
    pub fn with_response(mut self, response: serde_json::Value) -> Self {
        self.response = Some(response);
        self
    }

    /// Set capabilities for this agent
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.capabilities = capabilities;
        self
    }

    /// Add artificial delay to simulate processing
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    /// Create a simple echo agent that returns input data
    pub fn echo() -> Self {
        Self::new("echo_agent").with_capabilities(vec!["echo".to_string()])
    }

    /// Create a failing agent that always returns an error
    pub fn failing() -> Self {
        Self::new("failing_agent").with_capabilities(vec!["fail".to_string()])
    }
}

#[async_trait]
impl Agent for MockAgent {
    async fn invoke(&self, input: AgentInput) -> Result<AgentOutput> {
        // Add delay if configured
        if self.delay_ms > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
        }

        // Check if this is the failing agent
        if self.name == "failing_agent" {
            return Err(Error::AgentExecutionFailed(
                "Mock agent configured to fail".to_string(),
            ));
        }

        // Echo agent returns input data
        if self.name == "echo_agent" {
            return Ok(AgentOutput {
                data: input.data,
                metadata: HashMap::new(),
                need_help: None,
            });
        }

        // Return configured response or default
        let data = self.response.clone().unwrap_or_else(|| {
            json!({
                "message": "Mock agent response",
                "agent": self.name,
            })
        });

        Ok(AgentOutput {
            data,
            metadata: HashMap::new(),
            need_help: None,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn capabilities(&self) -> AgentCapabilities {
        self.capabilities.clone()
    }
}

/// Builder for constructing test AgentInput
pub struct AgentInputBuilder {
    data: serde_json::Value,
    context: HashMap<String, serde_json::Value>,
    metadata: HashMap<String, String>,
}

impl Default for AgentInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentInputBuilder {
    /// Create a new builder with empty data
    pub fn new() -> Self {
        Self {
            data: json!(null),
            context: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the input data
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Set text input (convenience method)
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.data = json!({ "text": text.into() });
        self
    }

    /// Add a context entry
    pub fn context(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Add multiple context entries
    pub fn context_many(mut self, ctx: HashMap<String, serde_json::Value>) -> Self {
        self.context.extend(ctx);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the AgentInput
    pub fn build(self) -> AgentInput {
        AgentInput {
            data: self.data,
            context: self.context,
            metadata: self.metadata,
        }
    }
}

/// Builder for constructing test KnowledgeSlice
pub struct KnowledgeSliceBuilder {
    id: String,
    time_range: TimeRange,
    tags: Vec<String>,
    description: Option<String>,
}

impl Default for KnowledgeSliceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeSliceBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            time_range: TimeRange {
                start: now - chrono::Duration::hours(24),
                end: now,
            },
            tags: vec![],
            description: None,
        }
    }

    /// Set the slice ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set time range
    pub fn time_range(mut self, start: chrono::DateTime<Utc>, end: chrono::DateTime<Utc>) -> Self {
        self.time_range = TimeRange { start, end };
        self
    }

    /// Add tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add a single tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Build the KnowledgeSlice
    pub fn build(self) -> KnowledgeSlice {
        KnowledgeSlice {
            id: self.id,
            time_range: self.time_range,
            tags: self.tags,
            description: self.description,
        }
    }
}

/// In-memory storage backend for testing
#[derive(Clone, Default)]
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<String, StoredData>>>,
}

#[derive(Clone)]
struct StoredData {
    content: String,
    memory_type: MemoryType,
    metadata: HashMap<String, String>,
    created_at: chrono::DateTime<Utc>,
}

impl InMemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Store data with given key and type
    pub async fn store(
        &self,
        key: impl Into<String>,
        content: impl Into<String>,
        memory_type: MemoryType,
    ) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(
            key.into(),
            StoredData {
                content: content.into(),
                memory_type,
                metadata: HashMap::new(),
                created_at: Utc::now(),
            },
        );
        Ok(())
    }

    /// Store data with metadata
    pub async fn store_with_metadata(
        &self,
        key: impl Into<String>,
        content: impl Into<String>,
        memory_type: MemoryType,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(
            key.into(),
            StoredData {
                content: content.into(),
                memory_type,
                metadata,
                created_at: Utc::now(),
            },
        );
        Ok(())
    }

    /// Retrieve data by key
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let data = self.data.read().await;
        Ok(data.get(key).map(|d| d.content.clone()))
    }

    /// Search across all stored data (simple text matching)
    pub async fn search(
        &self,
        query: &str,
        memory_type: Option<MemoryType>,
    ) -> Result<Vec<SearchResult>> {
        let data = self.data.read().await;
        let query_lower = query.to_lowercase();

        let results: Vec<SearchResult> = data
            .iter()
            .filter(|(_, d)| {
                // Filter by memory type if specified
                if let Some(mt) = memory_type {
                    if d.memory_type != mt {
                        return false;
                    }
                }
                // Simple text matching
                d.content.to_lowercase().contains(&query_lower)
            })
            .map(|(_key, d)| SearchResult {
                content: d.content.clone(),
                score: 0.8, // Fixed score for testing
                metadata: d.metadata.clone(),
                related_entities: vec![],
            })
            .collect();

        Ok(results)
    }

    /// Clear all stored data
    pub async fn clear(&self) -> Result<()> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }

    /// Get count of stored items
    pub async fn count(&self) -> usize {
        self.data.read().await.len()
    }
}

/// Test fixtures and helpers
pub mod fixtures {
    use super::*;

    /// Create a test agent input with text
    pub fn test_input(text: impl Into<String>) -> AgentInput {
        AgentInputBuilder::new().text(text).build()
    }

    /// Create a test knowledge slice
    pub fn test_slice() -> KnowledgeSlice {
        KnowledgeSliceBuilder::new()
            .tag("test")
            .description("Test knowledge slice")
            .build()
    }

    /// Create multiple test agents
    pub fn create_test_agents(count: usize) -> Vec<MockAgent> {
        (0..count)
            .map(|i| MockAgent::new(format!("test_agent_{}", i)))
            .collect()
    }

    /// Generate test content
    pub fn generate_test_content(word_count: usize) -> String {
        let words = vec![
            "artificial",
            "intelligence",
            "machine",
            "learning",
            "neural",
            "network",
            "deep",
            "model",
            "training",
            "inference",
            "data",
            "algorithm",
            "optimization",
            "gradient",
            "descent",
            "backpropagation",
        ];

        (0..word_count)
            .map(|_| words[rand::random::<usize>() % words.len()])
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_agent() {
        let agent = MockAgent::new("test").with_response(json!({"result": "ok"}));
        let input = AgentInputBuilder::new().text("test").build();

        let output = agent.invoke(input).await.unwrap();
        assert_eq!(output.data["result"], "ok");
    }

    #[tokio::test]
    async fn test_echo_agent() {
        let agent = MockAgent::echo();
        let input = AgentInputBuilder::new()
            .data(json!({"echo": "test"}))
            .build();

        let output = agent.invoke(input).await.unwrap();
        assert_eq!(output.data["echo"], "test");
    }

    #[tokio::test]
    async fn test_failing_agent() {
        let agent = MockAgent::failing();
        let input = AgentInputBuilder::new().text("test").build();

        let result = agent.invoke(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryStorage::new();

        storage
            .store("key1", "test content", MemoryType::Hot)
            .await
            .unwrap();
        storage
            .store("key2", "other content", MemoryType::Hot)
            .await
            .unwrap();

        assert_eq!(storage.count().await, 2);

        let retrieved = storage.get("key1").await.unwrap();
        assert_eq!(retrieved, Some("test content".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_search() {
        let storage = InMemoryStorage::new();

        storage
            .store(
                "key1",
                "artificial intelligence is growing",
                MemoryType::Hot,
            )
            .await
            .unwrap();
        storage
            .store("key2", "machine learning models", MemoryType::Hot)
            .await
            .unwrap();

        let results = storage.search("intelligence", None).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("intelligence"));
    }
}
