//! Core memory storage traits and common types
//!
//! This module defines the foundational abstractions for all storage backends
//! in the Brain system. Each memory type (Hot, Vector, Graph, Raw) implements
//! its respective trait, allowing for flexible backend implementations.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::memory::{
    EntityRelation, GraphEdge, GraphNode, GraphResponse, MemoryType, SearchResult,
};
use common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use uuid::Uuid;

/// Unique identifier for stored data
pub type StorageId = Uuid;

/// Metadata for stored items
pub type Metadata = HashMap<String, String>;

/// Raw data before processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawData {
    /// Unique identifier
    pub id: StorageId,

    /// Source of the data
    pub source: DataSource,

    /// Raw content
    pub content: String,

    /// Content type (text, json, html, etc.)
    pub content_type: String,

    /// Timestamp when collected
    pub collected_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: Metadata,
}

impl RawData {
    /// Create new raw data
    pub fn new(source: DataSource, content: String, content_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            content,
            content_type,
            collected_at: Utc::now(),
            metadata: Metadata::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Data source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataSource {
    RSS { url: String },
    API { endpoint: String },
    File { path: String },
    Manual { description: String },
    Web { url: String },
}

/// Processed knowledge item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedData {
    /// Unique identifier
    pub id: StorageId,

    /// Reference to original raw data
    pub raw_data_id: StorageId,

    /// Processed/cleaned content
    pub content: String,

    /// Extracted entities
    pub entities: Vec<Entity>,

    /// Generated tags
    pub tags: Vec<String>,

    /// Summary
    pub summary: Option<String>,

    /// Embedding vector (if generated)
    pub embedding: Option<Vec<f32>>,

    /// Timestamp when processed
    pub processed_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: Metadata,
}

/// Extracted entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity text
    pub text: String,

    /// Entity type (PERSON, ORG, LOC, etc.)
    pub entity_type: String,

    /// Confidence score
    pub confidence: f64,

    /// Position in text
    pub start: usize,
    pub end: usize,
}

// ============================================================================
// Storage Backend Traits
// ============================================================================

/// Hot Memory Backend - Fast, short-term storage (Embedded Hot Memory (e.g. Sled))
#[async_trait]
pub trait HotMemoryBackend: Send + Sync + Debug {
    /// Store data with TTL
    async fn set(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()>;

    /// Retrieve data
    async fn get(&self, key: &str) -> Result<Option<String>>;

    /// Delete data
    async fn delete(&self, key: &str) -> Result<bool>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Set multiple values (batch operation)
    async fn mset(&self, items: Vec<(String, String)>) -> Result<()>;

    /// Get multiple values (batch operation)
    async fn mget(&self, keys: Vec<String>) -> Result<Vec<Option<String>>>;

    /// Set key expiry
    async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<bool>;

    /// Get TTL for key
    async fn ttl(&self, key: &str) -> Result<Option<i64>>;

    /// Health check
    async fn health_check(&self) -> Result<bool>;
}

/// Vector Memory Backend - Semantic search (SQLite-vec/In-Memory)
#[async_trait]
pub trait VectorMemoryBackend: Send + Sync + Debug {
    /// Store document with embedding
    async fn store(
        &self,
        id: &str,
        content: &str,
        embedding: &[f32],
        metadata: Metadata,
    ) -> Result<()>;

    /// Store multiple documents (batch)
    async fn store_batch(&self, documents: Vec<VectorDocument>) -> Result<()>;

    /// Search similar documents
    async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<VectorSearchResult>>;

    /// Delete document by ID
    async fn delete(&self, id: &str) -> Result<bool>;

    /// Get document by ID
    async fn get(&self, id: &str) -> Result<Option<VectorDocument>>;

    /// Update document
    async fn update(&self, id: &str, content: &str, embedding: &[f32]) -> Result<bool>;

    /// Get total document count
    async fn count(&self) -> Result<usize>;

    /// Health check
    async fn health_check(&self) -> Result<bool>;
}

/// Vector document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: Metadata,
}

/// Vector search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub content: String,
    pub score: f64,
    pub metadata: Metadata,
}

/// Search filters for vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Tag filters (OR logic within group)
    pub tags: Option<Vec<String>>,

    /// Time range filter
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,

    /// Custom metadata filters
    pub metadata: Option<HashMap<String, String>>,
}

/// Graph Memory Backend - Knowledge graph (Sqlite/petgraph)
#[async_trait]
pub trait GraphMemoryBackend: Send + Sync + Debug {
    /// Add node to graph
    async fn add_node(&self, node: GraphNode) -> Result<String>;

