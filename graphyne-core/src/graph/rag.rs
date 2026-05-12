//! GraphRAG engine for enhanced agent reasoning.
//!
//! This module provides GraphRAG capabilities that combine graph traversal
//! with vector similarity to generate rich context for LLM consumption.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::error::{Result, GraphyneError};
use crate::graph::{GraphStore, GraphNode, GraphEdge, GraphPath};
use crate::lexical::LexicalIndex;
use crate::vector::VectorIndex;

/// Result from a GraphRAG query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRAGResult {
    /// Text context formatted for LLM consumption
    pub context: String,
    /// Nodes in the retrieved subgraph
    pub nodes: Vec<GraphNode>,
    /// Edges in the retrieved subgraph
    pub edges: Vec<GraphEdge>,
    /// Confidence score for the result (0.0 to 1.0)
    pub confidence: f32,
    /// Explanation of how the context was gathered
    pub explanation: String,
}

/// Configuration for GraphRAG queries.
#[derive(Debug, Clone)]
pub struct GraphRAGConfig {
    /// Maximum number of hops to traverse from seed nodes
    pub max_hops: usize,
    /// Maximum number of nodes to return
    pub limit: usize,
    /// Weight for lexical search score (0.0 to 1.0)
    pub lexical_weight: f32,
    /// Weight for vector similarity score (0.0 to 1.0)
    pub vector_weight: f32,
    /// Weight for graph centrality score (0.0 to 1.0)
    pub centrality_weight: f32,
}

impl Default for GraphRAGConfig {
    fn default() -> Self {
        Self {
            max_hops: 2,
            limit: 10,
            lexical_weight: 0.4,
            vector_weight: 0.4,
            centrality_weight: 0.2,
        }
    }
}

/// GraphRAG engine that combines graph traversal with vector similarity.
pub struct GraphRAG {
    graph_store: GraphStore,
    vector_index: VectorIndex,
    lexical_index: LexicalIndex,
    config: GraphRAGConfig,
}

impl GraphRAG {
    /// Create a new GraphRAG engine.
    pub fn new(
        graph_store: GraphStore,
        vector_index: VectorIndex,
        lexical_index: LexicalIndex,
        config: Option<GraphRAGConfig>,
    ) -> Self {
        Self {
            graph_store,
            vector_index,
            lexical_index,
            config: config.unwrap_or_default(),
        }
    }
    
    /// Query the knowledge graph to retrieve relevant context.
    ///
    /// This method:
    /// 1. Uses lexical search to find seed nodes
    /// 2. Expands via graph traversal (multi-hop)
    /// 3. Ranks nodes by relevance (graph centrality + vector similarity)
    /// 4. Returns subgraph with context for LLM
    pub fn query(&self, query: &str, max_hops: Option<usize>, limit: Option<usize>) -> Result<GraphRAGResult> {
        let max_hops = max_hops.unwrap_or(self.config.max_hops);
        let limit = limit.unwrap_or(self.config.limit);
        
        // Step 1: Find seed nodes using lexical search
        let seed_nodes = self.find_seed_nodes(query)?;
        
        if seed_nodes.is_empty() {
            return Ok(GraphRAGResult {
                context: "No relevant information found in the knowledge graph.".to_string(),
                nodes: vec![],
                edges: vec![],
                confidence: 0.0,
                explanation: "No seed nodes found for the query.".to_string(),
            });
        }
        
        // Step 2: Expand via graph traversal (multi-hop)
        let mut all_nodes: HashMap<String, GraphNode> = HashMap::new();
        let mut all_edges: HashMap<String, GraphEdge> = HashMap::new();
        let mut node_scores: HashMap<String, f32> = HashMap::new();
        
        for (seed_id, seed_score) in &seed_nodes {
            // Add seed node
            if let Some(node) = self.graph_store.get_node(seed_id)? {
                all_nodes.insert(seed_id.clone(), node);
                node_scores.insert(seed_id.clone(), *seed_score);
            }
            
            // Traverse from seed node
            let paths = self.graph_store.traverse(seed_id, max_hops)?;
            
            for path in paths {
                // Add nodes from path
                for node in &path.nodes {
                    if !all_nodes.contains_key(&node.id) {
                        all_nodes.insert(node.id.clone(), node.clone());
                        
                        // Calculate initial score based on path
                        let path_score = seed_score * (1.0 / (path.total_weight + 1.0));
                        node_scores.insert(node.id.clone(), path_score);
                    }
                }
                
                // Add edges from path
                for edge in &path.edges {
                    if !all_edges.contains_key(&edge.id) {
                        all_edges.insert(edge.id.clone(), edge.clone());
                    }
                }
            }
        }
        
        // Step 3: Rank nodes by relevance
        self.rank_nodes(&mut node_scores, &all_nodes)?;
        
        // Sort nodes by score and limit
        let mut sorted_nodes: Vec<(String, f32)> = node_scores.into_iter().collect();
        sorted_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted_nodes.truncate(limit);
        
        // Collect final nodes and edges
        let final_nodes: Vec<GraphNode> = sorted_nodes
            .iter()
            .filter_map(|(id, _)| all_nodes.get(id).cloned())
            .collect();
        
        // Get edges connecting the final nodes
        let final_node_ids: HashSet<String> = final_nodes.iter().map(|n| n.id.clone()).collect();
        let final_edges: Vec<GraphEdge> = all_edges
            .values()
            .filter(|e| final_node_ids.contains(&e.from) && final_node_ids.contains(&e.to))
            .cloned()
            .collect();
        
        // Step 4: Generate context for LLM
        let context = self.get_subgraph_context(&final_nodes, &final_edges, query);
        let confidence = self.calculate_confidence(&final_nodes, &final_edges);
        let explanation = self.generate_explanation(&seed_nodes, &final_nodes);
        
        Ok(GraphRAGResult {
            context,
            nodes: final_nodes,
            edges: final_edges,
            confidence,
            explanation,
        })
    }
    
