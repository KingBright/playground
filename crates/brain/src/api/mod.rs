//! Brain API - REST API for the Brain system
//!
//! This module provides HTTP endpoints for:
//! - Knowledge search across all storage backends
//! - Graph exploration
//! - Data ingestion
//! - Agent invocation
//! - Health checks

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::storage::{
    GraphMemoryBackend, HealthStatus, HotMemoryBackend, RawArchiveBackend, RawData, SearchFilters,
    StorageId, UnifiedMemory, UnifiedMemoryConfig, VectorMemoryBackend,
};
use common::memory::{GraphEdge as CommonGraphEdge, GraphNode as CommonGraphNode, SearchResult};
// Note: We use std::result::Result in handlers, not common::Result

/// Brain API state shared across handlers
#[derive(Clone, Debug)]
pub struct BrainApiState {
    /// Unified memory facade
    pub memory: Arc<UnifiedMemory>,
}

impl BrainApiState {
    /// Create new API state with unified memory
    pub fn new(memory: Arc<UnifiedMemory>) -> Self {
        Self { memory }
    }
}

/// Brain API
#[derive(Debug)]
pub struct BrainApi {
    state: BrainApiState,
}

impl BrainApi {
    /// Create new Brain API instance
    pub fn new(memory: Arc<UnifiedMemory>) -> Self {
        Self {
            state: BrainApiState::new(memory),
        }
    }

    /// Create API router with all routes
    pub fn router(&self) -> Router {
        Router::new()
            // Health check
            .route("/health", get(health_check))
            // Knowledge search
            .route("/knowledge/search", post(search_knowledge))
            // Graph exploration
            .route("/graph/explore", post(explore_graph))
            .route("/graph/nodes", post(add_graph_node))
            .route("/graph/edges", post(add_graph_edge))
            // Data ingestion
            .route("/data/ingest", post(ingest_data))
            // Memory operations
            .route(
                "/memory/hot/{key}",
                get(get_hot_memory).post(set_hot_memory),
            )
            .route("/memory/vector", post(store_vector).get(search_vector))
            // Mount knowledge slice
            .route("/mount/knowledge_slice", post(mount_knowledge_slice))
            .with_state(self.state.clone())
    }
}

// =============================================================================
// Request/Response Types
// =============================================================================

/// Search request
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    /// Search query
    pub query: String,
    /// Optional query embedding for semantic search
    pub embedding: Option<Vec<f32>>,
    /// Maximum results to return
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Search filters
    #[serde(default)]
    pub filters: HashMap<String, String>,
}

fn default_limit() -> usize {
    10
}

/// Search response
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total count
    pub total: usize,
    /// Query time in milliseconds
    pub query_time_ms: u64,
}

/// Graph exploration request
#[derive(Debug, Deserialize)]
pub struct GraphExploreRequest {
    /// Center node ID
    pub center_id: String,
    /// Exploration depth
    #[serde(default = "default_depth")]
    pub depth: usize,
}

fn default_depth() -> usize {
    2
}

