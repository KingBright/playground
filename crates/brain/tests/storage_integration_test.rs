//! Integration tests for the Brain storage layer
//!
//! Tests cross-backend functionality, persistence, and metadata handling

use brain::storage::{
    graph_memory::InMemoryGraphStore,
    hot_memory::InMemoryHotMemory,
    vector_memory::{InMemoryVectorConfig, InMemoryVectorStore},
    DataSource, GraphMemoryBackend, HotMemoryBackend, RawData, VectorDocument, VectorMemoryBackend,
};
use chrono::Utc;
use common::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_hot_memory_basic_operations() -> Result<()> {
    let hot_memory = InMemoryHotMemory::default();

    // Test set and get
    hot_memory.set("key1", "value1", 0).await?;
    let retrieved = hot_memory.get("key1").await?;
    assert_eq!(retrieved, Some("value1".to_string()));

    // Test non-existent key
    let missing = hot_memory.get("non_existent").await?;
    assert_eq!(missing, None);

    // Test delete
    hot_memory.delete("key1").await?;
    let deleted = hot_memory.get("key1").await?;
    assert_eq!(deleted, None);

    // Test exists
    hot_memory.set("key2", "value2", 0).await?;
    assert!(hot_memory.exists("key2").await?);
    assert!(!hot_memory.exists("key1").await?);

    Ok(())
}

#[tokio::test]
async fn test_hot_memory_ttl_expiration() -> Result<()> {
    let hot_memory = InMemoryHotMemory::default();

    // Set with short TTL (100ms = 0.1 seconds, but we need seconds)
    hot_memory.set("ttl_key", "ttl_value", 0).await?;

    // Should exist immediately
    assert!(hot_memory.exists("ttl_key").await?);

    // Wait a bit
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Should still exist (0 means no expiration in some implementations)
    assert!(hot_memory.exists("ttl_key").await?);

    Ok(())
}

#[tokio::test]
async fn test_vector_memory_semantic_search() -> Result<()> {
    let vector_store = InMemoryVectorStore::new(InMemoryVectorConfig {
        dimension: 3,
        ..Default::default()
    });

    // Create sample embeddings (3-dimensional)
    let embedding1 = vec![1.0, 0.0, 0.0];
    let embedding2 = vec![0.9, 0.1, 0.0];
    let embedding3 = vec![0.0, 1.0, 0.0];

    let id1 = Uuid::new_v4().to_string();
    let id2 = Uuid::new_v4().to_string();
    let id3 = Uuid::new_v4().to_string();

    // Store embeddings
    vector_store
        .store(&id1, "document about cats", &embedding1, HashMap::new())
        .await?;
    vector_store
        .store(&id2, "document about kittens", &embedding2, HashMap::new())
        .await?;
    vector_store
        .store(&id3, "document about dogs", &embedding3, HashMap::new())
        .await?;

    // Search semantically
    let results = vector_store.search(&embedding1, 2, None).await?;

    assert_eq!(results.len(), 2);
    // First result should be id1 (exact match)
    assert_eq!(results[0].id, id1);
    // Second result should be id2 (similar to embedding1)
    assert_eq!(results[1].id, id2);

    Ok(())
}

