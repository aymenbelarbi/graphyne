//! Graph storage with typed property graph support using petgraph.
//!
//! This module provides graph database capabilities with support for typed nodes,
//! typed edges, node/edge properties, and multi-hop traversal.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use petgraph::graph::{Graph, NodeIndex, EdgeIndex};
use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences};
use serde::{Deserialize, Serialize};
use sled::Tree;
use thiserror::Error;

use crate::error::{Result, GraphyneError};

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
    pub properties: serde_json::Value,
}

/// An edge in the property graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub properties: serde_json::Value,
}

/// A path resulting from graph traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPath {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub length: usize,
}

/// Graph store with typed property graph support.
pub struct GraphStore {
    /// In-memory graph for fast traversal
    graph: Graph<GraphNode, GraphEdge>,
    /// Mapping from node ID to petgraph NodeIndex
    node_indices: HashMap<String, NodeIndex>,
    /// Mapping from edge ID to petgraph EdgeIndex
    edge_indices: HashMap<String, EdgeIndex>,
    /// Persistent storage for nodes
    nodes_store: Tree,
    /// Persistent storage for edges
    edges_store: Tree,
    /// Counter for generating unique edge IDs
    edge_counter: u64,
}

impl GraphStore {
    /// Create a new graph store with the given sled database.
    pub fn new(db: &sled::Db) -> Result<Self> {
        let nodes_store = db.open_tree("graph_nodes")?;
        let edges_store = db.open_tree("graph_edges")?;
        
        let mut store = Self {
            graph: Graph::new(),
            node_indices: HashMap::new(),
            edge_indices: HashMap::new(),
            nodes_store,
            edges_store,
            edge_counter: 0,
        };
        
        // Load existing data from storage
        store.load_from_storage()?;
        
        Ok(store)
    }
    
    /// Load graph data from persistent storage.
    fn load_from_storage(&mut self) -> Result<()> {
        // Load nodes
        for item in self.nodes_store.iter() {
            let (key, value) = item?;
            let id = String::from_utf8_lossy(&key).to_string();
            let node: GraphNode = serde_json::from_slice(&value)
                .map_err(|e| GraphyneError::SerializationError(e.to_string()))?;
            
            let node_idx = self.graph.add_node(node);
            self.node_indices.insert(id, node_idx);
        }
        
        // Load edges
        for item in self.edges_store.iter() {
            let (key, value) = item?;
            let edge_id = String::from_utf8_lossy(&key).to_string();
            let edge: GraphEdge = serde_json::from_slice(&value)
                .map_err(|e| GraphyneError::SerializationError(e.to_string()))?;
            
            // Find source and target nodes
            if let (Some(&from_idx), Some(&to_idx)) = (
                self.node_indices.get(&edge.from),
                self.node_indices.get(&edge.to)
            ) {
                let edge_idx = self.graph.add_edge(from_idx, to_idx, edge);
                self.edge_indices.insert(edge_id, edge_idx);
            }
        }
        
        // Update edge counter
        self.edge_counter = self.edge_indices.len() as u64;
        
        Ok(())
    }
    
    /// Generate a unique edge ID.
    fn generate_edge_id(&mut self) -> String {
        self.edge_counter += 1;
        format!("edge_{}", self.edge_counter)
    }
    
    /// Add a node to the graph.
    pub fn add_node(&mut self, id: &str, node_type: &str, properties: serde_json::Value) -> Result<()> {
        if id.is_empty() || node_type.is_empty() {
            return Err(GraphyneError::GraphError(GraphError::InvalidId(
                "Node ID and type must not be empty".to_string()
            )));
        }
        
        // Check if node already exists
        if self.node_indices.contains_key(id) {
            // Update existing node
            self.remove_node(id)?;
        }
        
        let node = GraphNode {
            id: id.to_string(),
            node_type: node_type.to_string(),
            properties,
        };
        
        // Add to in-memory graph
        let node_idx = self.graph.add_node(node.clone());
        self.node_indices.insert(id.to_string(), node_idx);
        
        // Store in persistent storage
        let json = serde_json::to_vec(&node)
            .map_err(|e| GraphyneError::SerializationError(e.to_string()))?;
        self.nodes_store.insert(id.as_bytes(), json)?;
        
        Ok(())
    }
    
