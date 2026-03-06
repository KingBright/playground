//! Unified Memory Facade - Aggregates all storage backends
//!
//! This module provides a single interface to all storage backends with:
//! - Intelligent routing based on query type
//! - Fallback mechanisms
//! - Cross-backend search
//! - Performance optimization
//! - Caching layer

use crate::storage::{
    GraphMemoryBackend, HotMemoryBackend, MemoryContext, RawArchiveBackend, RawData, SearchFilters,
    StorageId, VectorMemoryBackend, VectorSearchResult,
};
use common::memory::{MemoryType, SearchResult};
use common::{Error, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Unified memory combining all backends
#[derive(Debug)]
pub struct UnifiedMemory {
    /// Hot memory backend (Embedded Hot Memory (e.g. Sled))
    hot: Arc<dyn HotMemoryBackend>,

    /// Vector memory backend (Qdrant/In-Memory)
    vector: Arc<dyn VectorMemoryBackend>,

    /// Graph memory backend (Sqlite/petgraph)
    graph: Arc<dyn GraphMemoryBackend>,

    /// Raw archive backend (S3/FileSystem)
    raw: Arc<dyn RawArchiveBackend>,

    /// In-memory cache for frequently accessed data
    cache: Arc<RwLock<crate::storage::hot_memory::InMemoryHotMemory>>,

    /// Configuration
    config: UnifiedMemoryConfig,
}

/// Unified memory configuration
#[derive(Debug, Clone)]
pub struct UnifiedMemoryConfig {
    /// Enable caching
    pub cache_enabled: bool,

    /// Cache TTL in seconds
    pub cache_ttl: u64,

    /// Enable fallback to other backends on failure
    pub fallback_enabled: bool,

    /// Maximum concurrent operations
    pub max_concurrent_ops: usize,
}

impl Default for UnifiedMemoryConfig {
    fn default() -> Self {
        Self {
            cache_enabled: true,
            cache_ttl: 3600, // 1 hour
            fallback_enabled: true,
            max_concurrent_ops: 100,
        }
    }
}

impl UnifiedMemory {
    /// Create a new unified memory instance
    pub fn new(
        hot: Arc<dyn HotMemoryBackend>,
        vector: Arc<dyn VectorMemoryBackend>,
        graph: Arc<dyn GraphMemoryBackend>,
        raw: Arc<dyn RawArchiveBackend>,
    ) -> Self {
        info!("Creating unified memory facade");

        Self {
            hot,
            vector,
            graph,
            raw,
            cache: Arc::new(RwLock::new(
                crate::storage::hot_memory::InMemoryHotMemory::new(),
            )),
            config: UnifiedMemoryConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        hot: Arc<dyn HotMemoryBackend>,
        vector: Arc<dyn VectorMemoryBackend>,
        graph: Arc<dyn GraphMemoryBackend>,
        raw: Arc<dyn RawArchiveBackend>,
        config: UnifiedMemoryConfig,
    ) -> Self {
        Self {
            hot,
            vector,
            graph,
            raw,
            cache: Arc::new(RwLock::new(
                crate::storage::hot_memory::InMemoryHotMemory::new(),
            )),
            config,
        }
    }

    // ========================================================================
    // Hot Memory Operations
    // ========================================================================

    /// Store data in hot memory with TTL
    pub async fn store_hot(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        debug!("Storing in hot memory: key='{}', ttl={}s", key, ttl_seconds);

        self.hot.set(key, value, ttl_seconds).await.map_err(|e| {
            warn!("Failed to store in hot memory: {}", e);
            e
        })?;

        // TODO: Implement caching
        // For now, just skip cache updates

        Ok(())
    }

    /// Retrieve data from hot memory (with cache fallback)
    pub async fn get_hot(&self, key: &str) -> Result<Option<String>> {
        debug!("Retrieving from hot memory: key='{}'", key);

        // TODO: Implement caching
        // For now, just query hot memory directly
        match self.hot.get(key).await {
            Ok(value) => Ok(value),
            Err(e) => {
                warn!("Failed to retrieve from hot memory: {}", e);
                Err(e)
            }
        }
    }

    // ========================================================================
    // Vector Memory Operations
    // ========================================================================

    /// Store document with embedding
    pub async fn store_vector(
        &self,
        id: &str,
        content: &str,
        embedding: &[f32],
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        debug!("Storing in vector memory: id='{}'", id);

        self.vector
            .store(id, content, embedding, metadata)
            .await
            .map_err(|e| {
                warn!("Failed to store in vector memory: {}", e);
                e
            })
    }

    /// Semantic search across vector memory
    pub async fn search_vector(
        &self,
        query_embedding: &[f32],
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<VectorSearchResult>> {
        debug!("Searching vector memory: limit={}", limit);

        self.vector
            .search(query_embedding, limit, filters)
            .await
            .map_err(|e| {
                warn!("Vector search failed: {}", e);
                e
            })
    }

    // ========================================================================
    // Graph Memory Operations
    // ========================================================================

    /// Add node to knowledge graph
    pub async fn add_graph_node(&self, node: common::memory::GraphNode) -> Result<String> {
        debug!("Adding node to graph: labels={:?}", node.labels);

        self.graph.add_node(node).await.map_err(|e| {
            warn!("Failed to add graph node: {}", e);
            e
        })
    }

    /// Add edge to knowledge graph
    pub async fn add_graph_edge(&self, edge: common::memory::GraphEdge) -> Result<String> {
        debug!("Adding edge to graph: {} -> {}", edge.from, edge.to);

        self.graph.add_edge(edge).await.map_err(|e| {
            warn!("Failed to add graph edge: {}", e);
            e
        })
    }

    /// Explore graph around a node
    pub async fn explore_graph(
        &self,
        center_id: &str,
        depth: usize,
    ) -> Result<common::memory::GraphResponse> {
        debug!("Exploring graph: center='{}', depth={}", center_id, depth);

        self.graph.explore(center_id, depth).await.map_err(|e| {
            warn!("Graph exploration failed: {}", e);
            e
        })
    }

    // ========================================================================
    // Raw Archive Operations
    // ========================================================================

    /// Store raw data in archive
    pub async fn archive_raw(&self, data: &RawData) -> Result<StorageId> {
        debug!("Archiving raw data: id={:?}", data.id);

        self.raw.store(data).await.map_err(|e| {
            warn!("Failed to archive raw data: {}", e);
            e
        })
    }

    /// Retrieve raw data from archive
    pub async fn retrieve_raw(&self, id: StorageId) -> Result<Option<RawData>> {
        debug!("Retrieving raw data: id={}", id);

        self.raw.get(id).await.map_err(|e| {
            warn!("Failed to retrieve raw data: {}", e);
            e
        })
    }

    /// Search raw archive
    pub async fn search_archive(&self, query: &str, limit: usize) -> Result<Vec<RawData>> {
        debug!("Searching archive: query='{}'", query);

        self.raw.search(query, limit).await.map_err(|e| {
            warn!("Archive search failed: {}", e);
            e
        })
    }

    // ========================================================================
    // Unified Search (Cross-backend)
    // ========================================================================

    /// Unified search across all backends
    pub async fn search_unified(
        &self,
        query: &str,
        query_embedding: Option<&[f32]>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        debug!("Unified search: query='{}', limit={}", query, limit);

        let mut all_results = Vec::new();

        // 1. Search hot memory
        if let Ok(Some(hot_result)) = self.get_hot(query).await {
            all_results.push(SearchResult {
                content: hot_result,
                score: 1.0, // Hot memory has highest priority
                metadata: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("source".to_string(), "hot_memory".to_string());
                    m
                },
                related_entities: vec![],
            });
        }

        // 2. Search vector memory if embedding provided
        if let Some(embedding) = query_embedding {
            if let Ok(vector_results) = self.search_vector(embedding, limit, None).await {
                for result in vector_results {
                    all_results.push(SearchResult {
                        content: result.content,
                        score: result.score,
                        metadata: {
                            let mut m = result.metadata;
                            m.insert("source".to_string(), "vector_memory".to_string());
                            m
                        },
                        related_entities: vec![],
                    });
                }
            }
        }

        // 3. Search raw archive
        if let Ok(archive_results) = self.search_archive(query, limit).await {
            for raw_data in archive_results {
                all_results.push(SearchResult {
                    content: raw_data.content,
                    score: 0.5, // Archive has lower priority
                    metadata: {
                        let mut m = raw_data.metadata;
                        m.insert("source".to_string(), "raw_archive".to_string());
                        m.insert(
                            "collected_at".to_string(),
                            raw_data.collected_at.to_rfc3339(),
                        );
                        m
                    },
                    related_entities: vec![],
                });
            }
        }

        // Sort by score
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Limit results
        all_results.truncate(limit);

        debug!("Unified search returned {} results", all_results.len());

        Ok(all_results)
    }

    // ========================================================================
    // Health & Metrics
    // ========================================================================

    /// Health check for all backends
    pub async fn health_check(&self) -> Result<HealthStatus> {
        debug!("Performing health check");

        let hot_healthy = self.hot.health_check().await.unwrap_or(false);
        let vector_healthy = self.vector.health_check().await.unwrap_or(false);
        let graph_healthy = self.graph.health_check().await.unwrap_or(false);
        let raw_healthy = self.raw.health_check().await.unwrap_or(false);

        let overall_healthy = hot_healthy && vector_healthy && graph_healthy && raw_healthy;

        Ok(HealthStatus {
            overall: overall_healthy,
            hot_memory: hot_healthy,
            vector_memory: vector_healthy,
            graph_memory: graph_healthy,
            raw_archive: raw_healthy,
        })
    }

    /// Get metrics from all backends
    pub async fn get_metrics(&self) -> Result<UnifiedMemoryMetrics> {
        Ok(UnifiedMemoryMetrics {
            hot_memory_ops: 0, // TODO: Track operations
            vector_memory_count: self.vector.count().await.unwrap_or(0),
            graph_memory_stats: self.graph.stats().await.ok(),
            raw_archive_stats: self.raw.stats().await.ok(),
        })
    }
}

/// Health status
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub overall: bool,
    pub hot_memory: bool,
    pub vector_memory: bool,
    pub graph_memory: bool,
    pub raw_archive: bool,
}

/// Unified memory metrics
#[derive(Debug, Clone)]
pub struct UnifiedMemoryMetrics {
    pub hot_memory_ops: u64,
    pub vector_memory_count: usize,
    pub graph_memory_stats: Option<crate::storage::GraphStats>,
    pub raw_archive_stats: Option<crate::storage::ArchiveStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::hot_memory::InMemoryHotMemory;
    use crate::storage::raw_archive::{FileSystemRawArchive, RawArchiveConfig};
    use crate::storage::vector_memory::InMemoryVectorStore;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_unified_memory_hot_operations() {
        let hot = Arc::new(InMemoryHotMemory::new()) as Arc<dyn HotMemoryBackend>;
        let vector =
            Arc::new(InMemoryVectorStore::with_default_config()) as Arc<dyn VectorMemoryBackend>;
        let graph = Arc::new(crate::storage::graph_memory::InMemoryGraphStore::new())
            as Arc<dyn GraphMemoryBackend>;
        let raw = Arc::new(
            FileSystemRawArchive::new(RawArchiveConfig {
                storage_dir: PathBuf::from("./test_data/unified"),
                ..Default::default()
            })
            .await
            .unwrap(),
        ) as Arc<dyn RawArchiveBackend>;

        let memory = UnifiedMemory::new(hot, vector, graph, raw);

        // Test hot memory operations
        memory
            .store_hot("test_key", "test_value", 60)
            .await
            .unwrap();

        let retrieved = memory.get_hot("test_key").await.unwrap();
        assert_eq!(retrieved, Some("test_value".to_string()));

        // Cleanup
        let _ = tokio::fs::remove_dir_all("./test_data");
    }

    #[tokio::test]
    async fn test_unified_memory_health() {
        let hot = Arc::new(InMemoryHotMemory::new()) as Arc<dyn HotMemoryBackend>;
        let vector =
            Arc::new(InMemoryVectorStore::with_default_config()) as Arc<dyn VectorMemoryBackend>;
        let graph = Arc::new(crate::storage::graph_memory::InMemoryGraphStore::new())
            as Arc<dyn GraphMemoryBackend>;
        let raw = Arc::new(
            FileSystemRawArchive::new(RawArchiveConfig {
                storage_dir: PathBuf::from("./test_data/health"),
                ..Default::default()
            })
            .await
            .unwrap(),
        ) as Arc<dyn RawArchiveBackend>;

        let memory = UnifiedMemory::new(hot, vector, graph, raw);

        let health = memory.health_check().await.unwrap();
        assert!(health.overall);

        // Cleanup
        let _ = tokio::fs::remove_dir_all("./test_data");
    }
}
