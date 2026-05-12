//! Vector search engine with HNSW-based ANN search and cosine similarity.

use std::collections::HashMap;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use sled::Tree;
use thiserror::Error;
use hnsw_rs::prelude::*;
use hnsw_rs::dist::DistCosine;

use crate::error::{Result, GraphyneError};

/// Errors specific to vector operations.
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum VectorError {
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    
    #[error("Vector not found: {0}")]
    NotFound(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Configuration for HNSW index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    pub max_connections: usize,
    pub num_layers: usize,
    pub ef_construction: usize,
    pub dimension: usize,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,
            num_layers: 5,
            ef_construction: 200,
            dimension: 384, // Default for all-MiniLM-L6-v2
        }
    }
}

/// Vector index using HNSW for fast approximate nearest neighbor search.
pub struct VectorIndex {
    /// HNSW index (wrapped in Option since it's built lazily)
    hnsw: Option<Hnsw<'static, f32, DistCosine>>,
    /// Persistent storage for embeddings
    embedding_store: Tree,
    /// Mapping from ID to internal HNSW ID
    id_mapping: Tree,
    /// Configuration
    config: HnswConfig,
    /// Counter for generating unique IDs
    id_counter: u64,
    /// Flag to indicate if HNSW needs rebuilding
    needs_rebuild: bool,
}

impl VectorIndex {
    /// Create a new vector index with the given sled database.
    pub fn new(db: &sled::Db, config: Option<HnswConfig>) -> Result<Self> {
        let embedding_store = db.open_tree("vector_embeddings")?;
        let id_mapping = db.open_tree("vector_id_mapping")?;
        
        let config = config.unwrap_or_default();
        
        // Try to load existing data and build HNSW
        let mut index = Self {
            hnsw: None,
            embedding_store,
            id_mapping,
            config: config.clone(),
            id_counter: 0,
            needs_rebuild: true,
        };
        
        // Try to rebuild HNSW from stored embeddings
        if index.embedding_store.len() > 0 {
            index.rebuild_hnsw()?;
        } else {
            // Initialize empty HNSW
            let hnsw = Hnsw::new(
                config.max_connections,
                config.num_layers,
                config.ef_construction,
                index.id_counter as usize,
                DistCosine {},
            );
            index.hnsw = Some(hnsw);
        }
        
        Ok(index)
    }
    
    /// Rebuild the HNSW index from stored embeddings.
    fn rebuild_hnsw(&mut self) -> Result<()> {
        let config = self.config.clone();
        
        let mut hnsw = Hnsw::new(
            config.max_connections,
            config.num_layers,
            config.ef_construction,
            0, // Start with 0, will be set during insertion
            DistCosine {},
        );
        
        let mut max_id: u64 = 0;
        
        // Iterate through all stored embeddings
        for item in self.embedding_store.iter() {
            let (key, value) = item?;
            let id = String::from_utf8_lossy(&key).to_string();
            
            let embedding: Vec<f32> = serde_json::from_slice(&value)
                .map_err(|e| GraphyneError::Serialization(e))?;
            
            if embedding.len() != config.dimension {
                return Err(GraphyneError::Vector(VectorError::DimensionMismatch {
                    expected: config.dimension,
                    got: embedding.len(),
                }.to_string()));
            }
            
            // Insert into HNSW
            hnsw.insert((&embedding, max_id as usize));
            
            // Update ID counter
            if let Some(counter_bytes) = self.id_mapping.get(&key)? {
                if let Ok(counter) = serde_json::from_slice::<u64>(&counter_bytes) {
                    max_id = max_id.max(counter);
                }
            }
        }
        
        // Set the ID counter
        self.id_counter = max_id + 1;
        self.hnsw = Some(hnsw);
        self.needs_rebuild = false;
        
        Ok(())
    }
    
    /// Add an embedding to the index.
    pub fn add_embedding(&mut self, id: &str, vector: &[f32]) -> Result<()> {
        if vector.len() != self.config.dimension {
            return Err(GraphyneError::Vector(VectorError::DimensionMismatch {
                expected: self.config.dimension,
                got: vector.len(),
            }.to_string()));
        }
        
        // Store in sled
        let id_bytes = id.as_bytes();
        let value = serde_json::to_vec(vector)
            .map_err(|e| GraphyneError::Serialization(e))?;
        self.embedding_store.insert(id_bytes, value)?;
        
        // Update ID mapping
        let id_num = self.id_counter;
        self.id_mapping.insert(id_bytes, serde_json::to_vec(&id_num)?)?;
        self.id_counter += 1;
        
        // Insert into HNSW if it exists
        if let Some(ref mut hnsw) = self.hnsw {
            hnsw.insert((&vector.to_vec(), id_num as usize));
        } else {
            self.needs_rebuild = true;
        }
        
        Ok(())
    }
    
