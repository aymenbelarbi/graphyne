use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Result, GraphyneError};
use crate::lexical::LexicalIndex;
use crate::vector::VectorIndex;
use crate::graph::{GraphStore, GraphNode, GraphEdge};
use crate::graph::rag::GraphRAG;

use super::types::{MemoryEntry, MemoryType, MemorySpace, ScoringConfig};
use super::retention::RetentionPolicy;
use super::scoring::MemoryScorer;
use super::MemoryQuery;

/// Main memory store that integrates with lexical, vector, and graph stores.
pub struct MemoryStore {
    db: sled::Db,
    lexical_index: LexicalIndex,
    vector_index: VectorIndex,
    graph_store: GraphStore,
    graph_rag: Option<GraphRAG>,
    spaces: HashMap<String, MemorySpace>,
    default_space: String,
}

impl MemoryStore {
    /// Create a new MemoryStore with the given sled database.
    pub fn new(db: sled::Db) -> Result<Self> {
        let lexical_index = LexicalIndex::new(&db)?;
        let vector_index = VectorIndex::new(&db, None)?;
        let graph_store = GraphStore::new(&db)?;

        let mut spaces = HashMap::new();
        let default_space = "default".to_string();
        spaces.insert(default_space.clone(), MemorySpace::new(default_space.clone()));

        Ok(Self {
            db,
            lexical_index,
            vector_index,
            graph_store,
            graph_rag: None,
            spaces,
            default_space,
        })
    }

    /// Initialize GraphRAG for enhanced memory recall.
    pub fn init_graph_rag(&mut self) -> Result<()> {
        // Reconstruct the components since they don't implement Clone
        let graph_store = GraphStore::new(&self.db)?;
        let vector_index = VectorIndex::new(&self.db, None)?;
        let lexical_index = LexicalIndex::new(&self.db)?;
        
        let graph_rag = GraphRAG::new(
            graph_store,
            vector_index,
            lexical_index,
            None,
        );
        self.graph_rag = Some(graph_rag);
        Ok(())
    }

    /// Create a new memory space.
    pub fn create_space(&mut self, name: String) -> Result<()> {
        if self.spaces.contains_key(&name) {
            return Err(GraphyneError::InvalidInput(
                format!("Memory space '{}' already exists", name)
            ));
        }
        self.spaces.insert(name.clone(), MemorySpace::new(name));
        Ok(())
    }

    /// Get a reference to a memory space.
    pub fn get_space(&self, name: &str) -> Option<&MemorySpace> {
        self.spaces.get(name)
    }

    /// Get a mutable reference to a memory space.
    pub fn get_space_mut(&mut self, name: &str) -> Option<&mut MemorySpace> {
        self.spaces.get_mut(name)
    }

    /// List all memory spaces.
    pub fn list_spaces(&self) -> Vec<&MemorySpace> {
        self.spaces.values().collect()
    }

    /// Store a memory entry.
    pub fn store_memory(&mut self, mut entry: MemoryEntry, space: Option<&str>) -> Result<String> {
        let space_name = space.unwrap_or(&self.default_space);
        let space = self.spaces.get(space_name)
            .ok_or_else(|| GraphyneError::InvalidInput(
                format!("Memory space '{}' not found", space_name)
            ))?;

        // Store in sled for persistence
        let key = format!("memory:{}", entry.id);
        let value = serde_json::to_vec(&entry)
            .map_err(|e| GraphyneError::Serialization(e))?;
        self.db.insert(key.as_bytes(), value)?;

        // Index in lexical store (for text search)
        // Use space_name as collection and memory type as bucket
        let bucket = format!("{:?}", entry.memory_type);
        self.lexical_index.push_text(space_name, &bucket, &entry.id, &entry.content)?;

        // Add to vector index (if embedding present)
        if let Some(ref embedding) = entry.embedding {
            self.vector_index.add_embedding(&entry.id, embedding)?;
        }

        // Add to graph store (as a node)
        let node_properties = serde_json::json!({
            "memory_type": format!("{:?}", entry.memory_type),
            "importance": entry.importance,
            "created_at": entry.created_at.to_rfc3339(),
            "last_accessed": entry.last_accessed.to_rfc3339(),
            "access_count": entry.access_count,
            "metadata": entry.metadata,
        });

        // Use first 100 chars of content as label
        let label = if entry.content.len() > 100 {
            format!("{}...", &entry.content[..100])
        } else {
            entry.content.clone()
        };

        self.graph_store.add_node(
            &entry.id,
            "MemoryEntry",
            &label,
            node_properties,
            entry.embedding.clone(),
        )?;

        Ok(entry.id)
    }

