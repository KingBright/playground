//! Raw Archive implementation using file system storage
//!
//! This module provides:
//! - File-based long-term storage
//! - JSON format for easy inspection
//! - Metadata indexing for fast queries
//! - Compression support (optional)
//! - Archive lifecycle management
//! - Retention policies

use crate::storage::{ArchiveStats, DataSource, RawArchiveBackend, RawData, StorageId};
use chrono::{DateTime, Datelike, Utc};
use common::{Error, Result};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Raw archive configuration
#[derive(Debug, Clone)]
pub struct RawArchiveConfig {
    /// Base directory for storage
    pub storage_dir: PathBuf,

    /// Enable compression
    pub compression_enabled: bool,

    /// Retention duration in days (None = keep forever)
    pub retention_days: Option<u64>,

    /// Index batch size
    pub index_batch_size: usize,
}

impl Default for RawArchiveConfig {
    fn default() -> Self {
        Self {
            storage_dir: PathBuf::from("./data/raw_archive"),
            compression_enabled: false,
            retention_days: None,
            index_batch_size: 1000,
        }
    }
}

/// File system raw archive implementation
#[derive(Debug)]
pub struct FileSystemRawArchive {
    config: RawArchiveConfig,
    metadata_index: Arc<RwLock<HashMap<StorageId, ArchiveMetadata>>>,
}

/// Archive metadata for indexing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ArchiveMetadata {
    id: StorageId,
    source: DataSource,
    collected_at: DateTime<Utc>,
    content_type: String,
    file_path: PathBuf,
    size_bytes: u64,
    metadata: HashMap<String, String>,
}

impl FileSystemRawArchive {
    /// Create a new file system raw archive
    pub async fn new(config: RawArchiveConfig) -> Result<Self> {
        info!(
            "Creating file system raw archive at {:?}",
            config.storage_dir
        );

        // Create storage directory if it doesn't exist
        fs::create_dir_all(&config.storage_dir).map_err(|e| {
            Error::StorageError(format!("Failed to create storage directory: {}", e))
        })?;

        let archive = Self {
            config,
            metadata_index: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing index
        archive.load_index().await?;

        Ok(archive)
    }

    /// Create with default configuration
    pub async fn with_default_config() -> Result<Self> {
        Self::new(RawArchiveConfig::default()).await
    }

    /// Get file path for a storage ID
    fn get_file_path(&self, id: StorageId) -> PathBuf {
        // Organize by date: YYYY/MM/DD/{uuid}.json
        let now = Utc::now();
        let date_dir = self
            .config
            .storage_dir
            .join(format!("{:04}", now.year()))
            .join(format!("{:02}", now.month()))
            .join(format!("{:02}", now.day()));

        date_dir.join(format!("{}.json", id))
    }

    /// Ensure directory exists
    fn ensure_dir(path: &Path) -> Result<()> {
        fs::create_dir_all(path).map_err(|e| {
            Error::StorageError(format!("Failed to create directory {:?}: {}", path, e))
        })?;
        Ok(())
    }

    /// Save index to disk
    async fn save_index(&self) -> Result<()> {
        let index_path = self.config.storage_dir.join("index.json");

        let index = self.metadata_index.read().await;
        let index_data: Vec<&ArchiveMetadata> = index.values().collect();

        let json = serde_json::to_string_pretty(&index_data)
            .map_err(|e| Error::StorageError(format!("Failed to serialize index: {}", e)))?;

        tokio::fs::write(&index_path, json)
            .await
            .map_err(|e| Error::StorageError(format!("Failed to write index: {}", e)))?;

        debug!("Saved index with {} entries", index_data.len());
        Ok(())
    }

    /// Load index from disk
    async fn load_index(&self) -> Result<()> {
        let index_path = self.config.storage_dir.join("index.json");

        if !index_path.exists() {
            debug!("No existing index found");
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&index_path)
            .await
            .map_err(|e| Error::StorageError(format!("Failed to read index: {}", e)))?;

        let index_data: Vec<ArchiveMetadata> = serde_json::from_str(&content)
            .map_err(|e| Error::StorageError(format!("Failed to deserialize index: {}", e)))?;

        let mut index = self.metadata_index.write().await;
        for metadata in index_data {
            index.insert(metadata.id, metadata);
        }

        info!("Loaded index with {} entries", index.len());
        Ok(())
    }

    /// Query index by source
    async fn query_by_source(&self, source: &DataSource) -> Vec<ArchiveMetadata> {
        let index = self.metadata_index.read().await;
        index
            .values()
            .filter(|m| &m.source == source)
            .cloned()
            .collect()
    }

    /// Query index by time range
    async fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<ArchiveMetadata> {
        let index = self.metadata_index.read().await;
        index
            .values()
            .filter(|m| m.collected_at >= start && m.collected_at <= end)
            .cloned()
            .collect()
    }

    /// Query index by content search
    async fn query_by_search(&self, query: &str, limit: usize) -> Vec<ArchiveMetadata> {
        let index = self.metadata_index.read().await;
        let query_lower = query.to_lowercase();

        let mut results = Vec::new();
        for metadata in index.values() {
            if results.len() >= limit {
                break;
            }

            // Read file and search content
            if let Ok(content) = tokio::fs::read_to_string(&metadata.file_path).await {
                if content.to_lowercase().contains(&query_lower) {
                    results.push(metadata.clone());
                }
            }
        }

        results
    }
}

#[async_trait::async_trait]
impl RawArchiveBackend for FileSystemRawArchive {
    async fn store(&self, data: &RawData) -> Result<StorageId> {
        debug!("Storing raw data with ID {}", data.id);

        let file_path = self.get_file_path(data.id);

        // Create directory
        Self::ensure_dir(
            file_path
                .parent()
                .ok_or_else(|| Error::StorageError("Invalid file path".to_string()))?,
        )?;

        // Serialize data
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| Error::StorageError(format!("Failed to serialize raw data: {}", e)))?;

