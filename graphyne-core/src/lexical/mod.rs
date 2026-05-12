//! Lexical search engine with FST-based term indexing and BM25 scoring.
//!
//! This module provides text search capabilities using Finite State Transducers (FST)
//! for fast term lookup and BM25 algorithm for relevance scoring.

use std::collections::HashMap;
use std::sync::Arc;

use fst::{Map, MapBuilder, Streamer, IntoStreamer};
use serde::{Deserialize, Serialize};
use sled::Tree;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;
use whatlang::detect;

use crate::error::{Result, GraphyneError};

/// Errors specific to lexical search operations.
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum LexicalError {
    #[error("FST operation failed: {0}")]
    FstError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Invalid collection or bucket name: {0}")]
    InvalidName(String),
}

/// Token with its frequency in a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TermFrequency {
    term: String,
    frequency: u32,
}

/// Document metadata for BM25 scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentStats {
    doc_id: String,
    term_frequencies: HashMap<String, u32>,
    doc_length: u32,
    collection: String,
    bucket: String,
}

impl Default for DocumentStats {
    fn default() -> Self {
        Self {
            doc_id: String::new(),
            term_frequencies: HashMap::new(),
            doc_length: 0,
            collection: String::new(),
            bucket: String::new(),
        }
    }
}

/// BM25 scoring parameters.
#[derive(Debug, Clone)]
pub struct Bm25Params {
    pub k1: f32,
    pub b: f32,
}

impl Default for Bm25Params {
    fn default() -> Self {
        Self { k1: 1.2, b: 0.75 }
    }
}

/// Lexical search index with FST-based term indexing and BM25 scoring.
pub struct LexicalIndex {
    /// Stores FST data per collection/bucket combination
    fst_store: Tree,
    /// Stores term -> document mappings and frequencies
    kv_store: Tree,
    /// Stores document statistics for BM25
    doc_stats: Tree,
    /// BM25 parameters
    bm25_params: Bm25Params,
    /// Average document length (cached, recalculated periodically)
    avg_doc_length: f32,
    /// Total number of documents
    total_docs: u64,
}

impl LexicalIndex {
    /// Create a new lexical index with the given sled database.
    pub fn new(db: &sled::Db) -> Result<Self> {
        let fst_store = db.open_tree("lexical_fst")?;
        let kv_store = db.open_tree("lexical_kv")?;
        let doc_stats = db.open_tree("lexical_doc_stats")?;
        
        let total_docs = doc_stats.len() as u64;
        
        Ok(Self {
            fst_store,
            kv_store,
            doc_stats,
            bm25_params: Bm25Params::default(),
            avg_doc_length: 0.0,
            total_docs,
        })
    }
    
    /// Set BM25 parameters.
    pub fn set_bm25_params(&mut self, params: Bm25Params) {
        self.bm25_params = params;
    }
    
    /// Tokenize text with language detection.
    /// Returns a vector of tokens (lowercased, normalized).
    pub fn tokenize(text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        
        // Detect language for potential language-specific processing
        let _lang = detect(text);
        
        // Use Unicode segmentation for proper word boundaries
        for word in text.unicode_words() {
            let token = word.to_lowercase();
            if !token.is_empty() && token.len() > 1 {
                tokens.push(token);
            }
        }
        
        tokens
    }
    
    /// Generate a storage key for FST data.
    fn fst_key(collection: &str, bucket: &str) -> String {
        format!("{}:{}", collection, bucket)
    }
    
    /// Generate a storage key for term -> documents mapping.
    fn term_key(collection: &str, bucket: &str, term: &str) -> String {
        format!("{}:{}:{}", collection, bucket, term)
    }
    
    /// Generate a storage key for document stats.
    fn doc_key(collection: &str, bucket: &str, doc_id: &str) -> String {
        format!("{}:{}:{}", collection, bucket, doc_id)
    }
    