    /// Find seed nodes using lexical search.
    fn find_seed_nodes(&self, query: &str) -> Result<HashMap<String, f32>> {
        let mut seed_nodes: HashMap<String, f32> = HashMap::new();
        
        // Use lexical search to find relevant nodes
        // We search in a default collection and bucket
        let search_results = self.lexical_index.search("default", "GraphNode", query, 20);
        
        for result in search_results {
            // The result id should correspond to a node id
            if self.graph_store.get_node(&result.0)?.is_some() {
                seed_nodes.insert(result.0, result.1);
            }
        }
        
        // If no results from lexical search, try to find nodes by label match
        if seed_nodes.is_empty() {
            for idx in self.graph_store.graph.node_indices() {
                if let Some(node) = self.graph_store.graph.node_weight(idx) {
                    if node.label.to_lowercase().contains(&query.to_lowercase()) ||
                       node.node_type.to_lowercase().contains(&query.to_lowercase()) {
                        seed_nodes.insert(node.id.clone(), 0.5);
                    }
                }
            }
        }
        
        Ok(seed_nodes)
    }
    
    /// Rank nodes by combining multiple relevance signals.
    fn rank_nodes(&self, node_scores: &mut HashMap<String, f32>, nodes: &HashMap<String, GraphNode>) -> Result<()> {
        for (node_id, score) in node_scores.iter_mut() {
            let mut final_score = *score;
            
            // Add centrality bonus
            if let Ok(centrality) = self.graph_store.node_centrality(node_id) {
                final_score += centrality * self.config.centrality_weight;
            }
            
            // Add vector similarity bonus if node has embedding
            if let Some(node) = nodes.get(node_id) {
                if node.embedding.is_some() {
                    // In a real implementation, we would compute similarity to query embedding
                    // For now, we give a small bonus
                    final_score += 0.1 * self.config.vector_weight;
                }
            }
            
            *score = final_score;
        }
        
        Ok(())
    }
    
    /// Generate text context from subgraph for LLM consumption.
    pub fn get_subgraph_context(&self, nodes: &[GraphNode], edges: &[GraphEdge], query: &str) -> String {
        let mut context = String::new();
        
        context.push_str(&format!("# Knowledge Graph Context for Query: \"{}\"\n\n", query));
        
        // Summarize the subgraph
        context.push_str(&format!("## Summary\n"));
        context.push_str(&format!("This subgraph contains {} nodes and {} edges.\n\n", nodes.len(), edges.len()));
        
        // List nodes by type
        let mut nodes_by_type: HashMap<String, Vec<&GraphNode>> = HashMap::new();
        for node in nodes {
            nodes_by_type.entry(node.node_type.clone()).or_default().push(node);
        }
        
        context.push_str("## Nodes\n\n");
        for (node_type, type_nodes) in nodes_by_type {
            context.push_str(&format!("### {} ({})\n", node_type, type_nodes.len()));
            for node in type_nodes {
                context.push_str(&format!("- **{}** ({})\n", node.label, node.id));
                if let Some(props) = node.properties.as_object() {
                    if !props.is_empty() {
                        context.push_str("  Properties:\n");
                        for (key, value) in props {
                            context.push_str(&format!("  - {}: {}\n", key, value));
                        }
                    }
                }
                context.push('\n');
            }
        }
        
        // List edges
        context.push_str("## Relationships\n\n");
        let mut edges_by_type: HashMap<String, Vec<&GraphEdge>> = HashMap::new();
        for edge in edges {
            edges_by_type.entry(edge.edge_type.clone()).or_default().push(edge);
        }
        
        for (edge_type, type_edges) in edges_by_type {
            context.push_str(&format!("### {} ({})\n", edge_type, type_edges.len()));
            for edge in type_edges {
                context.push_str(&format!("- {} --[{}]--> {}\n", edge.from, edge.edge_type, edge.to));
                if let Some(props) = edge.properties.as_object() {
                    if !props.is_empty() {
                        for (key, value) in props {
                            context.push_str(&format!("  - {}: {}\n", key, value));
                        }
                    }
                }
            }
            context.push('\n');
        }
        
        // Add paths if there are multiple nodes
        if nodes.len() > 1 {
            context.push_str("## Key Paths\n\n");
            // Find some paths between nodes (simplified - just show direct connections)
            let node_ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();
            let mut shown_paths = 0;
            for edge in edges {
                if node_ids.contains(&edge.from) && node_ids.contains(&edge.to) {
                    let from_node = nodes.iter().find(|n| n.id == edge.from);
                    let to_node = nodes.iter().find(|n| n.id == edge.to);
                    if let (Some(from), Some(to)) = (from_node, to_node) {
                        context.push_str(&format!("- {} ({}) --[{}]--> {} ({})\n", 
                            from.label, from.node_type, edge.edge_type, to.label, to.node_type));
                        shown_paths += 1;
                        if shown_paths >= 5 {
                            break;
                        }
                    }
                }
            }
        }
        
        context
    }
    