    /// Remove a node from the graph.
    pub fn remove_node(&mut self, id: &str) -> Result<()> {
        if let Some(&node_idx) = self.node_indices.get(id) {
            // Remove all edges connected to this node
            let edges_to_remove: Vec<String> = self.edge_indices
                .iter()
                .filter(|(_, &edge_idx)| {
                    if let Some(edge_ref) = self.graph.edge_endpoints(edge_idx) {
                        edge_ref.0 == node_idx || edge_ref.1 == node_idx
                    } else {
                        false
                    }
                })
                .map(|(id, _)| id.clone())
                .collect();
            
            for edge_id in edges_to_remove {
                self.remove_edge(&edge_id)?;
            }
            
            // Remove node from graph
            self.graph.remove_node(node_idx);
            self.node_indices.remove(id);
            
            // Remove from storage
            self.nodes_store.remove(id.as_bytes())?;
            
            Ok(())
        } else {
            Err(GraphyneError::GraphError(GraphError::NodeNotFound(id.to_string())))
        }
    }
    
    /// Add an edge to the graph.
    pub fn add_edge(&mut self, from: &str, to: &str, edge_type: &str, properties: serde_json::Value) -> Result<String> {
        if from.is_empty() || to.is_empty() || edge_type.is_empty() {
            return Err(GraphyneError::GraphError(GraphError::InvalidId(
                "From, to, and edge type must not be empty".to_string()
            )));
        }
        
        // Check if both nodes exist
        let from_idx = self.node_indices.get(from)
            .ok_or_else(|| GraphyneError::GraphError(GraphError::NodeNotFound(from.to_string())))?;
        let to_idx = self.node_indices.get(to)
            .ok_or_else(|| GraphyneError::GraphError(GraphError::NodeNotFound(to.to_string())))?;
        
        let edge_id = self.generate_edge_id();
        
        let edge = GraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type: edge_type.to_string(),
            properties,
        };
        
        // Add to in-memory graph
        let edge_idx = self.graph.add_edge(*from_idx, *to_idx, edge.clone());
        self.edge_indices.insert(edge_id.clone(), edge_idx);
        
        // Store in persistent storage
        let json = serde_json::to_vec(&edge)
            .map_err(|e| GraphyneError::SerializationError(e.to_string()))?;
        self.edges_store.insert(edge_id.as_bytes(), json)?;
        
