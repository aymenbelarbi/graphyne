//! Graph storage with typed property graph support using petgraph.
//!
//! This module provides graph database capabilities with support for typed nodes,
//! typed edges, node/edge properties, and multi-hop traversal.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use petgraph::graph::{DiGraph, NodeIndex, EdgeIndex};
use petgraph::visit::EdgeRef;
use petgraph::algo::dijkstra;
use serde::{Deserialize, Serialize};
use sled::Tree;
use thiserror::Error;

use crate::error::{Result, GraphyneError};

/// GraphRAG module for enhanced agent reasoning.
pub mod rag;

/// Errors specific to graph operations.
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum GraphError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("Edge not found: {0}")]
    EdgeNotFound(String),
    
    #[error("Invalid node or edge ID: {0}")]
    InvalidId(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Traversal error: {0}")]
    TraversalError(String),
}

/// A node in the property graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub properties: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// An edge in the property graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub properties: serde_json::Value,
    pub weight: f32,
}

/// A path resulting from graph traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub total_weight: f32,
}

/// Graph store with typed property graph support.
pub struct GraphStore {
    /// In-memory directed graph for fast traversal
    graph: DiGraph<GraphNode, GraphEdge>,
    /// Mapping from node ID to petgraph NodeIndex
    node_indices: HashMap<String, NodeIndex>,
    /// Mapping from edge ID to petgraph EdgeIndex
    edge_indices: HashMap<String, EdgeIndex>,
    /// Persistent storage for nodes
    nodes_store: Tree,
    /// Persistent storage for edges
    edges_store: Tree,
    /// Counter for generating unique node IDs
    node_counter: u64,
    /// Counter for generating unique edge IDs
    edge_counter: u64,
}

impl GraphStore {
    /// Iterate over all nodes in the graph.
    pub fn iter_nodes(&self) -> Vec<GraphNode> {
        self.graph
            .node_indices()
            .filter_map(|idx| self.graph.node_weight(idx).cloned())
            .collect()
    }

    /// Create a new graph store with the given sled database.
    pub fn new(db: &sled::Db) -> Result<Self> {
        let nodes_store = db.open_tree("graph_nodes")?;
        let edges_store = db.open_tree("graph_edges")?;
        
        let mut store = Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            edge_indices: HashMap::new(),
            nodes_store,
            edges_store,
            node_counter: 0,
            edge_counter: 0,
        };
        