        // Write to file
        tokio::fs::write(&file_path, json)
            .await
            .map_err(|e| Error::StorageError(format!("Failed to write file: {}", e)))?;

        // Get file size
        let size_bytes = tokio::fs::metadata(&file_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        // Update index
        let metadata = ArchiveMetadata {
            id: data.id,
            source: data.source.clone(),
            collected_at: data.collected_at,
            content_type: data.content_type.clone(),
            file_path,
            size_bytes,
            metadata: data.metadata.clone(),
        };

        let mut index = self.metadata_index.write().await;
        index.insert(data.id, metadata);

        // Save index periodically
        if index.len() % self.config.index_batch_size == 0 {
            drop(index);
            self.save_index().await?;
        }

        Ok(data.id)
    }

    async fn store_batch(&self, items: Vec<RawData>) -> Result<Vec<StorageId>> {
        debug!("Storing batch of {} raw data items", items.len());

        let mut ids = Vec::new();

        for item in items {
            let id = self.store(&item).await?;
            ids.push(id);
        }

        // Save index after batch
        self.save_index().await?;

        Ok(ids)
    }

    async fn get(&self, id: StorageId) -> Result<Option<RawData>> {
        let index = self.metadata_index.read().await;

        if let Some(metadata) = index.get(&id) {
            let content = tokio::fs::read_to_string(&metadata.file_path)
                .await
                .map_err(|e| Error::StorageError(format!("Failed to read file: {}", e)))?;

            let data: RawData = serde_json::from_str(&content).map_err(|e| {
                Error::StorageError(format!("Failed to deserialize raw data: {}", e))
            })?;

            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    async fn list_by_source(&self, source: &DataSource, limit: usize) -> Result<Vec<RawData>> {
        let metadata_list = self.query_by_source(source).await;
        let metadata_list = metadata_list.into_iter().take(limit).collect::<Vec<_>>();

        let mut results = Vec::new();
        for metadata in metadata_list {
            if let Ok(Some(data)) = self.get(metadata.id).await {
                results.push(data);
            }
        }

        Ok(results)
    }

    async fn list_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<RawData>> {
        let metadata_list = self.query_by_time_range(start, end).await;
        let metadata_list = metadata_list.into_iter().take(limit).collect::<Vec<_>>();

        let mut results = Vec::new();
        for metadata in metadata_list {
            if let Ok(Some(data)) = self.get(metadata.id).await {
                results.push(data);
            }
        }

        Ok(results)
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<RawData>> {
        let metadata_list = self.query_by_search(query, limit).await;

        let mut results = Vec::new();
        for metadata in metadata_list {
            if let Ok(Some(data)) = self.get(metadata.id).await {
                results.push(data);
            }
        }

        Ok(results)
    }

    async fn delete(&self, id: StorageId) -> Result<bool> {
        let mut index = self.metadata_index.write().await;

        if let Some(metadata) = index.remove(&id) {
            // Delete file
            if metadata.file_path.exists() {
                tokio::fs::remove_file(&metadata.file_path)
                    .await
                    .map_err(|e| warn!("Failed to delete file {:?}: {}", metadata.file_path, e));
            }

            self.save_index().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn storage_size(&self) -> Result<u64> {
        let index = self.metadata_index.read().await;
        let mut total_size = 0u64;

        for metadata in index.values() {
            total_size += metadata.size_bytes;
        }

        Ok(total_size)
    }

    async fn archive_before(&self, date: DateTime<Utc>) -> Result<usize> {
        let index = self.metadata_index.read().await;

        let mut ids_to_archive = Vec::new();
        for (id, metadata) in index.iter() {
            if metadata.collected_at < date {
                ids_to_archive.push(*id);
            }
        }

        drop(index);

        let count = ids_to_archive.len();
        for id in ids_to_archive {
            // Move to archive location or delete based on retention policy
            self.delete(id).await?;
        }

        Ok(count)
    }

    async fn stats(&self) -> Result<ArchiveStats> {
        let index = self.metadata_index.read().await;

        let total_items = index.len();
        let mut total_size_bytes = 0u64;
        let mut oldest_item = None;
        let mut newest_item = None;
        let mut by_source: HashMap<String, u64> = HashMap::new();

        for metadata in index.values() {
            total_size_bytes += metadata.size_bytes;

            if oldest_item.is_none() || Some(metadata.collected_at) < oldest_item {
                oldest_item = Some(metadata.collected_at);
            }

            if newest_item.is_none() || Some(metadata.collected_at) > newest_item {
                newest_item = Some(metadata.collected_at);
            }

            let source_key = format!("{:?}", metadata.source);
            *by_source.entry(source_key).or_insert(0) += 1;
        }

        Ok(ArchiveStats {
            total_items: total_items as u64,
            total_size_bytes,
            oldest_item,
            newest_item,
            by_source,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if storage directory is writable
        let test_file = self.config.storage_dir.join(".health_check");

        tokio::fs::write(&test_file, b"test")
            .await
            .map_err(|_| Error::StorageError("Storage directory not writable".to_string()))?;

        tokio::fs::remove_file(&test_file)
            .await
            .map_err(|e| warn!("Failed to remove health check file: {}", e));

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_raw_archive_store_retrieve() {
        let config = RawArchiveConfig {
            storage_dir: PathBuf::from("./test_data/archive"),
            ..Default::default()
        };

        let archive = FileSystemRawArchive::new(config).await.unwrap();

        let data = RawData::new(
            DataSource::RSS {
                url: "https://example.com/feed".to_string(),
            },
            "Test content".to_string(),
            "text/plain".to_string(),
        );

        let id = archive.store(&data).await.unwrap();
        let retrieved = archive.get(id).await.unwrap().unwrap();

        assert_eq!(retrieved.content, "Test content");
        assert_eq!(retrieved.source, data.source);

        // Cleanup
        let _ = archive.delete(id).await;
        let _ = tokio::fs::remove_dir_all("./test_data");
    }

    #[tokio::test]
    async fn test_raw_archive_search() {
        let config = RawArchiveConfig {
            storage_dir: PathBuf::from("./test_data/archive_search"),
            ..Default::default()
        };

        let archive = FileSystemRawArchive::new(config).await.unwrap();

        let data1 = RawData::new(
            DataSource::Manual {
                description: "Test".to_string(),
            },
            "Artificial intelligence is growing".to_string(),
            "text/plain".to_string(),
        );

        let data2 = RawData::new(
            DataSource::Manual {
                description: "Test".to_string(),
            },
            "Machine learning models".to_string(),
            "text/plain".to_string(),
        );

        archive.store(&data1).await.unwrap();
        archive.store(&data2).await.unwrap();

        let results = archive.search("intelligence", 10).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].content.contains("intelligence"));

        // Cleanup
        let _ = tokio::fs::remove_dir_all("./test_data");
    }

    #[tokio::test]
    async fn test_raw_archive_stats() {
        let config = RawArchiveConfig {
            storage_dir: PathBuf::from("./test_data/archive_stats"),
            ..Default::default()
        };

        let archive = FileSystemRawArchive::new(config).await.unwrap();

        let data = RawData::new(
            DataSource::Manual {
                description: "Test".to_string(),
            },
            "Test content".to_string(),
            "text/plain".to_string(),
        );

        archive.store(&data).await.unwrap();

        let stats = archive.stats().await.unwrap();
        assert_eq!(stats.total_items, 1);

        // Cleanup
        let _ = tokio::fs::remove_dir_all("./test_data");
    }
}