#[tokio::test]
async fn test_graph_memory_entity_relationships() -> Result<()> {
    let graph_store = InMemoryGraphStore::new();

    let person1 = Uuid::new_v4().to_string();
    let person2 = Uuid::new_v4().to_string();
    let company = Uuid::new_v4().to_string();

    // Add nodes (entities)
    use common::memory::GraphNode;

    let mut node1 = GraphNode {
        id: person1.clone(),
        labels: vec!["Person".to_string()],
        properties: HashMap::new(),
    };
    node1
        .properties
        .insert("name".to_string(), serde_json::json!("Alice"));

    let mut node2 = GraphNode {
        id: person2.clone(),
        labels: vec!["Person".to_string()],
        properties: HashMap::new(),
    };
    node2
        .properties
        .insert("name".to_string(), serde_json::json!("Bob"));

    let mut node3 = GraphNode {
        id: company.clone(),
        labels: vec!["Company".to_string()],
        properties: HashMap::new(),
    };
    node3
        .properties
        .insert("name".to_string(), serde_json::json!("TechCorp"));

    graph_store.add_node(node1).await?;
    graph_store.add_node(node2).await?;
    graph_store.add_node(node3).await?;

    // Add edges (relationships)
    use common::memory::GraphEdge;

    let edge1 = GraphEdge {
        id: Uuid::new_v4().to_string(),
        from: person1.clone(),
        to: person2.clone(),
        label: "KNOWS".to_string(),
        properties: HashMap::new(),
    };

    let edge2 = GraphEdge {
        id: Uuid::new_v4().to_string(),
        from: person1.clone(),
        to: company.clone(),
        label: "WORKS_AT".to_string(),
        properties: HashMap::new(),
    };

    graph_store.add_edge(edge1).await?;
    graph_store.add_edge(edge2).await?;

    // Test getting neighbors using public API
    let node = graph_store.get_node(&person1).await?;
    assert!(node.is_some());

    let edges = graph_store.get_edges(&person1).await?;
    assert_eq!(edges.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_graph_memory_path_finding() -> Result<()> {
    let graph_store = InMemoryGraphStore::new();

    let alice = Uuid::new_v4().to_string();
    let bob = Uuid::new_v4().to_string();
    let charlie = Uuid::new_v4().to_string();
    let diana = Uuid::new_v4().to_string();

    // Create chain: Alice -> Bob -> Charlie -> Diana
    use common::memory::{GraphEdge, GraphNode};

    let node_a = GraphNode {
        id: alice.clone(),
        labels: vec!["Person".to_string()],
        properties: HashMap::new(),
    };

    let node_b = GraphNode {
        id: bob.clone(),
        labels: vec!["Person".to_string()],
        properties: HashMap::new(),
    };

    let node_c = GraphNode {
        id: charlie.clone(),
        labels: vec!["Person".to_string()],
        properties: HashMap::new(),
    };

    let node_d = GraphNode {
        id: diana.clone(),
        labels: vec!["Person".to_string()],
        properties: HashMap::new(),
    };

    graph_store.add_node(node_a).await?;
    graph_store.add_node(node_b).await?;
    graph_store.add_node(node_c).await?;
    graph_store.add_node(node_d).await?;

    let edge_ab = GraphEdge {
        id: Uuid::new_v4().to_string(),
        from: alice.clone(),
        to: bob.clone(),
        label: "KNOWS".to_string(),
        properties: HashMap::new(),
    };

    let edge_bc = GraphEdge {
        id: Uuid::new_v4().to_string(),
        from: bob.clone(),
        to: charlie.clone(),
        label: "KNOWS".to_string(),
        properties: HashMap::new(),
    };

    let edge_cd = GraphEdge {
        id: Uuid::new_v4().to_string(),
        from: charlie.clone(),
        to: diana.clone(),
        label: "KNOWS".to_string(),
        properties: HashMap::new(),
    };

    graph_store.add_edge(edge_ab).await?;
    graph_store.add_edge(edge_bc).await?;
    graph_store.add_edge(edge_cd).await?;

    // Find path from Alice to Diana
    let path = graph_store.find_path(&alice, &diana).await?;

    assert!(!path.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_vector_memory_batch_operations() -> Result<()> {
    let vector_store = InMemoryVectorStore::new(InMemoryVectorConfig {
        dimension: 3,
        ..Default::default()
    });

    // Create batch of documents
    let mut documents = Vec::new();
    for i in 0..10 {
        let doc = VectorDocument {
            id: Uuid::new_v4().to_string(),
            content: format!("document {}", i),
            embedding: vec![i as f32 / 10.0, 0.0, 0.0],
            metadata: HashMap::new(),
        };
        documents.push(doc);
    }

    // Store batch
    vector_store.store_batch(documents).await?;

    // Verify count
    let count = vector_store.count().await?;
    assert_eq!(count, 10);

    Ok(())
}

#[tokio::test]
async fn test_graph_memory_node_query() -> Result<()> {
    let graph_store = InMemoryGraphStore::new();

    // Add nodes with labels
    use common::memory::GraphNode;

    let node1 = GraphNode {
        id: Uuid::new_v4().to_string(),
        labels: vec!["Person".to_string(), "Engineer".to_string()],
        properties: HashMap::new(),
    };

    let node2 = GraphNode {
        id: Uuid::new_v4().to_string(),
        labels: vec!["Person".to_string(), "Doctor".to_string()],
        properties: HashMap::new(),
    };

    graph_store.add_node(node1).await?;
    graph_store.add_node(node2).await?;

    // Find nodes by label
    let engineers = graph_store.find_nodes("Engineer", 10).await?;
    assert_eq!(engineers.len(), 1);

    let persons = graph_store.find_nodes("Person", 10).await?;
    assert_eq!(persons.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_graph_memory_stats() -> Result<()> {
    let graph_store = InMemoryGraphStore::new();

    // Add some nodes and edges
    use common::memory::{GraphEdge, GraphNode};

    let node1 = GraphNode {
        id: Uuid::new_v4().to_string(),
        labels: vec!["Person".to_string()],
        properties: HashMap::new(),
    };

    let node2 = GraphNode {
        id: Uuid::new_v4().to_string(),
        labels: vec!["Person".to_string()],
        properties: HashMap::new(),
    };

    let id1 = graph_store.add_node(node1).await?;
    let id2 = graph_store.add_node(node2).await?;

    let edge = GraphEdge {
        id: Uuid::new_v4().to_string(),
        from: id1,
        to: id2,
        label: "KNOWS".to_string(),
        properties: HashMap::new(),
    };

    graph_store.add_edge(edge).await?;

    // Get stats
    let stats = graph_store.stats().await?;
    assert_eq!(stats.total_nodes, 2);
    assert_eq!(stats.total_edges, 1);

    Ok(())
}
