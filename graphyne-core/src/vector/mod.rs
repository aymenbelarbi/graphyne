//! Vector search engine with HNSW-based ANN search and cosine similarity.
//!
//! This module provides vector/embedding search capabilities using HNSW (Hierarchical
//! Navigable Small World) algorithm for fast approximate nearest neighbor search.

use std::sync::Arc;
use std::collections::HashMap;

use hnsw_rs::prelude::*;
use serde::{Deserialize, Serialize};
use sled::Tree;
use thiserror::Error;

use crate::error::{Result, GraphyneError};

/// Errors specific to vector search operations.
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum VectorError {
    #[error("HNSW operation failed: {0}")]
    HnswError(String),
    
    #[error("Embedding dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    
    #[error("Invalid embedding: {0}")]
    InvalidEmbedding(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Configuration for HNSW index.
#[derive(Debug, Clone)]
pub struct HnswConfig {
    /// Maximum number of connections per element per layer (M)
    pub max_connections: usize,
    /// Size of the dynamic candidate list (ef_construction)
    pub ef_construction: usize,
    /// Number of layers in the graph
    pub num_layers: usize,
    /// Dimension of the vectors
    pub dimension: usize,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            max_connections: 16,
            ef_construction: 200,
            num_layers: 16,
            dimension: 768, // Default for many embedding models
        }
    }
}

/// Vector index using HNSW for ANN search.
pub struct VectorIndex {
    /// HNSW index for fast ANN search
    hnsw: Option<Hnsw<'static, f32, DistCosine>>,
    /// Storage for embeddings
    embedding_store: Tree,
    /// Storage for ID to HNSW internal ID mapping
    id_mapping: Tree,
    /// HNSW configuration
    config: HnswConfig,
    /// Counter for HNSW internal IDs
    id_counter: u64,
    /// Whether the HNSW index needs rebuilding
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
        
        // Set ef for search (can be adjusted later)
        hnsw.set_ef(50);
        
        let mut max_id: u64 = 0;
        
        // Iterate through all stored embeddings
        for item in self.embedding_store.iter() {
            let (key, value) = item?;
            let id = String::from_utf8_lossy(&key).to_string();
            
            let embedding: Vec<f32> = serde_json::from_slice(&value)
                .map_err(|e| GraphyneError::SerializationError(e.to_string()))?;
            
            if embedding.len() != config.dimension {
                return Err(GraphyneError::VectorError(VectorError::DimensionMismatch {
                    expected: config.dimension,
                    got: embedding.len(),
                }));
            }
            
            // Insert into HNSW
            hnsw.insert((embedding, id.clone()));
            
            // Update ID counter
            if let Ok(counter_bytes) = self.id_mapping.get(&key)? {
                if let Ok(counter) = serde_json::from_slice::<u64>(&counter_bytes) {
                    max_id = max_id.max(counter);
                }
            }
        }
        
        self.id_counter = max_id + 1;
        self.hnsw = Some(hnsw);
        self.needs_rebuild = false;
        