    /// Store a memory entry with graph relationships.
    pub fn store_memory_with_graph(
        &mut self,
        mut entry: MemoryEntry,
        relationships: Vec<(String, String, String)> // (from_id, edge_type, to_id)
    ) -> Result<String> {
        // Store the memory first
        let memory_id = self.store_memory(entry, None)?;

        // Create relationships in graph
        for (from_id, edge_type, to_id) in relationships {
            // Ensure both nodes exist
            if self.graph_store.get_node(&from_id)?.is_none() {
                // Create a placeholder node if it doesn't exist
                self.graph_store.add_node(
                    &from_id,
                    "Entity",
                    &from_id,
                    serde_json::json!({}),
                    None,
                )?;
            }

            if self.graph_store.get_node(&to_id)?.is_none() {
                // Create a placeholder node if it doesn't exist
                self.graph_store.add_node(
                    &to_id,
                    "Entity",
                    &to_id,
                    serde_json::json!({}),
                    None,
                )?;
            }

            // Add edge
            self.graph_store.add_edge(
                &from_id,
                &to_id,
                &edge_type,
                serde_json::json!({}),
                1.0,
            )?;
        }

        Ok(memory_id)
    }

    /// Retrieve a memory entry by ID.
    pub fn get_memory(&mut self, id: &str) -> Result<Option<MemoryEntry>> {
        let key = format!("memory:{}", id);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let mut entry: MemoryEntry = serde_json::from_slice(&data)
                .map_err(|e| GraphyneError::Serialization(e))?;

            // Update access information
            entry.access();

            // Update in storage
            let value = serde_json::to_vec(&entry)
                .map_err(|e| GraphyneError::Serialization(e))?;
            self.db.insert(key.as_bytes(), value)?;

            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Update an existing memory entry.
    pub fn update_memory(&mut self, id: &str, updates: MemoryUpdate) -> Result<()> {
        let key = format!("memory:{}", id);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let mut entry: MemoryEntry = serde_json::from_slice(&data)
                .map_err(|e| GraphyneError::Serialization(e))?;

            // Apply updates
            if let Some(content) = updates.content {
                entry.content = content;
                // Re-index in lexical store
                let bucket = format!("{:?}", entry.memory_type);
                let space_name = self.default_space.clone();
                self.lexical_index.push_text(&space_name, &bucket, &entry.id, &entry.content)?;
            }
            if let Some(importance) = updates.importance {
                entry.importance = importance.clamp(0.0, 1.0);
            }
            if let Some(metadata) = updates.metadata {
                entry.metadata = metadata;
            }
            if let Some(embedding) = updates.embedding {
                entry.embedding = Some(embedding.clone());
                // Update vector index
                self.vector_index.add_embedding(&entry.id, &embedding)?;
                // Update graph node with embedding
                if let Some(mut node) = self.graph_store.get_node(&entry.id)? {
                    node.embedding = Some(embedding);
                    // Re-add node with updated embedding
                    let props = serde_json::json!({
                        "memory_type": format!("{:?}", entry.memory_type),
                        "importance": entry.importance,
                        "metadata": node.properties,
                    });
                    self.graph_store.add_node(
                        &entry.id,
                        "MemoryEntry",
                        &node.label,
                        props,
                        node.embedding,
                    )?;
                }
            }

            // Save updated entry
            let value = serde_json::to_vec(&entry)
                .map_err(|e| GraphyneError::Serialization(e))?;
            self.db.insert(key.as_bytes(), value)?;

            Ok(())
        } else {
            Err(GraphyneError::NotFound(format!("Memory entry '{}' not found", id)))
        }
    }

    /// Delete a memory entry.
    pub fn delete_memory(&mut self, id: &str) -> Result<()> {
        let key = format!("memory:{}", id);
        if self.db.remove(key.as_bytes())?.is_some() {
            // Remove from graph store
            self.graph_store.remove_node(id)?;
            // Note: Lexical and vector stores don't have easy removal,
            // but the entry won't be found in future queries
            Ok(())
        } else {
            Err(GraphyneError::NotFound(format!("Memory entry '{}' not found", id)))
        }
    }

