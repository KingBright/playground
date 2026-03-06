//! Storage layer for the Brain system
//!
//! This module provides all storage implementations:
//! - Hot Memory: Fast embedded cache with TTL
//! - Vector Memory: Semantic search with embeddings
//! - Graph Memory: Knowledge graph for entity relationships
//! - Raw Archive: Long-term file-based storage
//! - Unified Memory: Facade combining all backends

pub mod graph_memory;
pub mod hot_memory;
pub mod memory;
pub mod raw_archive;
pub mod unified_memory;
pub mod vector_memory;

// Re-exports
pub use memory::{
    ArchiveStats, BackendMetrics, DataSource, Entity, GraphMemoryBackend, GraphStats,
    HotMemoryBackend, MemoryBackend, MemoryContext, Metadata, ProcessedData, RawArchiveBackend,
    RawData, SearchFilters, StorageId, VectorDocument, VectorMemoryBackend, VectorSearchResult,
};

pub use graph_memory::InMemoryGraphStore;
pub use hot_memory::{InMemoryHotMemory, HotMemoryConfig, HotMemoryMetrics};
pub use raw_archive::{FileSystemRawArchive, RawArchiveConfig};
pub use unified_memory::{HealthStatus, UnifiedMemory, UnifiedMemoryConfig, UnifiedMemoryMetrics};
pub use vector_memory::InMemoryVectorStore;