        Ok(edge_id)
    }
    
    /// Remove an edge from the graph.
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<()> {
        if let Some(&edge_idx) = self.edge_indices.get(edge_id) {
            self.graph.remove_edge(edge_idx);
            self.edge_indices.remove(edge_id);
            self.edges_store.remove(edge_id.as_bytes())?;
            Ok(())
        } else {
            Err(GraphyneError::GraphError(GraphError::EdgeNotFound(edge_id.to_string())))
        }
    }
    
    /// Get a node by ID.
    pub fn get_node(&self, id: &str) -> Result<Option<GraphNode>> {
        if let Some(&node_idx) = self.node_indices.get(id) {
            if let Some(node) = self.graph.node_weight(node_idx) {
                Ok(Some(node.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// Get an edge by ID.
    pub fn get_edge(&self, edge_id: &str) -> Result<Option<GraphEdge>> {
        if let Some(&edge_idx) = self.edge_indices.get(edge_id) {
            if let Some(edge) = self.graph.edge_weight(edge_idx) {
                Ok(Some(edge.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
    
    /// Get all edges from a node.
    pub fn get_edges_from(&self, node_id: &str) -> Result<Vec<GraphEdge>> {
        if let Some(&node_idx) = self.node_indices.get(node_id) {
            let edges: Vec<GraphEdge> = self.graph
                .edges(node_idx)
                .filter_map(|edge_ref| {
                    self.graph.edge_weight(edge_ref.id()).cloned()
                })
                .collect();
            Ok(edges)
        } else {
            Err(GraphyneError::GraphError(GraphError::NodeNotFound(node_id.to_string())))
        }
    }
    
    /// Traverse the graph starting from a node, up to max_hops.
    pub fn traverse(&self, start: &str, max_hops: usize) -> Result<Vec<GraphPath>> {
        if let Some(&start_idx) = self.node_indices.get(start) {
            let mut paths = Vec::new();
            let mut visited = HashMap::new();
            let mut queue: VecDeque<(NodeIndex, Vec<NodeIndex>, Vec<EdgeIndex>)> = VecDeque::new();
            
            queue.push_back((start_idx, vec![start_idx], vec![]));
            visited.insert(start_idx, 0);
            
            while let Some((current, node_path, edge_path)) = queue.pop_front() {
                let current_hops = *visited.get(&current).unwrap_or(&0);
                
                if current_hops >= max_hops {
                    continue;
                }
                
                // Explore neighbors
                for edge_ref in self.graph.edges(current) {
                    let neighbor = edge_ref.target();
                    let edge_idx = edge_ref.id();
                    
                    let neighbor_hops = current_hops + 1;
                    
                    if let Some(&prev_hops) = visited.get(&neighbor) {
                        if prev_hops <= neighbor_hops {
                            continue;
                        }
                    }
                    
                    visited.insert(neighbor, neighbor_hops);
                    
                    let mut new_node_path = node_path.clone();
                    new_node_path.push(neighbor);
                    
                    let mut new_edge_path = edge_path.clone();
                    new_edge_path.push(edge_idx);
                    
                    // Build path
                    let nodes: Vec<GraphNode> = new_node_path
                        .iter()
                        .filter_map(|&idx| self.graph.node_weight(idx).cloned())
                        .collect();
                    
                    let edges: Vec<GraphEdge> = new_edge_path
                        .iter()
                        .filter_map(|&idx| self.graph.edge_weight(idx).cloned())
                        .collect();
                    
                    paths.push(GraphPath {
                        nodes,
                        edges,
                        length: neighbor_hops,
                    });
                    
                    queue.push_back((neighbor, new_node_path, new_edge_path));
                }
            }
            
            Ok(paths)
        } else {
            Err(GraphyneError::GraphError(GraphError::NodeNotFound(start.to_string())))
        }
    }
    
    /// Find nodes by type.
    pub fn find_nodes_by_type(&self, node_type: &str) -> Vec<GraphNode> {
        self.graph
            .node_references()
            .filter_map(|(_, node)| {
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
                let edge = self.graph.edge_weight(edge_ref.id())?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde_json::json;
    
    fn create_test_store() -> (GraphStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = sled::open(dir.path()).unwrap();
        let store = GraphStore::new(&db).unwrap();
        (store, dir)
    }
    
    #[test]
    fn test_add_and_get_node() {
        let (mut store, _dir) = create_test_store();
        
        let props = json!({"name": "Alice", "age": 30});
        store.add_node("n1", "Person", props).unwrap();
        
        let node = store.get_node("n1").unwrap();
        assert!(node.is_some());
        assert_eq!(node.unwrap().node_type, "Person");
    }
    
    #[test]
    fn test_add_edge() {
        let (mut store, _dir) = create_test_store();
        
        store.add_node("n1", "Person", json!({})).unwrap();
        store.add_node("n2", "Person", json!({})).unwrap();
        
        let edge_id = store.add_edge("n1", "n2", "KNOWS", json!({"since": 2020})).unwrap();
        
        let edge = store.get_edge(&edge_id).unwrap();
        assert!(edge.is_some());
        assert_eq!(edge.unwrap().edge_type, "KNOWS");
    }
    
    #[test]
    fn test_traverse() {
        let (mut store, _dir) = create_test_store();
        
        // Create a simple graph: n1 -> n2 -> n3
        store.add_node("n1", "Person", json!({})).unwrap();
        store.add_node("n2", "Person", json!({})).unwrap();
        store.add_node("n3", "Person", json!({})).unwrap();
        
        store.add_edge("n1", "n2", "KNOWS", json!({})).unwrap();
        store.add_edge("n2", "n3", "KNOWS", json!({})).unwrap();
        
        let paths = store.traverse("n1", 2).unwrap();
        assert!(!paths.is_empty());
    }
    
    #[test]
    fn test_find_nodes_by_type() {
        let (mut store, _dir) = create_test_store();
        
        store.add_node("n1", "Person", json!({})).unwrap();
        store.add_node("n2", "Company", json!({})).unwrap();
        
        let persons = store.find_nodes_by_type("Person");
        assert_eq!(persons.len(), 1);
        assert_eq!(persons[0].id, "n1");
    }
}
