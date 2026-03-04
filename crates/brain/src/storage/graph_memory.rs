//! Graph Memory implementation using petgraph
//!
//! This module provides:
//! - In-memory knowledge graph using petgraph
//! - Entity and relationship management
//! - Graph traversal algorithms
//! - Path finding
//! - Subgraph extraction
//! - Clean interface for future Neo4j integration

use crate::storage::{GraphMemoryBackend, GraphStats};
use common::memory::{GraphEdge, GraphNode, GraphResponse};
use common::{Error, Result};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Graph edge with internal petgraph representation
#[derive(Debug, Clone)]
struct InternalEdge {
    id: String,
    from: NodeIndex,
    to: NodeIndex,
    edge: GraphEdge,
}

/// In-memory graph store
#[derive(Debug)]
pub struct InMemoryGraphStore {
    /// Directed graph
    graph: Arc<RwLock<DiGraph<GraphNode, InternalEdge>>>,

    /// Mapping from node ID to node index
    node_indices: Arc<RwLock<HashMap<String, NodeIndex>>>,

    /// Mapping from edge ID to (from_index, to_index)
    edge_indices: Arc<RwLock<HashMap<String, (NodeIndex, NodeIndex)>>>,
}

impl InMemoryGraphStore {
    /// Create a new in-memory graph store
    pub fn new() -> Self {
        info!("Creating in-memory graph store");

        Self {
            graph: Arc::new(RwLock::new(DiGraph::new())),
            node_indices: Arc::new(RwLock::new(HashMap::new())),
            edge_indices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate unique ID
    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Find shortest path using simple BFS
    async fn find_shortest_path_internal(&self, from: &str, to: &str) -> Result<Vec<GraphEdge>> {
        let indices = self.node_indices.read().await;
        let graph = self.graph.read().await;

        let from_idx = *indices
            .get(from)
            .ok_or_else(|| Error::MemoryError(format!("Source node '{}' not found", from)))?;
        let to_idx = *indices
            .get(to)
            .ok_or_else(|| Error::MemoryError(format!("Target node '{}' not found", to)))?;

        // Simple BFS to find path
        let mut queue = std::collections::VecDeque::new();
        let mut visited: std::collections::HashSet<NodeIndex> = std::collections::HashSet::new();
        let mut parent: std::collections::HashMap<
            NodeIndex,
            (NodeIndex, petgraph::graph::EdgeIndex),
        > = std::collections::HashMap::new();

        queue.push_back(from_idx);
        visited.insert(from_idx);

        let mut found = false;
        while let Some(current) = queue.pop_front() {
            if current == to_idx {
                found = true;
                break;
            }

            for neighbor in graph.neighbors(current) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    if let Some(edge_idx) = graph.find_edge(current, neighbor) {
                        parent.insert(neighbor, (current, edge_idx));
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if !found {
            return Err(Error::MemoryError("No path found".to_string()));
        }

        // Reconstruct path
        let mut edges = Vec::new();
        let mut current = to_idx;

        while current != from_idx {
            if let Some((pred, edge_idx)) = parent.get(&current) {
                edges.push(graph[*edge_idx].edge.clone());
                current = *pred;
            } else {
                break;
            }
        }

        edges.reverse();
        Ok(edges)
    }

    /// Get neighbors of a node
    async fn get_neighbors(&self, node_id: &str) -> Result<(Vec<GraphNode>, Vec<GraphEdge>)> {
        let indices = self.node_indices.read().await;
        let graph = self.graph.read().await;

        let idx = indices
            .get(node_id)
            .ok_or_else(|| Error::MemoryError(format!("Node '{}' not found", node_id)))?;

        let mut neighbor_nodes = Vec::new();
        let mut neighbor_edges = Vec::new();

        // Outgoing neighbors
        for neighbor in graph.neighbors(*idx) {
            neighbor_nodes.push(graph[neighbor].clone());
            // Find the edge
            if let Some(edge_idx) = graph.find_edge(*idx, neighbor) {
                neighbor_edges.push(graph[edge_idx].edge.clone());
            }
        }

        // Incoming neighbors
        for neighbor in graph.neighbors_directed(*idx, Direction::Incoming) {
            let source_node = &graph[neighbor];
            if !neighbor_nodes.iter().any(|n| n.id == source_node.id) {
                neighbor_nodes.push(source_node.clone());
            }
            if let Some(edge_idx) = graph.find_edge(neighbor, *idx) {
                let edge_data = &graph[edge_idx].edge;
                if !neighbor_edges.iter().any(|e| e.id == edge_data.id) {
                    neighbor_edges.push(edge_data.clone());
                }
            }
        }

        Ok((neighbor_nodes, neighbor_edges))
    }
}

impl Default for InMemoryGraphStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl GraphMemoryBackend for InMemoryGraphStore {
    async fn add_node(&self, mut node: GraphNode) -> Result<String> {
        debug!("Adding node with labels: {:?}", node.labels);

        // Generate ID if not provided
        if node.id.is_empty() {
            node.id = Self::generate_id();
        }

        let mut graph = self.graph.write().await;
        let mut indices = self.node_indices.write().await;

        // Check if node already exists
        if indices.contains_key(&node.id) {
            return Err(Error::MemoryError(format!(
                "Node with id '{}' already exists",
                node.id
            )));
        }

        // Add node to graph
        let idx = graph.add_node(node.clone());
        indices.insert(node.id.clone(), idx);

        Ok(node.id)
    }

    async fn add_edge(&self, mut edge: GraphEdge) -> Result<String> {
        debug!("Adding edge: {} -> {} ({})", edge.from, edge.to, edge.label);

        // Generate ID if not provided
        if edge.id.is_empty() {
            edge.id = Self::generate_id();
        }

        let mut indices = self.node_indices.write().await;

        // Get source and target node indices
        let from_idx = *indices
            .get(&edge.from)
            .ok_or_else(|| Error::MemoryError(format!("Source node '{}' not found", edge.from)))?;

        let to_idx = *indices
            .get(&edge.to)
            .ok_or_else(|| Error::MemoryError(format!("Target node '{}' not found", edge.to)))?;

        let mut graph = self.graph.write().await;
        let mut edge_indices = self.edge_indices.write().await;

        // Check if edge already exists
        if edge_indices.contains_key(&edge.id) {
            return Err(Error::MemoryError(format!(
                "Edge with id '{}' already exists",
                edge.id
            )));
        }

        // Add edge to graph
        let internal_edge = InternalEdge {
            id: edge.id.clone(),
            from: from_idx,
            to: to_idx,
            edge: edge.clone(),
        };

        graph.add_edge(from_idx, to_idx, internal_edge);
        edge_indices.insert(edge.id.clone(), (from_idx, to_idx));

        Ok(edge.id)
    }

    async fn get_node(&self, id: &str) -> Result<Option<GraphNode>> {
        let indices = self.node_indices.read().await;
        let graph = self.graph.read().await;

        if let Some(&idx) = indices.get(id) {
            Ok(Some(graph[idx].clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_edges(&self, node_id: &str) -> Result<Vec<GraphEdge>> {
        let indices = self.node_indices.read().await;
        let graph = self.graph.read().await;

        let idx = indices
            .get(node_id)
            .ok_or_else(|| Error::MemoryError(format!("Node '{}' not found", node_id)))?;

        let mut edges = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Outgoing edges
        for neighbor in graph.neighbors(*idx) {
            if let Some(edge_idx) = graph.find_edge(*idx, neighbor) {
                let edge_id = &graph[edge_idx].edge.id;
                if !seen.contains(edge_id) {
                    seen.insert(edge_id.clone());
                    edges.push(graph[edge_idx].edge.clone());
                }
            }
        }

        // Incoming edges
        for neighbor in graph.neighbors_directed(*idx, Direction::Incoming) {
            if let Some(edge_idx) = graph.find_edge(neighbor, *idx) {
                let edge_id = &graph[edge_idx].edge.id;
                if !seen.contains(edge_id) {
                    seen.insert(edge_id.clone());
                    edges.push(graph[edge_idx].edge.clone());
                }
            }
        }

        Ok(edges)
    }

    async fn find_nodes(&self, label: &str, limit: usize) -> Result<Vec<GraphNode>> {
        let graph = self.graph.read().await;

        let mut nodes = Vec::new();

        for node in graph.node_indices() {
            let graph_node = &graph[node];
            if graph_node.labels.contains(&label.to_string()) {
                nodes.push(graph_node.clone());
                if nodes.len() >= limit {
                    break;
                }
            }
        }

        Ok(nodes)
    }

    async fn find_path(&self, from: &str, to: &str) -> Result<Vec<GraphEdge>> {
        self.find_shortest_path_internal(from, to).await
    }

    async fn explore(&self, center_id: &str, depth: usize) -> Result<GraphResponse> {
        debug!(
            "Exploring graph around '{}' with depth {}",
            center_id, depth
        );

        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut queue = vec![(center_id.to_string(), 0)];

        let indices = self.node_indices.read().await;
        let graph = self.graph.read().await;

        // Validate center node exists
        if !indices.contains_key(center_id) {
            return Err(Error::MemoryError(format!(
                "Center node '{}' not found",
                center_id
            )));
        }

        while let Some((node_id, current_depth)) = queue.pop() {
            if current_depth > depth {
                continue;
            }

            if let Some(&idx) = indices.get(&node_id) {
                // Add node if not already present
                let node = graph[idx].clone();
                if !nodes.iter().any(|n| n.id == node.id) {
                    nodes.push(node);
                }

                // Add edges and enqueue neighbors - outgoing
                for neighbor in graph.neighbors(idx) {
                    if let Some(edge_idx) = graph.find_edge(idx, neighbor) {
                        let edge_data = &graph[edge_idx].edge;
                        if !edges.iter().any(|e| e.id == edge_data.id) {
                            edges.push(edge_data.clone());
                        }

                        let neighbor_node = &graph[neighbor];
                        let neighbor_id = &neighbor_node.id;
                        if !nodes.iter().any(|n| n.id == *neighbor_id) {
                            queue.push((neighbor_id.clone(), current_depth + 1));
                        }
                    }
                }

                // Also check incoming edges
                for neighbor in graph.neighbors_directed(idx, Direction::Incoming) {
                    if let Some(edge_idx) = graph.find_edge(neighbor, idx) {
                        let edge_data = &graph[edge_idx].edge;
                        if !edges.iter().any(|e| e.id == edge_data.id) {
                            edges.push(edge_data.clone());
                        }

                        let neighbor_node = &graph[neighbor];
                        let neighbor_id = &neighbor_node.id;
                        if !nodes.iter().any(|n| n.id == *neighbor_id) {
                            queue.push((neighbor_id.clone(), current_depth + 1));
                        }
                    }
                }
            }
        }

        Ok(GraphResponse {
            center_entity: center_id.to_string(),
            nodes,
            edges,
        })
    }

    async fn search_nodes(
        &self,
        property: &str,
        value: &str,
        limit: usize,
    ) -> Result<Vec<GraphNode>> {
        let graph = self.graph.read().await;

        let mut nodes = Vec::new();

        for node_idx in graph.node_indices() {
            let node = &graph[node_idx];

            // Search in properties
            if let Some(prop_value) = node.properties.get(property) {
                if prop_value.as_str() == Some(value) {
                    nodes.push(node.clone());
                    if nodes.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(nodes)
    }

    async fn delete_node(&self, id: &str) -> Result<bool> {
        let mut indices = self.node_indices.write().await;
        let mut edge_indices = self.edge_indices.write().await;
        let mut graph = self.graph.write().await;

        if let Some(&idx) = indices.get(id) {
            // Remove all edges connected to this node
            let edges_to_remove: Vec<_> = graph.edges(idx).map(|e| e.weight().id.clone()).collect();

            for edge_id in edges_to_remove {
                edge_indices.remove(&edge_id);
            }

            // Remove node
            indices.remove(id);
            graph.remove_node(idx);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_edge(&self, id: &str) -> Result<bool> {
        let mut edge_indices = self.edge_indices.write().await;
        let mut graph = self.graph.write().await;

        if let Some((from_idx, to_idx)) = edge_indices.remove(id) {
            // Find and remove the edge
            if let Some(edge_idx) = graph.find_edge(from_idx, to_idx) {
                // Check if this is the right edge by comparing IDs
                let edge = &graph[edge_idx];
                if edge.id == id {
                    graph.remove_edge(edge_idx);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn stats(&self) -> Result<GraphStats> {
        let graph = self.graph.read().await;

        let node_labels = {
            let mut labels = HashSet::new();
            for node_idx in graph.node_indices() {
                for label in &graph[node_idx].labels {
                    labels.insert(label.clone());
                }
            }
            labels.into_iter().collect()
        };

        let edge_types = {
            let mut types = HashSet::new();
            for edge_idx in graph.edge_indices() {
                types.insert(graph[edge_idx].edge.label.clone());
            }
            types.into_iter().collect()
        };

        Ok(GraphStats {
            total_nodes: graph.node_count(),
            total_edges: graph.edge_count(),
            node_labels,
            edge_types,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_node(id: &str, labels: Vec<String>) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            labels,
            properties: HashMap::new(),
        }
    }

    fn create_test_edge(id: &str, from: &str, to: &str, label: &str) -> GraphEdge {
        GraphEdge {
            id: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
            properties: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_graph_basic() {
        let store = InMemoryGraphStore::new();

        let node1_id = store
            .add_node(create_test_node("node1", vec!["Person".to_string()]))
            .await
            .unwrap();

        let node = store.get_node(&node1_id).await.unwrap().unwrap();
        assert_eq!(node.id, "node1");
        assert!(node.labels.contains(&"Person".to_string()));
    }

    #[tokio::test]
    async fn test_graph_edges() {
        let store = InMemoryGraphStore::new();

        let node1 = store
            .add_node(create_test_node("node1", vec!["Person".to_string()]))
            .await
            .unwrap();

        let node2 = store
            .add_node(create_test_node("node2", vec!["Company".to_string()]))
            .await
            .unwrap();

        let _edge = store
            .add_edge(create_test_edge("edge1", &node1, &node2, "WORKS_FOR"))
            .await
            .unwrap();

        let edges = store.get_edges(&node1).await.unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].label, "WORKS_FOR");
    }

    #[tokio::test]
    async fn test_find_nodes_by_label() {
        let store = InMemoryGraphStore::new();

        store
            .add_node(create_test_node("p1", vec!["Person".to_string()]))
            .await
            .unwrap();
        store
            .add_node(create_test_node("p2", vec!["Person".to_string()]))
            .await
            .unwrap();
        store
            .add_node(create_test_node("c1", vec!["Company".to_string()]))
            .await
            .unwrap();

        let people = store.find_nodes("Person", 10).await.unwrap();
        assert_eq!(people.len(), 2);

        let companies = store.find_nodes("Company", 10).await.unwrap();
        assert_eq!(companies.len(), 1);
    }

    #[tokio::test]
    async fn test_graph_explore() {
        let store = InMemoryGraphStore::new();

        let node1 = store
            .add_node(create_test_node("n1", vec!["Person".to_string()]))
            .await
            .unwrap();
        let node2 = store
            .add_node(create_test_node("n2", vec!["Person".to_string()]))
            .await
            .unwrap();
        let node3 = store
            .add_node(create_test_node("n3", vec!["Company".to_string()]))
            .await
            .unwrap();

        store
            .add_edge(create_test_edge("e1", &node1, &node2, "KNOWS"))
            .await
            .unwrap();
        store
            .add_edge(create_test_edge("e2", &node2, &node3, "WORKS_FOR"))
            .await
            .unwrap();

        let response = store.explore(&node1, 2).await.unwrap();

        assert_eq!(response.center_entity, node1);
        assert_eq!(response.nodes.len(), 3); // All nodes
        assert_eq!(response.edges.len(), 2); // Both edges
    }

    #[tokio::test]
    async fn test_graph_stats() {
        let store = InMemoryGraphStore::new();

        store
            .add_node(create_test_node("n1", vec!["Person".to_string()]))
            .await
            .unwrap();
        store
            .add_node(create_test_node("n2", vec!["Person".to_string()]))
            .await
            .unwrap();

        store
            .add_edge(create_test_edge("e1", "n1", "n2", "KNOWS"))
            .await
            .unwrap();

        let stats = store.stats().await.unwrap();
        assert_eq!(stats.total_nodes, 2);
        assert_eq!(stats.total_edges, 1);
        assert!(stats.node_labels.contains(&"Person".to_string()));
        assert!(stats.edge_types.contains(&"KNOWS".to_string()));
    }

    #[tokio::test]
    async fn test_delete_node() {
        let store = InMemoryGraphStore::new();

        let node1 = store
            .add_node(create_test_node("n1", vec!["Person".to_string()]))
            .await
            .unwrap();
        let node2 = store
            .add_node(create_test_node("n2", vec!["Person".to_string()]))
            .await
            .unwrap();

        store
            .add_edge(create_test_edge("e1", &node1, &node2, "KNOWS"))
            .await
            .unwrap();

        assert!(store.delete_node(&node1).await.unwrap());
        assert!(!store.delete_node(&node1).await.unwrap()); // Already deleted

        // Check that edge was also removed
        let edges = store.get_edges(&node2).await.unwrap();
        assert_eq!(edges.len(), 0);
    }
}