    /// Calculate confidence score for the result.
    fn calculate_confidence(&self, nodes: &[GraphNode], edges: &[GraphEdge]) -> f32 {
        if nodes.is_empty() {
            return 0.0;
        }
        
        let mut confidence = 0.5; // Base confidence
        
        // More nodes = higher confidence (up to a point)
        confidence += (nodes.len() as f32 * 0.05).min(0.3);
        
        // More edges = higher confidence (graph is well-connected)
        confidence += (edges.len() as f32 * 0.02).min(0.2);
        
        confidence.min(1.0)
    }
    
    /// Generate explanation of how the context was gathered.
    fn generate_explanation(&self, seed_nodes: &HashMap<String, f32>, final_nodes: &[GraphNode]) -> String {
        let mut explanation = String::new();
        
        explanation.push_str(&format!("Found {} seed nodes from lexical search. ", seed_nodes.len()));
        explanation.push_str(&format!("Expanded graph traversal to retrieve {} total nodes. ", final_nodes.len()));
        
        if !seed_nodes.is_empty() {
            explanation.push_str("Seed nodes: ");
            let seed_ids: Vec<String> = seed_nodes.keys().cloned().collect();
            explanation.push_str(&seed_ids.join(", "));
            explanation.push('.');
        }
        
        explanation
    }
    
    /// Get a subgraph around specific nodes.
    pub fn get_subgraph(&self, node_ids: &[String], max_hops: usize) -> Result<(Vec<GraphNode>, Vec<GraphEdge>)> {
        let mut all_nodes: HashMap<String, GraphNode> = HashMap::new();
        let mut all_edges: HashMap<String, GraphEdge> = HashMap::new();
        
        for node_id in node_ids {
            if let Some(node) = self.graph_store.get_node(node_id)? {
                all_nodes.insert(node.id.clone(), node);
                
                // Traverse from this node
                let paths = self.graph_store.traverse(node_id, max_hops)?;
                
                for path in paths {
                    for node in &path.nodes {
                        if !all_nodes.contains_key(&node.id) {
                            all_nodes.insert(node.id.clone(), node.clone());
                        }
                    }
                    
                    for edge in &path.edges {
                        if !all_edges.contains_key(&edge.id) {
                            all_edges.insert(edge.id.clone(), edge.clone());
                        }
                    }
                }
            }
        }
        
        Ok((all_nodes.into_values().collect(), all_edges.into_values().collect()))
    }
    
    /// Expand a node with its neighbors.
    pub fn expand_node(&self, node_id: &str, depth: usize) -> Result<(Vec<GraphNode>, Vec<GraphEdge>)> {
        self.get_subgraph(&[node_id.to_string()], depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use serde_json::json;
    
    fn create_test_graphrag() -> (GraphRAG, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = sled::open(dir.path()).unwrap();
        
        let graph_store = GraphStore::new(&db).unwrap();
        let vector_index = VectorIndex::new(&db, None).unwrap();
        let lexical_index = LexicalIndex::new(&db).unwrap();
        
        let graphrag = GraphRAG::new(graph_store, vector_index, lexical_index, None);
        (graphrag, dir)
    }
    
    #[test]
    fn test_graphrag_query_empty() {
        let (graphrag, _dir) = create_test_graphrag();
        
        let result = graphrag.query("nonexistent", None, None).unwrap();
        assert_eq!(result.confidence, 0.0);
        assert!(result.nodes.is_empty());
    }
}
