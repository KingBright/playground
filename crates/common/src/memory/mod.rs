//! Memory API types and interfaces

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of memory storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Hot memory - Embedded (e.g., Sled)
    Hot,

    /// Vector memory - SQLite-vec (semantic search)
    Vector,

    /// Graph memory - Embedded (e.g., petgraph+sqlite)
    Graph,

    /// Raw archive - S3/MinIO (long-term storage)
    Raw,
}

/// A slice of knowledge for mounting to agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSlice {
    /// Unique identifier for this slice
    pub id: String,

    /// Time range for this slice
    pub time_range: TimeRange,

    /// Tags for filtering
    pub tags: Vec<String>,

    /// Optional description
    pub description: Option<String>,
}

/// Time range for knowledge slicing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, time: DateTime<Utc>) -> bool {
        time >= self.start && time <= self.end
    }
}

/// Search result from memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The content
    pub content: String,

    /// Relevance score (0-1)
    pub score: f64,

    /// Source metadata
    pub metadata: HashMap<String, String>,

    /// Related entities (if from graph memory)
    pub related_entities: Vec<EntityRelation>,
}

/// Entity relation from graph memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelation {
    pub entity: String,
    pub relation: String,
    pub direction: RelationDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationDirection {
    Outgoing,
    Incoming,
    Both,
}

/// Graph exploration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResponse {
    pub center_entity: String,

    pub nodes: Vec<GraphNode>,

    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphNode {
    pub id: String,
    pub labels: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

use std::collections::HashMap;