/// Graph node creation request
#[derive(Debug, Deserialize)]
pub struct GraphNodeRequest {
    /// Node ID (optional, auto-generated if not provided)
    pub id: Option<String>,
    /// Node labels
    pub labels: Vec<String>,
    /// Node properties
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// Graph edge creation request
#[derive(Debug, Deserialize)]
pub struct GraphEdgeRequest {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Edge relationship type (label)
    pub label: String,
    /// Edge properties
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// Data ingestion request
#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    /// Content to ingest
    pub content: String,
    /// Content type
    #[serde(default = "default_content_type")]
    pub content_type: String,
    /// Source information
    #[serde(default)]
    pub source: String,
    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_content_type() -> String {
    "text/plain".to_string()
}

/// Ingestion response
#[derive(Debug, Serialize)]
pub struct IngestResponse {
    /// Storage ID of archived data
    pub storage_id: String,
    /// Status message
    pub message: String,
}

/// Hot memory set request
#[derive(Debug, Deserialize)]
pub struct HotMemorySetRequest {
    /// Value to store
    pub value: String,
    /// TTL in seconds
    #[serde(default = "default_ttl")]
    pub ttl_seconds: u64,
}

fn default_ttl() -> u64 {
    3600 // 1 hour
}

/// Vector document store request
#[derive(Debug, Deserialize)]
pub struct VectorStoreRequest {
    /// Document ID
    pub id: String,
    /// Document content
    pub content: String,
    /// Document embedding vector
    pub embedding: Vec<f32>,
    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Vector search request
#[derive(Debug, Deserialize)]
pub struct VectorSearchRequest {
    /// Query embedding
    pub embedding: Vec<f32>,
    /// Maximum results
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Knowledge slice mount request
#[derive(Debug, Deserialize)]
pub struct MountKnowledgeSliceRequest {
    /// Slice name
    pub name: String,
    /// Slice type
    pub slice_type: String,
    /// Slice content
    pub content: String,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Overall health status
    pub status: String,
    /// Hot memory status
    pub hot_memory: bool,
    /// Vector memory status
    pub vector_memory: bool,
    /// Graph memory status
    pub graph_memory: bool,
    /// Raw archive status
    pub raw_archive: bool,
}

// =============================================================================
// Handlers
// =============================================================================

/// Health check handler
async fn health_check(
    State(state): State<BrainApiState>,
) -> std::result::Result<Json<HealthResponse>, StatusCode> {
    match state.memory.health_check().await {
        Ok(health) => {
            let status = if health.overall {
                "healthy"
            } else {
                "unhealthy"
            };
            Ok(Json(HealthResponse {
                status: status.to_string(),
                hot_memory: health.hot_memory,
                vector_memory: health.vector_memory,
                graph_memory: health.graph_memory,
                raw_archive: health.raw_archive,
            }))
        }
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Search knowledge across all backends
async fn search_knowledge(
    State(state): State<BrainApiState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ErrorResponse>)> {
    let start = std::time::Instant::now();

    let embedding = request.embedding.as_deref();

    match state
        .memory
        .search_unified(&request.query, embedding, request.limit)
        .await
    {
        Ok(results) => {
            let total = results.len();
            let query_time_ms = start.elapsed().as_millis() as u64;

            Ok(Json(SearchResponse {
                results,
                total,
                query_time_ms,
            }))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Search failed: {}", e),
                code: "SEARCH_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Explore knowledge graph
async fn explore_graph(
    State(state): State<BrainApiState>,
    Json(request): Json<GraphExploreRequest>,
) -> Result<Json<common::memory::GraphResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .memory
        .explore_graph(&request.center_id, request.depth)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Graph exploration failed: {}", e),
                code: "GRAPH_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Add node to knowledge graph
async fn add_graph_node(
    State(state): State<BrainApiState>,
    Json(request): Json<GraphNodeRequest>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    let node = CommonGraphNode {
        id: request
            .id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        labels: request.labels,
        properties: request.properties,
    };

    match state.memory.add_graph_node(node).await {
        Ok(id) => {
            let mut response = HashMap::new();
            response.insert("id".to_string(), id);
            response.insert("status".to_string(), "created".to_string());
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to add node: {}", e),
                code: "NODE_CREATE_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Add edge to knowledge graph
async fn add_graph_edge(
    State(state): State<BrainApiState>,
    Json(request): Json<GraphEdgeRequest>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    let edge = CommonGraphEdge {
        id: uuid::Uuid::new_v4().to_string(),
        from: request.from,
        to: request.to,
        label: request.label,
        properties: request.properties,
    };

    match state.memory.add_graph_edge(edge).await {
        Ok(id) => {
            let mut response = HashMap::new();
            response.insert("id".to_string(), id);
            response.insert("status".to_string(), "created".to_string());
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to add edge: {}", e),
                code: "EDGE_CREATE_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Ingest data into the system
async fn ingest_data(
    State(state): State<BrainApiState>,
    Json(request): Json<IngestRequest>,
) -> std::result::Result<Json<IngestResponse>, (StatusCode, Json<ErrorResponse>)> {
    use crate::storage::DataSource;

    let source = DataSource::Manual {
        description: request.source,
    };

    let raw_data = RawData::new(source, request.content, request.content_type)
        .with_metadata("ingested_via", "api");

    match state.memory.archive_raw(&raw_data).await {
        Ok(storage_id) => Ok(Json(IngestResponse {
            storage_id: storage_id.to_string(),
            message: "Data ingested successfully".to_string(),
        })),
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Ingestion failed: {}", e),
                code: "INGEST_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Get value from hot memory
async fn get_hot_memory(
    State(state): State<BrainApiState>,
    Path(key): Path<String>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    match state.memory.get_hot(&key).await {
        Ok(Some(value)) => {
            let mut response = HashMap::new();
            response.insert("key".to_string(), key);
            response.insert("value".to_string(), value);
            Ok(Json(response))
        }
        Ok(None) => {
            let error_response = ErrorResponse {
                error: "Key not found".to_string(),
                code: "NOT_FOUND".to_string(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to get value: {}", e),
                code: "GET_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Set value in hot memory
async fn set_hot_memory(
    State(state): State<BrainApiState>,
    Path(key): Path<String>,
    Json(request): Json<HotMemorySetRequest>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .memory
        .store_hot(&key, &request.value, request.ttl_seconds)
        .await
    {
        Ok(()) => {
            let mut response = HashMap::new();
            response.insert("key".to_string(), key);
            response.insert("status".to_string(), "stored".to_string());
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to store value: {}", e),
                code: "STORE_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Store document in vector memory
async fn store_vector(
    State(state): State<BrainApiState>,
    Json(request): Json<VectorStoreRequest>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .memory
        .store_vector(
            &request.id,
            &request.content,
            &request.embedding,
            request.metadata,
        )
        .await
    {
        Ok(()) => {
            let mut response = HashMap::new();
            response.insert("id".to_string(), request.id);
            response.insert("status".to_string(), "stored".to_string());
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Failed to store vector: {}", e),
                code: "VECTOR_STORE_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Search vector memory
async fn search_vector(
    State(state): State<BrainApiState>,
    Query(request): Query<VectorSearchRequest>,
) -> Result<Json<Vec<crate::storage::VectorSearchResult>>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .memory
        .search_vector(&request.embedding, request.limit, None)
        .await
    {
        Ok(results) => Ok(Json(results)),
        Err(e) => {
            let error_response = ErrorResponse {
                error: format!("Vector search failed: {}", e),
                code: "VECTOR_SEARCH_ERROR".to_string(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// Mount knowledge slice (placeholder implementation)
async fn mount_knowledge_slice(
    State(_state): State<BrainApiState>,
    Json(request): Json<MountKnowledgeSliceRequest>,
) -> Result<Json<HashMap<String, String>>, (StatusCode, Json<ErrorResponse>)> {
    // This is a placeholder - actual implementation would integrate with mount module
    let mut response = HashMap::new();
    response.insert("name".to_string(), request.name);
    response.insert("slice_type".to_string(), request.slice_type);
    response.insert("status".to_string(), "mounted".to_string());
    response.insert(
        "message".to_string(),
        "Knowledge slice mounted successfully (placeholder)".to_string(),
    );
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{
        hot_memory::InMemoryHotMemory,
        raw_archive::{FileSystemRawArchive, RawArchiveConfig},
        vector_memory::InMemoryVectorStore,
    };
    use std::path::PathBuf;

    async fn create_test_state() -> BrainApiState {
        let hot = Arc::new(InMemoryHotMemory::new()) as Arc<dyn HotMemoryBackend>;
        let vector =
            Arc::new(InMemoryVectorStore::with_default_config()) as Arc<dyn VectorMemoryBackend>;
        let graph = Arc::new(crate::storage::graph_memory::InMemoryGraphStore::new())
            as Arc<dyn GraphMemoryBackend>;
        let raw = Arc::new(
            FileSystemRawArchive::new(RawArchiveConfig {
                storage_dir: PathBuf::from("./test_data/api"),
                ..Default::default()
            })
            .await
            .unwrap(),
        ) as Arc<dyn RawArchiveBackend>;

        let memory = Arc::new(UnifiedMemory::new(hot, vector, graph, raw));
        BrainApiState::new(memory)
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = create_test_state().await;
        let result = health_check(State(state)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_search_knowledge() {
        let state = create_test_state().await;
        let request = SearchRequest {
            query: "test".to_string(),
            embedding: None,
            limit: 10,
            filters: HashMap::new(),
        };
        let result = search_knowledge(State(state), Json(request)).await;
        assert!(result.is_ok());
    }
}