    /// Add edge to graph
    async fn add_edge(&self, edge: GraphEdge) -> Result<String>;

    /// Get node by ID
    async fn get_node(&self, id: &str) -> Result<Option<GraphNode>>;

    /// Get edges for node
    async fn get_edges(&self, node_id: &str) -> Result<Vec<GraphEdge>>;

    /// Find nodes by label
    async fn find_nodes(&self, label: &str, limit: usize) -> Result<Vec<GraphNode>>;

    /// Find shortest path between nodes
    async fn find_path(&self, from: &str, to: &str) -> Result<Vec<GraphEdge>>;

    /// Explore graph around a node
    async fn explore(&self, center_id: &str, depth: usize) -> Result<GraphResponse>;

    /// Search nodes by property
    async fn search_nodes(
        &self,
        property: &str,
        value: &str,
        limit: usize,
    ) -> Result<Vec<GraphNode>>;

    /// Delete node
    async fn delete_node(&self, id: &str) -> Result<bool>;

    /// Delete edge
    async fn delete_edge(&self, id: &str) -> Result<bool>;

    /// Get graph statistics
    async fn stats(&self) -> Result<GraphStats>;

    /// Health check
    async fn health_check(&self) -> Result<bool>;
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_labels: Vec<String>,
    pub edge_types: Vec<String>,
}

/// Raw Archive Backend - Long-term storage (S3/FileSystem)
#[async_trait]
pub trait RawArchiveBackend: Send + Sync + Debug {
    /// Store raw data
    async fn store(&self, data: &RawData) -> Result<StorageId>;

    /// Store multiple raw data items (batch)
    async fn store_batch(&self, items: Vec<RawData>) -> Result<Vec<StorageId>>;

    /// Retrieve by ID
    async fn get(&self, id: StorageId) -> Result<Option<RawData>>;

    /// List items by source
    async fn list_by_source(&self, source: &DataSource, limit: usize) -> Result<Vec<RawData>>;

    /// List items by time range
    async fn list_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<RawData>>;

    /// Search content
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<RawData>>;

    /// Delete by ID
    async fn delete(&self, id: StorageId) -> Result<bool>;

    /// Get storage size in bytes
    async fn storage_size(&self) -> Result<u64>;

    /// Archive old data
    async fn archive_before(&self, date: DateTime<Utc>) -> Result<usize>;

    /// Get statistics
    async fn stats(&self) -> Result<ArchiveStats>;

    /// Health check
    async fn health_check(&self) -> Result<bool>;
}

/// Archive statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStats {
    pub total_items: u64,
    pub total_size_bytes: u64,
    pub oldest_item: Option<DateTime<Utc>>,
    pub newest_item: Option<DateTime<Utc>>,
    pub by_source: HashMap<String, u64>,
}

/// Memory operation context for tracking and metrics
#[derive(Debug, Clone)]
pub struct MemoryContext {
    /// Operation ID for tracing
    pub operation_id: Uuid,

    /// Request ID for correlation
    pub request_id: Option<String>,

    /// User ID for authorization
    pub user_id: Option<String>,

    /// Additional context
    pub extra: HashMap<String, String>,
}

impl Default for MemoryContext {
    fn default() -> Self {
        Self {
            operation_id: Uuid::new_v4(),
            request_id: None,
            user_id: None,
            extra: HashMap::new(),
        }
    }
}

/// Generic memory backend trait that all backends must implement
#[async_trait]
pub trait MemoryBackend: Send + Sync + Debug {
    /// Get backend type
    fn backend_type(&self) -> MemoryType;

    /// Initialize backend
    async fn initialize(&self) -> Result<()>;

    /// Shutdown backend gracefully
    async fn shutdown(&self) -> Result<()>;

    /// Health check
    async fn health_check(&self) -> Result<bool>;

    /// Get backend metrics
    async fn metrics(&self) -> Result<BackendMetrics>;
}

/// Backend metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendMetrics {
    pub backend_type: MemoryType,
    pub uptime_seconds: u64,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_latency_ms: f64,
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_data_creation() {
        let data = RawData::new(
            DataSource::RSS {
                url: "https://example.com/feed".to_string(),
            },
            "Test content".to_string(),
            "text/plain".to_string(),
        )
        .with_metadata("author", "Test Author");

        assert_eq!(data.content, "Test content");
        assert_eq!(
            data.metadata.get("author"),
            Some(&"Test Author".to_string())
        );
    }

    #[test]
    fn test_memory_context_default() {
        let ctx = MemoryContext::default();
        assert!(ctx.request_id.is_none());
        assert!(ctx.user_id.is_none());
    }
}