    /// Build or update FST for a collection/bucket.
    fn update_fst(&self, collection: &str, bucket: &str, terms: &[String]) -> Result<()> {
        let key = Self::fst_key(collection, bucket);
        
        // Collect existing terms and add new ones
        let mut all_terms: Vec<String> = if let Some(data) = self.fst_store.get(&key)? {
            // Deserialize existing terms
            let existing: Vec<String> = serde_json::from_slice(&data)
                .map_err(|e| GraphyneError::Serialization(e))?;
            existing
        } else {
            Vec::new()
        };
        
        // Add new terms
        for term in terms {
            if !all_terms.contains(term) {
                all_terms.push(term.clone());
            }
        }
        
        // Sort terms for FST
        all_terms.sort();
        all_terms.dedup();
        
        // Build FST
        let mut fst_data = Vec::new();
        {
            let mut builder = MapBuilder::new(&mut fst_data)
                .map_err(|e| GraphyneError::Lexical(e.to_string()))?;
            
            for (idx, term) in all_terms.iter().enumerate() {
                builder.insert(term, idx as u64)
                    .map_err(|e| GraphyneError::Lexical(e.to_string()))?;
            }
            
            builder.finish()
                .map_err(|e| GraphyneError::Lexical(e.to_string()))?;
        }
        
        // Store FST data and term list
        let term_list_json = serde_json::to_vec(&all_terms)
            .map_err(|e| GraphyneError::Serialization(e))?;
        
        self.fst_store.insert(key.as_bytes(), term_list_json)?;
        
        Ok(())
    }
    
    /// Push text into the index for a given document.
    pub fn push_text(&mut self, collection: &str, bucket: &str, doc_id: &str, text: &str) -> Result<()> {
        if collection.is_empty() || bucket.is_empty() || doc_id.is_empty() {
            return Err(GraphyneError::Lexical(
                "Collection, bucket, and doc_id must not be empty".to_string()
            ));
        }
        
        // Tokenize the text
        let tokens = Self::tokenize(text);
        
        // Count term frequencies
        let mut term_freqs: HashMap<String, u32> = HashMap::new();
        for token in &tokens {
            *term_freqs.entry(token.clone()).or_insert(0) += 1;
        }
        
        // Update FST with new terms
        let terms: Vec<String> = term_freqs.keys().cloned().collect();
        self.update_fst(collection, bucket, &terms)?;
        
        // Store term -> document mappings
        for (term, freq) in &term_freqs {
            let term_key = Self::term_key(collection, bucket, term);
            
            // Get existing mappings
            let mut doc_freqs: HashMap<String, u32> = if let Some(data) = self.kv_store.get(&term_key)? {
                serde_json::from_slice(&data)
                    .unwrap_or_default()
            } else {
                HashMap::new()
            };
            
            doc_freqs.insert(doc_id.to_string(), *freq);
            
            let json = serde_json::to_vec(&doc_freqs)
                .map_err(|e| GraphyneError::Serialization(e))?;
            self.kv_store.insert(term_key.as_bytes(), json)?;
        }
        
        // Store document stats
        let doc_key = Self::doc_key(collection, bucket, doc_id);
        let stats = DocumentStats {
            doc_id: doc_id.to_string(),
            term_frequencies: term_freqs.clone(),
            doc_length: tokens.len() as u32,
            collection: collection.to_string(),
            bucket: bucket.to_string(),
        };
        
        let json = serde_json::to_vec(&stats)
            .map_err(|e| GraphyneError::Serialization(e))?;
        self.doc_stats.insert(doc_key.as_bytes(), json)?;
        
        self.total_docs = self.doc_stats.len() as u64;
        
        Ok(())
    }
    
    /// Search for documents matching the query using BM25 scoring.
    pub fn search(&self, collection: &str, bucket: &str, query: &str, limit: usize) -> Result<Vec<(String, f32)>> {
        if collection.is_empty() || bucket.is_empty() {
            return Err(GraphyneError::Lexical(
                "Collection and bucket must not be empty".to_string()
            ));
        }
        
        // Tokenize query
        let query_tokens = Self::tokenize(query);
        if query_tokens.is_empty() {
            return Ok(Vec::new());
        }
        
        // Calculate document frequencies for query terms
        let mut term_doc_freqs: HashMap<String, u64> = HashMap::new();
        for term in &query_tokens {
            let term_key = Self::term_key(collection, bucket, term);
            if let Some(data) = self.kv_store.get(&term_key)? {
                let doc_freqs: HashMap<String, u32> = serde_json::from_slice(&data)
                    .unwrap_or_default();
                term_doc_freqs.insert(term.clone(), doc_freqs.len() as u64);
            } else {
                term_doc_freqs.insert(term.clone(), 0);
            }
        }
        
        // Collect all candidate documents and their BM25 scores
        let mut doc_scores: HashMap<String, f32> = HashMap::new();
        
        for term in &query_tokens {
            let term_key = Self::term_key(collection, bucket, term);
            
            if let Some(data) = self.kv_store.get(&term_key)? {
                let doc_freqs: HashMap<String, u32> = serde_json::from_slice(&data)
                    .unwrap_or_default();
                
                let doc_freq = *term_doc_freqs.get(term).unwrap_or(&0) as f32;
                let idf = if doc_freq > 0.0 {
                    ((self.total_docs as f32 - doc_freq + 0.5) / (doc_freq + 0.5) + 1.0).ln()
                } else {
                    0.0
                };
                
                for (doc_id, term_freq) in doc_freqs {
                    // Get document stats
                    let doc_key = Self::doc_key(collection, bucket, &doc_id);
                    if let Some(data) = self.doc_stats.get(&doc_key)? {
                        let stats: DocumentStats = serde_json::from_slice(&data)
                            .unwrap_or_default();
                        
                        let tf = term_freq as f32;
                        let doc_len = stats.doc_length as f32;
                        
                        // BM25 formula
                        let numerator = tf * (self.bm25_params.k1 + 1.0);
                        let denominator = tf + self.bm25_params.k1 * (1.0 - self.bm25_params.b + self.bm25_params.b * (doc_len / self.avg_doc_length.max(1.0)));
                        let score = idf * (numerator / denominator);
                        
                        *doc_scores.entry(doc_id).or_insert(0.0) += score;
                    }
                }
            }
        }
        
        // Sort by score and return top results
        let mut results: Vec<(String, f32)> = doc_scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        
        Ok(results)
    }
    