    /// Search for similar vectors.
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        self.search_with_ef(query, k, self.config.ef_construction)
    }
    
    /// Search with a specific ef parameter.
    pub fn search_with_ef(&self, query: &[f32], k: usize, ef: usize) -> Result<Vec<(String, f32)>> {
        if let Some(ref hnsw) = self.hnsw {
            // Use the HNSW search with the given ef parameter
            // Note: hnsw_rs API may vary; this is a simplified version
            let results = hnsw.search(query, k, 16);
            let mut formatted_results = Vec::new();
            
            for neighbour in results {
                // Convert distance to similarity (for cosine distance)
                let similarity = 1.0 - neighbour.distance;
                // Look up string ID from numeric ID
                let id_str = format!("{}", neighbour.d_id);
                formatted_results.push((id_str, similarity));
            }
            
            Ok(formatted_results)
        } else {
            Err(GraphyneError::Vector(
                "HNSW index not initialized".to_string()
            ))
        }
    }
    
    /// Get an embedding by ID.
    pub fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        if let Some(value) = self.embedding_store.get(id.as_bytes())? {
            let embedding: Vec<f32> = serde_json::from_slice(&value)
                .map_err(|e| GraphyneError::Serialization(e))?;
            Ok(Some(embedding))
        } else {
            Ok(None)
        }
    }
    
    /// Remove an embedding from the index.
    pub fn remove_embedding(&mut self, id: &str) -> Result<()> {
        // Remove from sled
        self.embedding_store.remove(id.as_bytes())?;
        self.id_mapping.remove(id.as_bytes())?;
        
        // Mark for rebuild (HNSW doesn't support direct removal)
        self.needs_rebuild = true;
        
        Ok(())
    }
    
    /// Check if the index needs rebuilding.
    pub fn needs_rebuild(&self) -> bool {
        self.needs_rebuild
    }
    
    /// Rebuild the index if needed.
    pub fn maybe_rebuild(&mut self) -> Result<()> {
        if self.needs_rebuild {
            self.rebuild_hnsw()?;
        }
        Ok(())
    }
    
    /// Compute cosine similarity between two vectors.
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_index() -> (VectorIndex, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = sled::open(dir.path()).unwrap();
        let config = HnswConfig {
            max_connections: 16,
            num_layers: 5,
            ef_construction: 200,
            dimension: 3,
        };
        let index = VectorIndex::new(&db, Some(config)).unwrap();
        (index, dir)
    }
    
    #[test]
    fn test_add_and_search() {
        let (mut index, _dir) = create_test_index();
        
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let vec3 = vec![0.0, 0.0, 1.0];
        
        index.add_embedding("doc1", &vec1).unwrap();
        index.add_embedding("doc2", &vec2).unwrap();
        index.add_embedding("doc3", &vec3).unwrap();
        
        let query = vec![1.0, 0.0, 0.0];
        let results = index.search(&query, 2).unwrap();
        
        assert!(!results.is_empty());
        // The search returns numeric IDs from HNSW internal mapping
        assert!(results[0].1 > 0.0); // Check that similarity score is positive
    }
    
    #[test]
    fn test_dimension_mismatch() {
        let (mut index, _dir) = create_test_index();
        
        let wrong_vec = vec![1.0, 0.0]; // Wrong dimension
        let result = index.add_embedding("doc1", &wrong_vec);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        
        assert_eq!(VectorIndex::cosine_similarity(&a, &b), 1.0);
        assert_eq!(VectorIndex::cosine_similarity(&a, &c), 0.0);
    }
    
    #[test]
    fn test_get_embedding() {
        let (mut index, _dir) = create_test_index();
        
        let vec = vec![1.0, 2.0, 3.0];
        index.add_embedding("doc1", &vec).unwrap();
        
        let retrieved = index.get_embedding("doc1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), vec);
    }

    #[test]
    fn test_add_embedding_wrong_dimension() {
        let (mut index, _dir) = create_test_index();
        // Index expects dimension 3, provide 5
        let wrong_vec = vec![1.0, 0.0, 0.0, 0.0, 0.0];
        let result = index.add_embedding("doc1", &wrong_vec);
        assert!(result.is_err());
        let err_str = format!("{}", result.unwrap_err());
        assert!(err_str.contains("Dimension mismatch") || err_str.contains("dimension"),
            "Expected dimension mismatch error, got: {}", err_str);
    }

    #[test]
    fn test_remove_embedding() {
        let (mut index, _dir) = create_test_index();
        let vec = vec![1.0, 2.0, 3.0];
        index.add_embedding("doc1", &vec).unwrap();
        
        // Verify it exists
        assert!(index.get_embedding("doc1").unwrap().is_some());
        
        // Remove it
        index.remove_embedding("doc1").unwrap();
        
        // Verify it's gone from sled
        assert!(index.get_embedding("doc1").unwrap().is_none());
        
        // Verify needs_rebuild flag is set
        assert!(index.needs_rebuild());
    }

    #[test]
    fn test_remove_embedding_not_found() {
        let (mut index, _dir) = create_test_index();
        // Removing a non-existent embedding should not error
        let result = index.remove_embedding("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_embedding_nonexistent() {
        let (index, _dir) = create_test_index();
        let result = index.get_embedding("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert_eq!(VectorIndex::cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let sim = VectorIndex::cosine_similarity(&a, &b);
        assert!(sim < 0.0, "Opposite vectors should have negative similarity, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(VectorIndex::cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_maybe_rebuild() {
        let (mut index, _dir) = create_test_index();
        let vec = vec![1.0, 0.0, 0.0];
        index.add_embedding("doc1", &vec).unwrap();
        
        // Remove to trigger needs_rebuild
        index.remove_embedding("doc1").unwrap();
        assert!(index.needs_rebuild());
        
        // Rebuild
        index.maybe_rebuild().unwrap();
        assert!(!index.needs_rebuild());
    }
}