        Ok(())
    }
    
    /// Add an embedding to the index.
    pub fn add_embedding(&mut self, id: &str, vector: &[f32]) -> Result<()> {
        if id.is_empty() {
            return Err(GraphyneError::VectorError(VectorError::InvalidEmbedding(
                "ID must not be empty".to_string()
            )));
        }
        
        if vector.len() != self.config.dimension {
            return Err(GraphyneError::VectorError(VectorError::DimensionMismatch {
                expected: self.config.dimension,
                got: vector.len(),
            }));
        }
        
        // Validate vector
        for (i, &val) in vector.iter().enumerate() {
            if val.is_nan() || val.is_infinite() {
                return Err(GraphyneError::VectorError(VectorError::InvalidEmbedding(
                    format!("Vector contains NaN or infinite value at index {}", i)
                )));
            }
        }
        
        // Store embedding
        let json = serde_json::to_vec(vector)
            .map_err(|e| GraphyneError::SerializationError(e.to_string()))?;
        self.embedding_store.insert(id.as_bytes(), json)?;
        
        // Store ID mapping
        let id_bytes = serde_json::to_vec(&self.id_counter)
            .map_err(|e| GraphyneError::SerializationError(e.to_string()))?;
        self.id_mapping.insert(id.as_bytes(), id_bytes)?;
        
        // Insert into HNSW
        if let Some(ref mut hnsw) = self.hnsw {
            hnsw.insert((vector.to_vec(), id.to_string()));
        } else {
            self.needs_rebuild = true;
        }
        
        self.id_counter += 1;
        
        Ok(())
    }
    
    /// Search for nearest neighbors using ANN.
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        if query.len() != self.config.dimension {
            return Err(GraphyneError::VectorError(VectorError::DimensionMismatch {
                expected: self.config.dimension,
                got: query.len(),
            }));
        }
        
        if let Some(ref hnsw) = self.hnsw {
            // Perform ANN search
            let results = hnsw.search(query, k);
            
            // Convert results to (id, distance) pairs
            let mut formatted_results: Vec<(String, f32)> = results
                .iter()
                .map(|(distance, data)| {
                    let id = data.clone();
                    (*id, *distance)
                })
                .collect();
            
            // Sort by distance (lower is better for cosine distance)
            formatted_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            
            Ok(formatted_results)
        } else {
            // HNSW not initialized, try to rebuild
            Err(GraphyneError::VectorError(VectorError::HnswError(
                "HNSW index not initialized".to_string()
            )))
        }
    }
    
    /// Search with a specific ef value (controls recall/performance tradeoff).
    pub fn search_with_ef(&self, query: &[f32], k: usize, ef: usize) -> Result<Vec<(String, f32)>> {
        if query.len() != self.config.dimension {
            return Err(GraphyneError::VectorError(VectorError::DimensionMismatch {
                expected: self.config.dimension,
                got: query.len(),
            }));
        }
        
        if let Some(ref hnsw) = self.hnsw {
            // Set ef for this search
            hnsw.set_ef(ef);
            
            // Perform ANN search
            let results = hnsw.search(query, k);
            
            // Convert results
            let formatted_results: Vec<(String, f32)> = results
                .iter()
                .map(|(distance, data)| {
                    (data.clone(), *distance)
                })
                .collect();
            
            Ok(formatted_results)
        } else {
            Err(GraphyneError::VectorError(VectorError::HnswError(
                "HNSW index not initialized".to_string()
            )))
        }
    }
    
    /// Get an embedding by ID.
    pub fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>> {
        if let Some(data) = self.embedding_store.get(id.as_bytes())? {
            let embedding: Vec<f32> = serde_json::from_slice(&data)
                .map_err(|e| GraphyneError::SerializationError(e.to_string()))?;
            Ok(Some(embedding))
        } else {
            Ok(None)
        }
    }
    
    /// Remove an embedding from the index.
    pub fn remove_embedding(&mut self, id: &str) -> Result<()> {
        // Remove from storage
        self.embedding_store.remove(id.as_bytes())?;
        self.id_mapping.remove(id.as_bytes())?;
        
        // Mark for rebuild (HNSW doesn't support direct removal)
        self.needs_rebuild = true;
        
        Ok(())
    }
    
    /// Get the number of embeddings in the index.
    pub fn len(&self) -> usize {
        self.embedding_store.len() as usize
    }
    
    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.embedding_store.is_empty()
    }
    
    /// Rebuild the HNSW index if needed.
    pub fn maybe_rebuild(&mut self) -> Result<()> {
        if self.needs_rebuild {
            self.rebuild_hnsw()?;
        }
        Ok(())
    }
    
    /// Calculate cosine similarity between two vectors.
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        
        for i in 0..a.len() {
            dot_product += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }
        
        let norm_a = norm_a.sqrt();
        let norm_b = norm_b.sqrt();
        
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
            max_connections: 8,
            ef_construction: 50,
            num_layers: 8,
            dimension: 4,
        };
        let index = VectorIndex::new(&db, Some(config)).unwrap();
        (index, dir)
    }
    
    #[test]
    fn test_add_and_search() {
        let (mut index, _dir) = create_test_index();
        
        // Add some embeddings
        index.add_embedding("doc1", &[1.0, 0.0, 0.0, 0.0]).unwrap();
        index.add_embedding("doc2", &[0.0, 1.0, 0.0, 0.0]).unwrap();
        index.add_embedding("doc3", &[0.0, 0.0, 1.0, 0.0]).unwrap();
        
        // Search for similar vector
        let results = index.search(&[1.0, 0.0, 0.0, 0.0], 2).unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "doc1"); // Should be most similar
    }
    
    #[test]
    fn test_dimension_mismatch() {
        let (mut index, _dir) = create_test_index();
        
        let result = index.add_embedding("doc1", &[1.0, 0.0, 0.0]); // Wrong dimension
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
        
        let vec = vec![0.5, 0.5, 0.5, 0.5];
        index.add_embedding("doc1", &vec).unwrap();
        
        let retrieved = index.get_embedding("doc1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), vec);
    }
}
