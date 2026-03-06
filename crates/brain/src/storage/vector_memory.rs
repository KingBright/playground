//! Vector Memory implementation with semantic search
//!
//! This module provides:
//! - In-memory vector store using cosine similarity
//! - Production-ready indexing for moderate datasets
//! - Clean interface for future SQLite-vec integration
//! - Efficient top-K search
//! - Metadata filtering

use crate::storage::{SearchFilters, VectorDocument, VectorMemoryBackend, VectorSearchResult};
use common::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// In-memory vector store configuration
#[derive(Debug, Clone)]
pub struct InMemoryVectorConfig {
    /// Embedding dimension
    pub dimension: usize,

    /// Similarity threshold (0-1)
    pub similarity_threshold: f64,

    /// Maximum documents before triggering indexing
    pub index_threshold: usize,
}

impl Default for InMemoryVectorConfig {
    fn default() -> Self {
        Self {
            dimension: 1536, // OpenAI default
            similarity_threshold: 0.7,
            index_threshold: 1000,
        }
    }
}

/// In-memory vector store implementation
#[derive(Debug)]
pub struct InMemoryVectorStore {
    config: InMemoryVectorConfig,
    documents: Arc<RwLock<HashMap<String, VectorDocument>>>,
    /// Index: tag -> document IDs
    tag_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new(config: InMemoryVectorConfig) -> Self {
        info!(
            "Creating in-memory vector store with dimension {}",
            config.dimension
        );

        Self {
            config,
            documents: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(InMemoryVectorConfig::default())
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
        if a.len() != b.len() {
            warn!("Vector dimension mismatch: {} vs {}", a.len(), b.len());
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        (dot_product / (norm_a * norm_b)) as f64
    }

    /// Update tag index
    async fn update_tag_index(&self, doc_id: &str, tags: &[String]) {
        let mut index = self.tag_index.write().await;
        for tag in tags {
            index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(doc_id.to_string());
        }
    }

    /// Filter documents by tags
    async fn filter_by_tags(&self, candidate_ids: &mut Vec<String>, tags: &[String]) {
        if tags.is_empty() {
            return;
        }

        let index = self.tag_index.read().await;

        // Get candidate IDs for each tag
        let mut tag_sets: Vec<std::collections::HashSet<String>> = Vec::new();
        for tag in tags {
            if let Some(ids) = index.get(tag) {
                tag_sets.push(ids.iter().cloned().collect());
            }
        }

        if tag_sets.is_empty() {
            candidate_ids.clear();
            return;
        }

        // Union of all tag sets (OR logic)
        let union_set: std::collections::HashSet<_> = tag_sets.into_iter().flatten().collect();

        candidate_ids.retain(|id| union_set.contains(id));
    }

    /// Filter documents by metadata
    async fn filter_by_metadata(
        &self,
        candidate_ids: &mut Vec<String>,
        filters: &HashMap<String, String>,
    ) {
        if filters.is_empty() {
            return;
        }

        let docs = self.documents.read().await;

        candidate_ids.retain(|id| {
            if let Some(doc) = docs.get(id) {
                for (key, value) in filters {
                    if doc.metadata.get(key).map_or(true, |v| v != value) {
                        return false;
                    }
                }
                true
            } else {
                false
            }
        });
    }
}

#[async_trait::async_trait]
impl VectorMemoryBackend for InMemoryVectorStore {
    async fn store(
        &self,
        id: &str,
        content: &str,
        embedding: &[f32],
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        debug!(
            "Storing document '{}' with embedding size {}",
            id,
            embedding.len()
        );

        if embedding.len() != self.config.dimension {
            return Err(Error::MemoryError(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.config.dimension,
                embedding.len()
            )));
        }

        let doc = VectorDocument {
            id: id.to_string(),
            content: content.to_string(),
            embedding: embedding.to_vec(),
            metadata,
        };

        // Extract tags for indexing
        let tags = doc
            .metadata
            .get("tags")
            .and_then(|t| serde_json::from_str::<Vec<String>>(t).ok())
            .unwrap_or_default();

        // Update tag index
        self.update_tag_index(id, &tags).await;

        // Store document
        let mut docs = self.documents.write().await;
        docs.insert(id.to_string(), doc);

        Ok(())
    }

    async fn store_batch(&self, documents: Vec<VectorDocument>) -> Result<()> {
        debug!("Storing batch of {} documents", documents.len());

        let mut docs = self.documents.write().await;
        let mut index = self.tag_index.write().await;

        for doc in documents {
            let id = doc.id.clone();

            // Validate embedding dimension
            if doc.embedding.len() != self.config.dimension {
                warn!(
                    "Skipping document '{}' due to embedding dimension mismatch",
                    id
                );
                continue;
            }

            // Update tag index
            if let Some(tags_str) = doc.metadata.get("tags") {
                if let Ok(tags) = serde_json::from_str::<Vec<String>>(tags_str) {
                    for tag in tags {
                        index.entry(tag).or_insert_with(Vec::new).push(id.clone());
                    }
                }
            }

            docs.insert(id, doc);
        }

        Ok(())
    }

    async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<VectorSearchResult>> {
        debug!("Searching for top {} documents with filters", limit);

        if query_embedding.len() != self.config.dimension {
            return Err(Error::MemoryError(format!(
                "Query embedding dimension mismatch: expected {}, got {}",
                self.config.dimension,
                query_embedding.len()
            )));
        }

        let docs = self.documents.read().await;

        // Get all document IDs
        let mut candidate_ids: Vec<String> = docs.keys().cloned().collect();

        // Apply filters
        if let Some(filters) = filters {
            // Filter by tags
            if let Some(tags) = filters.tags {
                self.filter_by_tags(&mut candidate_ids, &tags).await;
            }

            // Filter by metadata
            if let Some(metadata_filters) = filters.metadata {
                self.filter_by_metadata(&mut candidate_ids, &metadata_filters)
                    .await;
            }
        }

        // Calculate similarities
        let mut results: Vec<VectorSearchResult> = candidate_ids
            .into_iter()
            .filter_map(|id| {
                if let Some(doc) = docs.get(&id) {
                    let score = Self::cosine_similarity(query_embedding, &doc.embedding);

                    if score >= self.config.similarity_threshold {
                        Some(VectorSearchResult {
                            id: doc.id.clone(),
                            content: doc.content.clone(),
                            score,
                            metadata: doc.metadata.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(limit);

        debug!("Found {} results", results.len());

        Ok(results)
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let mut docs = self.documents.write().await;

        // Remove from tag index
        if let Some(doc) = docs.get(id) {
            if let Some(tags_str) = doc.metadata.get("tags") {
                if let Ok(tags) = serde_json::from_str::<Vec<String>>(tags_str) {
                    let mut index = self.tag_index.write().await;
                    for tag in tags {
                        if let Some(ids) = index.get_mut(&tag) {
                            ids.retain(|x| x != id);
                        }
                    }
                }
            }
        }

        Ok(docs.remove(id).is_some())
    }

    async fn get(&self, id: &str) -> Result<Option<VectorDocument>> {
        let docs = self.documents.read().await;
        Ok(docs.get(id).cloned())
    }

    async fn update(&self, id: &str, content: &str, embedding: &[f32]) -> Result<bool> {
        let mut docs = self.documents.write().await;

        if let Some(doc) = docs.get_mut(id) {
            doc.content = content.to_string();
            doc.embedding = embedding.to_vec();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn count(&self) -> Result<usize> {
        let docs = self.documents.read().await;
        Ok(docs.len())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

/// Simple dot-product similarity (alternative to cosine)
pub fn dot_product_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    dot_product as f64
}

/// Euclidean distance (alternative similarity metric)
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return f64::MAX;
    }

    let sum_sq: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) as f64 * (x - y) as f64)
        .sum();

    sum_sq.sqrt()
}

/// Convert distance to similarity (inverse relationship)
pub fn distance_to_similarity(distance: f64, max_distance: f64) -> f64 {
    if max_distance == 0.0 {
        return 1.0;
    }
    1.0 - (distance / max_distance).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_embedding(value: f32) -> Vec<f32> {
        // Create a vector where the first element is value and rest are slightly different
        // This ensures different vectors have different directions
        let mut vec = vec![value; 1536];
        vec[0] = value;
        vec[1] = value + 0.1;
        vec[2] = value - 0.05;
        vec
    }

    #[tokio::test]
    async fn test_vector_store_basic() {
        let store = InMemoryVectorStore::with_default_config();

        let embedding = create_test_embedding(0.5);

        store
            .store("doc1", "Test content", &embedding, HashMap::new())
            .await
            .unwrap();

        let retrieved = store.get("doc1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content, "Test content");
    }

    #[tokio::test]
    async fn test_vector_search() {
        let store = InMemoryVectorStore::with_default_config();

        // Store documents with different embeddings
        let embedding1 = create_test_embedding(1.0);
        let embedding2 = create_test_embedding(0.5);
        let embedding3 = create_test_embedding(0.0);

        store
            .store("doc1", "Content 1", &embedding1, HashMap::new())
            .await
            .unwrap();
        store
            .store("doc2", "Content 2", &embedding2, HashMap::new())
            .await
            .unwrap();
        store
            .store("doc3", "Content 3", &embedding3, HashMap::new())
            .await
            .unwrap();

        // Search with query similar to doc1
        let query = create_test_embedding(0.95);
        let results = store.search(&query, 2, None).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "doc1"); // Should be most similar
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let c = vec![-1.0, -2.0, -3.0];

        // Same vectors should have similarity 1.0
        let sim_ab = InMemoryVectorStore::cosine_similarity(&a, &b);
        assert!((sim_ab - 1.0).abs() < 0.001);

        // Opposite vectors should have similarity -1.0
        let sim_ac = InMemoryVectorStore::cosine_similarity(&a, &c);
        assert!((sim_ac - (-1.0)).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_tag_filtering() {
        let store = InMemoryVectorStore::with_default_config();

        let embedding = create_test_embedding(0.5);

        // Store documents with tags
        let mut metadata1 = HashMap::new();
        metadata1.insert(
            "tags".to_string(),
            serde_json::json!(["tech", "ai"]).to_string(),
        );

        let mut metadata2 = HashMap::new();
        metadata2.insert("tags".to_string(), serde_json::json!(["news"]).to_string());

        store
            .store("doc1", "Tech content", &embedding, metadata1.clone())
            .await
            .unwrap();
        store
            .store("doc2", "News content", &embedding, metadata2.clone())
            .await
            .unwrap();

        // Search with tag filter
        let query = create_test_embedding(0.5);
        let filters = SearchFilters {
            tags: Some(vec!["tech".to_string()]),
            time_range: None,
            metadata: None,
        };

        let results = store.search(&query, 10, Some(filters)).await.unwrap();

        // Should only return doc1
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc1");
    }

    #[tokio::test]
    async fn test_delete() {
        let store = InMemoryVectorStore::with_default_config();

        let embedding = create_test_embedding(0.5);
        store
            .store("doc1", "Test", &embedding, HashMap::new())
            .await
            .unwrap();

        assert!(store.delete("doc1").await.unwrap());
        assert!(!store.delete("doc1").await.unwrap()); // Already deleted

        assert!(store.get("doc1").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_count() {
        let store = InMemoryVectorStore::with_default_config();

        assert_eq!(store.count().await.unwrap(), 0);

        let embedding = create_test_embedding(0.5);
        store
            .store("doc1", "Test 1", &embedding, HashMap::new())
            .await
            .unwrap();
        store
            .store("doc2", "Test 2", &embedding, HashMap::new())
            .await
            .unwrap();

        assert_eq!(store.count().await.unwrap(), 2);
    }
}