        store.load_from_storage()?;
        Ok(store)
    }
    
    /// Load existing graph data from storage.
    fn load_from_storage(&mut self) -> Result<()> {
        // Load nodes
        for item in self.nodes_store.iter() {
            let (key, value) = item?;
            let node_id = String::from_utf8(key.to_vec()).map_err(|e| GraphyneError::Storage(e.to_string()))?;
            let node: GraphNode = serde_json::from_slice(&value)?;
            
            let idx = self.graph.add_node(node);
            self.node_indices.insert(node_id, idx);
        }
        
        // Load edges
        for item in self.edges_store.iter() {
            let (key, value) = item?;
            let edge_id = String::from_utf8(key.to_vec()).map_err(|e| GraphyneError::Storage(e.to_string()))?;
            let edge: GraphEdge = serde_json::from_slice(&value)?;
            
            if let (Some(&from_idx), Some(&to_idx)) = (
                self.node_indices.get(&edge.from),
                self.node_indices.get(&edge.to)
            ) {
                let idx = self.graph.add_edge(from_idx, to_idx, edge);
                self.edge_indices.insert(edge_id, idx);
            }
        }
        
        Ok(())
    }
    
    /// Generate a new unique node ID.
    fn generate_node_id(&mut self) -> String {
        self.node_counter += 1;
        format!("node_{}", self.node_counter)
    }
    
    /// Generate a new unique edge ID.
    fn generate_edge_id(&mut self) -> String {
        self.edge_counter += 1;
        format!("edge_{}", self.edge_counter)
    }
    
    /// Add a node to the graph.
    pub fn add_node(&mut self, id: &str, node_type: &str, label: &str, properties: serde_json::Value, embedding: Option<Vec<f32>>) -> Result<()> {
        let node = GraphNode {
            id: id.to_string(),
            node_type: node_type.to_string(),
            label: label.to_string(),
            properties,
            embedding,
            created_at: chrono::Utc::now(),
        };
        
        // Store in sled
        let key = id.as_bytes();
        let value = serde_json::to_vec(&node)?;
        self.nodes_store.insert(key, value)?;
        
        // Add to in-memory graph
        let idx = self.graph.add_node(node);
        self.node_indices.insert(id.to_string(), idx);
        
        Ok(())
    }
    
    /// Remove a node from the graph.
    pub fn remove_node(&mut self, id: &str) -> Result<()> {
        if let Some(&idx) = self.node_indices.get(id) {
            // Remove from sled
            self.nodes_store.remove(id.as_bytes())?;
            
            // Remove from in-memory graph (this also removes connected edges)
            self.graph.remove_node(idx);
            self.node_indices.remove(id);
            
            Ok(())
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(id.to_string()).to_string()))
        }
    }
    
    /// Add an edge to the graph.
    pub fn add_edge(&mut self, from: &str, to: &str, edge_type: &str, properties: serde_json::Value, weight: f32) -> Result<String> {
        let edge_id = self.generate_edge_id();
        
        let edge = GraphEdge {
            id: edge_id.clone(),
            from: from.to_string(),
            to: to.to_string(),
            edge_type: edge_type.to_string(),
            properties,
            weight,
        };
        
        if let (Some(&from_idx), Some(&to_idx)) = (
            self.node_indices.get(from),
            self.node_indices.get(to)
        ) {
            // Store in sled
            let key = edge_id.as_bytes();
            let value = serde_json::to_vec(&edge)?;
            self.edges_store.insert(key, value)?;
            
            // Add to in-memory graph
            let idx = self.graph.add_edge(from_idx, to_idx, edge);
            self.edge_indices.insert(edge_id.clone(), idx);
            
            Ok(edge_id)
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(
                if self.node_indices.get(from).is_none() { from.to_string() } else { to.to_string() }
            ).to_string()))
        }
    }
    
    /// Remove an edge from the graph.
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<()> {
        if let Some(&idx) = self.edge_indices.get(edge_id) {
            // Remove from sled
            self.edges_store.remove(edge_id.as_bytes())?;
            
            // Remove from in-memory graph
            self.graph.remove_edge(idx);
            self.edge_indices.remove(edge_id);
            
            Ok(())
        } else {
            Err(GraphyneError::Graph(GraphError::EdgeNotFound(edge_id.to_string()).to_string()))
        }
    }
    
    /// Get a node by ID.
    pub fn get_node(&self, id: &str) -> Result<Option<GraphNode>> {
        if let Some(&idx) = self.node_indices.get(id) {
            if let Some(node) = self.graph.node_weight(idx) {
                return Ok(Some(node.clone()));
            }
        }
        Ok(None)
    }
    
    /// Get an edge by ID.
    pub fn get_edge(&self, edge_id: &str) -> Result<Option<GraphEdge>> {
        if let Some(&idx) = self.edge_indices.get(edge_id) {
            if let Some(edge) = self.graph.edge_weight(idx) {
                return Ok(Some(edge.clone()));
            }
        }
        Ok(None)
    }
    
    /// Get all edges from a node.
    pub fn get_edges_from(&self, node_id: &str) -> Result<Vec<GraphEdge>> {
        if let Some(&idx) = self.node_indices.get(node_id) {
            let edges: Vec<GraphEdge> = self.graph
                .edges(idx)
                .filter_map(|edge_ref| {
                    self.graph.edge_weight(edge_ref.id()).cloned()
                })
                .collect();
            Ok(edges)
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(node_id.to_string()).to_string()))
        }
    }
    
    /// Get all edges to a node.
    pub fn get_edges_to(&self, node_id: &str) -> Result<Vec<GraphEdge>> {
        if let Some(&idx) = self.node_indices.get(node_id) {
            let edges: Vec<GraphEdge> = self.graph
                .edges_directed(idx, petgraph::Direction::Incoming)
                .filter_map(|edge_ref| {
                    self.graph.edge_weight(edge_ref.id()).cloned()
                })
                .collect();
            Ok(edges)
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(node_id.to_string()).to_string()))
        }
    }
    
    /// Traverse the graph starting from a node.
    pub fn traverse(&self, start: &str, max_hops: usize) -> Result<Vec<GraphPath>> {
        if let Some(&start_idx) = self.node_indices.get(start) {
            let mut visited = HashMap::new();
            let mut queue: VecDeque<(NodeIndex, Vec<NodeIndex>, Vec<EdgeIndex>, f32)> = VecDeque::new();
            let mut paths = Vec::new();
            
            queue.push_back((start_idx, vec![start_idx], vec![], 0.0));
            visited.insert(start_idx, 0);
            
            while let Some((current, nodes, edges, total_weight)) = queue.pop_front() {
                if nodes.len() > max_hops + 1 {
                    continue;
                }
                
                if nodes.len() > 1 {
                    // Build path
                    let path_nodes: Vec<GraphNode> = nodes
                        .iter()
                        .filter_map(|&idx| self.graph.node_weight(idx).cloned())
                        .collect();
                    
                    let path_edges: Vec<GraphEdge> = edges
                        .iter()
                        .filter_map(|&idx| self.graph.edge_weight(idx).cloned())
                        .collect();
                    
                    paths.push(GraphPath {
                        nodes: path_nodes,
                        edges: path_edges,
                        total_weight,
                    });
                }
                
                // Explore neighbors
                for edge_ref in self.graph.edges(current) {
                    let neighbor = edge_ref.target();
                    let edge_idx = edge_ref.id();
                    
                    if let Some(&dist) = visited.get(&neighbor) {
                        if dist <= nodes.len() {
                            continue;
                        }
                    }
                    
                    if let Some(edge) = self.graph.edge_weight(edge_idx) {
                        let mut new_nodes = nodes.clone();
                        new_nodes.push(neighbor);
                        
                        let mut new_edges = edges.clone();
                        new_edges.push(edge_idx);
                        
                        queue.push_back((
                            neighbor,
                            new_nodes,
                            new_edges,
                            total_weight + edge.weight,
                        ));
                        
                        visited.insert(neighbor, nodes.len());
                    }
                }
            }
            
            Ok(paths)
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(start.to_string()).to_string()))
        }
    }
    
    /// Find nodes by type.
    pub fn find_nodes_by_type(&self, node_type: &str) -> Vec<GraphNode> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let node = self.graph.node_weight(idx)?;
                if node.node_type == node_type {
                    Some(node.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Find edges by type.
    pub fn find_edges_by_type(&self, edge_type: &str) -> Vec<GraphEdge> {
        self.graph
            .edge_references()
            .filter_map(|edge_ref| {
                let edge = edge_ref.weight();
                if edge.edge_type == edge_type {
                    Some(edge.clone())
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Get the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }
    
    /// Get the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
    
    // ==================== Advanced Graph Traversal Methods ====================
    
    /// Find shortest path between two nodes using Dijkstra's algorithm.
    pub fn shortest_path(&self, from: &str, to: &str) -> Result<Option<GraphPath>> {
        if let (Some(&from_idx), Some(&to_idx)) = (
            self.node_indices.get(from),
            self.node_indices.get(to)
        ) {
            // Use Dijkstra's algorithm with edge weights
            let path = petgraph::algo::dijkstra(
                &self.graph,
                from_idx,
                Some(to_idx),
                |edge| edge.weight().weight as u32,
            );
            
            if let Some(&cost) = path.get(&to_idx) {
                if cost < u32::MAX {
                    // Reconstruct the path
                    return self.reconstruct_shortest_path(from_idx, to_idx);
                }
            }
            
            Ok(None)
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(
                if self.node_indices.get(from).is_none() { from.to_string() } else { to.to_string() }
            ).to_string()))
        }
    }
    
    /// Reconstruct shortest path using BFS with edge weights.
    fn reconstruct_shortest_path(&self, from: NodeIndex, to: NodeIndex) -> Result<Option<GraphPath>> {
        // Simple BFS-based approach for path reconstruction
        let mut visited = HashMap::new();
        let mut queue: VecDeque<(NodeIndex, Vec<NodeIndex>, Vec<EdgeIndex>)> = VecDeque::new();
        
        queue.push_back((from, vec![from], vec![]));
        visited.insert(from, 0.0f32);
        
        while let Some((current, nodes, edges)) = queue.pop_front() {
            if current == to {
                // Build path
                let path_nodes: Vec<GraphNode> = nodes
                    .iter()
                    .filter_map(|&idx| self.graph.node_weight(idx).cloned())
                    .collect();
                
                let path_edges: Vec<GraphEdge> = edges
                    .iter()
                    .filter_map(|&idx| self.graph.edge_weight(idx).cloned())
                    .collect();
                
                let total_weight = path_edges.iter().map(|e| e.weight).sum();
                
                return Ok(Some(GraphPath {
                    nodes: path_nodes,
                    edges: path_edges,
                    total_weight,
                }));
            }
            
            for edge_ref in self.graph.edges(current) {
                let neighbor = edge_ref.target();
                let edge_idx = edge_ref.id();
                
                if let Some(edge) = self.graph.edge_weight(edge_idx) {
                    let new_cost = visited.get(&current).unwrap_or(&0.0f32) + edge.weight;
                    
                    if let Some(&old_cost) = visited.get(&neighbor) {
                        if new_cost >= old_cost {
                            continue;
                        }
                    }
                    
                    visited.insert(neighbor, new_cost);
                    
                    let mut new_nodes = nodes.clone();
                    new_nodes.push(neighbor);
                    
                    let mut new_edges = edges.clone();
                    new_edges.push(edge_idx);
                    
                    queue.push_back((neighbor, new_nodes, new_edges));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Calculate node centrality (degree-based).
    pub fn node_centrality(&self, node_id: &str) -> Result<f32> {
        if let Some(&idx) = self.node_indices.get(node_id) {
            let in_degree = self.graph.edges_directed(idx, petgraph::Direction::Incoming).count() as f32;
            let out_degree = self.graph.edges(idx).count() as f32;
            Ok((in_degree + out_degree) / 2.0)
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(node_id.to_string()).to_string()))
        }
    }
    
    /// Get neighbors of a node.
    pub fn get_neighbors(&self, node_id: &str) -> Result<Vec<(GraphNode, String)>> {
        if let Some(&idx) = self.node_indices.get(node_id) {
            let mut neighbors = Vec::new();
            
            for edge_ref in self.graph.edges(idx) {
                let neighbor_idx = edge_ref.target();
                let edge_idx = edge_ref.id();
                
                if let (Some(node), Some(edge)) = (
                    self.graph.node_weight(neighbor_idx),
                    self.graph.edge_weight(edge_idx)
                ) {
                    neighbors.push((node.clone(), edge.edge_type.clone()));
                }
            }
            
            Ok(neighbors)
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(node_id.to_string()).to_string()))
        }
    }
    
    /// Recommend nodes based on graph traversal.
    pub fn recommend_nodes(&self, start: &str, limit: usize) -> Result<Vec<(GraphNode, f32)>> {
        if let Some(&start_idx) = self.node_indices.get(start) {
            let mut scores: HashMap<NodeIndex, f32> = HashMap::new();
            
            // Simple recommendation: traverse 2 hops and score by edge weights
            for edge_ref in self.graph.edges(start_idx) {
                let neighbor_idx = edge_ref.target();
                if let Some(edge) = self.graph.edge_weight(edge_ref.id()) {
                    *scores.entry(neighbor_idx).or_insert(0.0) += edge.weight;
                    
                    // Second hop
                    for edge_ref2 in self.graph.edges(neighbor_idx) {
                        let neighbor2_idx = edge_ref2.target();
                        if neighbor2_idx != start_idx {
                            if let Some(edge2) = self.graph.edge_weight(edge_ref2.id()) {
                                *scores.entry(neighbor2_idx).or_insert(0.0) += edge.weight * edge2.weight * 0.5;
                            }
                        }
                    }
                }
            }
            
            // Sort by score and return top results
            let mut results: Vec<(GraphNode, f32)> = scores
                .into_iter()
                .filter_map(|(idx, score)| {
                    self.graph.node_weight(idx)
                        .map(|node| (node.clone(), score))
                })
                .collect();
            
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            results.truncate(limit);
            
            Ok(results)
        } else {
            Err(GraphyneError::Graph(GraphError::NodeNotFound(start.to_string()).to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_store() -> (GraphStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = sled::open(dir.path()).unwrap();
        let store = GraphStore::new(&db).unwrap();
        (store, dir)
    }
    
    #[test]
    fn test_add_and_get_node() {
        let (mut store, _dir) = create_test_store();
        
        store.add_node("n1", "person", "Alice", serde_json::json!({}), None).unwrap();
        
        let node = store.get_node("n1").unwrap().unwrap();
        assert_eq!(node.node_type, "person");
        assert_eq!(node.label, "Alice");
    }
    
    #[test]
    fn test_add_edge() {
        let (mut store, _dir) = create_test_store();
        
        store.add_node("n1", "person", "Alice", serde_json::json!({}), None).unwrap();
        store.add_node("n2", "person", "Bob", serde_json::json!({}), None).unwrap();
        
        let edge_id = store.add_edge("n1", "n2", "knows", serde_json::json!({}), 1.0).unwrap();
        
        let edge = store.get_edge(&edge_id).unwrap().unwrap();
        assert_eq!(edge.edge_type, "knows");
        assert_eq!(edge.weight, 1.0);
    }
    
    #[test]
    fn test_traverse() {
        let (mut store, _dir) = create_test_store();
        
        store.add_node("n1", "person", "Alice", serde_json::json!({}), None).unwrap();
        store.add_node("n2", "person", "Bob", serde_json::json!({}), None).unwrap();
        store.add_node("n3", "person", "Charlie", serde_json::json!({}), None).unwrap();
        
        store.add_edge("n1", "n2", "knows", serde_json::json!({}), 1.0).unwrap();
        store.add_edge("n2", "n3", "knows", serde_json::json!({}), 1.0).unwrap();
        
        let paths = store.traverse("n1", 2).unwrap();
        assert!(!paths.is_empty());
    }
    
    #[test]
    fn test_find_nodes_by_type() {
        let (mut store, _dir) = create_test_store();
        
        store.add_node("n1", "person", "Alice", serde_json::json!({}), None).unwrap();
        store.add_node("n2", "company", "Acme", serde_json::json!({}), None).unwrap();
        
        let persons = store.find_nodes_by_type("person");
        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].label, "Alice");
    }
    
    #[test]
    fn test_shortest_path() {
        let (mut store, _dir) = create_test_store();
        
        store.add_node("n1", "person", "Alice", serde_json::json!({}), None).unwrap();
        store.add_node("n2", "person", "Bob", serde_json::json!({}), None).unwrap();
        store.add_node("n3", "person", "Charlie", serde_json::json!({}), None).unwrap();
        
        store.add_edge("n1", "n2", "knows", serde_json::json!({}), 1.0).unwrap();
        store.add_edge("n2", "n3", "knows", serde_json::json!({}), 1.0).unwrap();
        
        let path = store.shortest_path("n1", "n3").unwrap();
        assert!(path.is_some());
    }
    
    #[test]
    fn test_node_centrality() {
        let (mut store, _dir) = create_test_store();
        
        store.add_node("n1", "person", "Alice", serde_json::json!({}), None).unwrap();
        store.add_node("n2", "person", "Bob", serde_json::json!({}), None).unwrap();
        store.add_node("n3", "person", "Charlie", serde_json::json!({}), None).unwrap();
        
        store.add_edge("n1", "n2", "knows", serde_json::json!({}), 1.0).unwrap();
        store.add_edge("n1", "n3", "knows", serde_json::json!({}), 1.0).unwrap();
        
        let centrality = store.node_centrality("n1").unwrap();
        assert!(centrality > 0.0);
    }
}