    /// Recall memories based on a query.
    pub fn recall(&self, query: &MemoryQuery) -> Result<Vec<(MemoryEntry, f32)>> {
        let mut results: HashMap<String, (MemoryEntry, f32)> = HashMap::new();

        // Get the space for scoring config
        let _space_name = query.include_metadata; // This is a hack, we need to pass space name differently
        // For now, use default space
        let space = self.spaces.get(&self.default_space)
            .ok_or_else(|| GraphyneError::InvalidInput("Default space not found".to_string()))?;

        // Lexical search
        if let Some(ref query_text) = query.query_text {
            let search_results = self.lexical_index.search(&self.default_space, "Working", query_text, query.limit * 2)?;
            for (id, score) in search_results {
                if let Some(entry) = self.get_memory_by_id(&id)? {
                    let normalized_score = 1.0 / (1.0 + score); // Convert BM25 to 0-1 range
                    results.insert(id, (entry, normalized_score));
                }
            }
        }

        // Vector search
        if let Some(ref embedding) = query.embedding {
            let search_results = self.vector_index.search(embedding, query.limit * 2)?;
            for (id, distance) in search_results {
                if let Some(entry) = self.get_memory_by_id(&id)? {
                    // Convert cosine distance to similarity (0-1 range)
                    let similarity = 1.0 - distance.min(1.0).max(0.0);
                    let entry_id = entry.id.clone();
                    match results.get_mut(&entry_id) {
                        Some((_, existing_score)) => {
                            // Combine scores (take max)
                            *existing_score = existing_score.max(similarity);
                        }
                        None => {
                            results.insert(entry_id, (entry, similarity));
                        }
                    }
                }
            }
        }

        // Apply filters
        let mut filtered_results: Vec<(MemoryEntry, f32)> = results
            .into_values()
            .filter(|(entry, _)| {
                // Filter by memory type
                if let Some(ref types) = query.memory_types {
                    if !types.contains(&entry.memory_type) {
                        return false;
                    }
                }
                // Filter by time range
                if let Some((start, end)) = query.time_range {
                    if entry.created_at < start || entry.created_at > end {
                        return false;
                    }
                }
                // Filter by minimum importance
                if let Some(min_imp) = query.min_importance {
                    if entry.importance < min_imp {
                        return false;
                    }
                }
                true
            })
            .collect();

        // Apply memory-specific scoring
        let scorer = MemoryScorer::from_config(&space.scoring_config);
        for (entry, relevance) in &mut filtered_results {
            *relevance = scorer.score(entry, *relevance);
        }

        // Sort by score (descending) and limit
        filtered_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        filtered_results.truncate(query.limit);

        Ok(filtered_results)
    }

    /// Recall memories using GraphRAG for enhanced context.
    pub fn recall_with_graph(&self, query: &str, max_hops: Option<usize>, limit: Option<usize>) -> Result<crate::graph::rag::GraphRAGResult> {
        if let Some(ref graph_rag) = self.graph_rag {
            graph_rag.query(query, max_hops, limit)
        } else {
            Err(GraphyneError::InvalidInput(
                "GraphRAG not initialized. Call init_graph_rag() first.".to_string()
            ))
        }
    }

    /// Get memory by ID (helper that doesn't update access).
    fn get_memory_by_id(&self, id: &str) -> Result<Option<MemoryEntry>> {
        let key = format!("memory:{}", id);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let entry: MemoryEntry = serde_json::from_slice(&data)
                .map_err(|e| GraphyneError::Serialization(e))?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// List all memories in a space.
    pub fn list_memories(&self, space: Option<&str>, limit: usize) -> Result<Vec<MemoryEntry>> {
        let _space_name = space.unwrap_or(&self.default_space);
        let prefix = format!("memory:");
        let mut results = Vec::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_key, value) = item?;
            let entry: MemoryEntry = serde_json::from_slice(&value)
                .map_err(|e| GraphyneError::Serialization(e))?;
            results.push(entry);
            if results.len() >= limit {
                break;
            }
        }

        Ok(results)
    }

    /// Apply retention policy to a space.
    pub fn apply_retention(&mut self, space_name: &str) -> Result<usize> {
        let space = self.spaces.get(space_name)
            .ok_or_else(|| GraphyneError::InvalidInput(
                format!("Memory space '{}' not found", space_name)
            ))?;

        let policy = space.retention_policy.clone();
        let prefix = format!("memory:");
        let mut entries = Vec::new();

        // Collect all entries
        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, value) = item?;
            let entry: MemoryEntry = serde_json::from_slice(&value)
                .map_err(|e| GraphyneError::Serialization(e))?;
            entries.push((key, entry));
        }

        // Apply retention policy
        let mut entries_to_prune: Vec<MemoryEntry> = entries.iter().map(|(_, e)| e.clone()).collect();
        policy.prune(&mut entries_to_prune);

        // Delete entries that didn't make it
        let retained_ids: std::collections::HashSet<String> = entries_to_prune.iter().map(|e| e.id.clone()).collect();
        let mut deleted_count = 0;
        for (key, entry) in entries {
            if !retained_ids.contains(&entry.id) {
                self.db.remove(key)?;
                self.graph_store.remove_node(&entry.id)?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }
}