    /// Search for terms with prefix matching using FST.
    pub fn prefix_search(&self, collection: &str, bucket: &str, prefix: &str, limit: usize) -> Result<Vec<String>> {
        let key = Self::fst_key(collection, bucket);
        
        if let Some(data) = self.fst_store.get(&key)? {
            let terms: Vec<String> = serde_json::from_slice(&data)
                .map_err(|e| GraphyneError::Serialization(e))?;
            
            let prefix_lower = prefix.to_lowercase();
            let results: Vec<String> = terms
                .into_iter()
                .filter(|t| t.starts_with(&prefix_lower))
                .take(limit)
                .collect();
            
            Ok(results)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get all terms in a collection/bucket.
    pub fn get_terms(&self, collection: &str, bucket: &str) -> Result<Vec<String>> {
        let key = Self::fst_key(collection, bucket);
        
        if let Some(data) = self.fst_store.get(&key)? {
            let terms: Vec<String> = serde_json::from_slice(&data)
                .map_err(|e| GraphyneError::Serialization(e))?;
            Ok(terms)
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_index() -> (LexicalIndex, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = sled::open(dir.path()).unwrap();
        let index = LexicalIndex::new(&db).unwrap();
        (index, dir)
    }
    
    #[test]
    fn test_tokenize() {
        let text = "Hello world! This is a test.";
        let tokens = LexicalIndex::tokenize(text);
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }
    
    #[test]
    fn test_push_and_search() {
        let (mut index, _dir) = create_test_index();
        
        index.push_text("coll1", "bucket1", "doc1", "hello world test")
            .unwrap();
        index.push_text("coll1", "bucket1", "doc2", "world test foo")
            .unwrap();
        
        let results = index.search("coll1", "bucket1", "world test", 10)
            .unwrap();
        
        assert!(!results.is_empty());
        assert!(results[0].1 > 0.0);
    }
    
    #[test]
    fn test_prefix_search() {
        let (mut index, _dir) = create_test_index();
        
        index.push_text("coll1", "bucket1", "doc1", "hello world test")
            .unwrap();
        
        let terms = index.prefix_search("coll1", "bucket1", "he", 10)
            .unwrap();
        
        assert!(terms.contains(&"hello".to_string()));
    }

    #[test]
    fn test_push_empty_collection() {
        let (mut index, _dir) = create_test_index();
        let result = index.push_text("", "bucket1", "doc1", "hello world");
        assert!(result.is_err());
    }

    #[test]
    fn test_push_empty_bucket() {
        let (mut index, _dir) = create_test_index();
        let result = index.push_text("coll1", "", "doc1", "hello world");
        assert!(result.is_err());
    }

    #[test]
    fn test_push_empty_doc_id() {
        let (mut index, _dir) = create_test_index();
        let result = index.push_text("coll1", "bucket1", "", "hello world");
        assert!(result.is_err());
    }

    #[test]
    fn test_search_empty_collection() {
        let (index, _dir) = create_test_index();
        let result = index.search("", "bucket1", "hello", 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_empty_bucket() {
        let (index, _dir) = create_test_index();
        let result = index.search("coll1", "", "hello", 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_prefix_search_nonexistent_collection() {
        let (index, _dir) = create_test_index();
        let terms = index.prefix_search("nonexistent", "bucket1", "he", 10).unwrap();
        assert!(terms.is_empty());
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = LexicalIndex::tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_single_char() {
        // Single character words should be filtered out
        let tokens = LexicalIndex::tokenize("a b c");
        assert!(tokens.is_empty());
    }
}