/// Updates to apply to a memory entry.
#[derive(Debug, Default)]
pub struct MemoryUpdate {
    pub content: Option<String>,
    pub importance: Option<f32>,
    pub metadata: Option<serde_json::Value>,
    pub embedding: Option<Vec<f32>>,
}

#[cfg(test)]
mod store_integration_tests {
    use super::*;
    use tempfile::TempDir;
    use crate::memory::types::{MemoryEntry, MemoryType};

    fn create_test_store() -> (MemoryStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = sled::open(dir.path()).unwrap();
        let store = MemoryStore::new(db).unwrap();
        (store, dir)
    }

    fn make_entry(content: &str) -> MemoryEntry {
        MemoryEntry::new(MemoryType::Semantic, content.to_string(), 0.5)
    }

    #[test]
    fn test_store_memory_nonexistent_space() {
        let (mut store, _dir) = create_test_store();
        let entry = make_entry("test content");
        let result = store.store_memory(entry, Some("nonexistent"));
        assert!(result.is_err());
        let err_str = format!("{}", result.unwrap_err());
        assert!(err_str.contains("not found"), "Expected 'not found' error, got: {}", err_str);
    }

    #[test]
    fn test_get_memory_nonexistent() {
        let (mut store, _dir) = create_test_store();
        let result = store.get_memory("nonexistent-id").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_memory_not_found() {
        let (mut store, _dir) = create_test_store();
        let updates = MemoryUpdate {
            content: Some("new content".to_string()),
            ..Default::default()
        };
        let result = store.update_memory("nonexistent-id", updates);
        assert!(result.is_err());
        let err_str = format!("{}", result.unwrap_err());
        assert!(err_str.contains("not found"), "Expected 'not found' error, got: {}", err_str);
    }

    #[test]
    fn test_delete_memory_not_found() {
        let (mut store, _dir) = create_test_store();
        let result = store.delete_memory("nonexistent-id");
        assert!(result.is_err());
        let err_str = format!("{}", result.unwrap_err());
        assert!(err_str.contains("not found"), "Expected 'not found' error, got: {}", err_str);
    }

    #[test]
    fn test_recall_empty_query_no_memories() {
        let (mut store, _dir) = create_test_store();
        // First store a memory so the lexical index has data
        let entry = make_entry("some stored content");
        store.store_memory(entry, None).unwrap();
        
        let query = MemoryQuery {
            query_text: Some("".to_string()),
            limit: 10,
            ..Default::default()
        };
        // Empty query text tokenizes to nothing, so lexical search returns empty
        let results = store.recall(&query).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_recall_no_matching_documents() {
        let (mut store, _dir) = create_test_store();
        // First store a memory so the lexical index is populated
        let entry = make_entry("some stored content");
        store.store_memory(entry, None).unwrap();
        
        let query = MemoryQuery {
            query_text: Some("zzzznonexistentxxxx".to_string()),
            limit: 10,
            ..Default::default()
        };
        // Query with no matching documents should return empty results
        let results = store.recall(&query).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_list_memories_empty_store() {
        let (store, _dir) = create_test_store();
        let results = store.list_memories(None, 100).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_apply_retention_empty_store() {
        let (mut store, _dir) = create_test_store();
        let deleted = store.apply_retention("default").unwrap();
        assert_eq!(deleted, 0);
    }

    #[test]
    fn test_store_memory_with_graph_creates_edges() {
        let (mut store, _dir) = create_test_store();
        let entry = make_entry("test memory with relationships");
        let relationships = vec![
            ("entity1".to_string(), "relates_to".to_string(), "entity2".to_string()),
            ("entity2".to_string(), "connected_to".to_string(), "entity3".to_string()),
        ];
        let id = store.store_memory_with_graph(entry, relationships).unwrap();

        // Verify the memory was stored
        let retrieved = store.get_memory(&id).unwrap();
        assert!(retrieved.is_some());

        // Verify graph nodes were created for the entities
        let node1 = store.graph_store.get_node("entity1").unwrap();
        assert!(node1.is_some());
        let node2 = store.graph_store.get_node("entity2").unwrap();
        assert!(node2.is_some());
        let node3 = store.graph_store.get_node("entity3").unwrap();
        assert!(node3.is_some());

        // Verify edges were created
        let edges_from_1 = store.graph_store.get_edges_from("entity1").unwrap();
        assert_eq!(edges_from_1.len(), 1);
        assert_eq!(edges_from_1[0].edge_type, "relates_to");
        assert_eq!(edges_from_1[0].to, "entity2");

        let edges_from_2 = store.graph_store.get_edges_from("entity2").unwrap();
        assert_eq!(edges_from_2.len(), 1);
        assert_eq!(edges_from_2[0].edge_type, "connected_to");
        assert_eq!(edges_from_2[0].to, "entity3");
    }
}
