//! Hybrid scoring system for combining retrieval results.
//!
//! This module provides functionality to combine and normalize scores from
//! different retrieval methods (lexical, vector, graph) into a unified ranking.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::error::{Result, GraphyneError};

/// Errors specific to scoring operations.
#[derive(Debug, Serialize, Deserialize)]
pub enum ScoringError {
    InvalidWeight(String),
    EmptyResults,
}

impl std::fmt::Display for ScoringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScoringError::InvalidWeight(msg) => write!(f, "Invalid weight: {}", msg),
            ScoringError::EmptyResults => write!(f, "No results to score"),
        }
    }
}

impl std::error::Error for ScoringError {}

/// Configuration for hybrid scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridScorerConfig {
    /// Weight for lexical search results (0.0 to 1.0)
    pub lexical_weight: f32,
    /// Weight for vector search results (0.0 to 1.0)
    pub vector_weight: f32,
    /// Weight for graph search results (0.0 to 1.0)
    pub graph_weight: f32,
    /// Whether to normalize scores to 0-1 range
    pub normalize: bool,
    /// Minimum score threshold (optional)
    pub min_score: Option<f32>,
}

impl Default for HybridScorerConfig {
    fn default() -> Self {
        Self {
            lexical_weight: 0.4,
            vector_weight: 0.4,
            graph_weight: 0.2,
            normalize: true,
            min_score: None,
        }
    }
}

impl HybridScorerConfig {
    /// Validate the configuration.
    pub fn validate(&self) -> std::result::Result<(), ScoringError> {
        if self.lexical_weight < 0.0 || self.lexical_weight > 1.0 {
            return Err(ScoringError::InvalidWeight(
                "Lexical weight must be between 0.0 and 1.0".to_string()
            ));
        }
        if self.vector_weight < 0.0 || self.vector_weight > 1.0 {
            return Err(ScoringError::InvalidWeight(
                "Vector weight must be between 0.0 and 1.0".to_string()
            ));
        }
        if self.graph_weight < 0.0 || self.graph_weight > 1.0 {
            return Err(ScoringError::InvalidWeight(
                "Graph weight must be between 0.0 and 1.0".to_string()
            ));
        }
        
        let sum = self.lexical_weight + self.vector_weight + self.graph_weight;
        if (sum - 1.0).abs() > 0.001 {
            return Err(ScoringError::InvalidWeight(
                format!("Weights must sum to 1.0, got {}", sum)
            ));
        }
        
        Ok(())
    }
}

/// Hybrid scorer for combining retrieval results.
pub struct HybridScorer {
    config: HybridScorerConfig,
}

impl HybridScorer {
    /// Create a new hybrid scorer with the given configuration.
    pub fn new(config: HybridScorerConfig) -> std::result::Result<Self, ScoringError> {
        config.validate()?;
        Ok(Self { config })
    }
    
    /// Create a new hybrid scorer with default configuration.
    pub fn default() -> Self {
        Self {
            config: HybridScorerConfig::default(),
        }
    }
    
    /// Normalize scores to 0-1 range.
    /// Assumes higher scores are better.
    fn normalize_scores(scores: &mut Vec<(String, f32)>) {
        if scores.is_empty() {
            return;
        }
        
        let max_score = scores.iter().map(|(_, s)| *s).fold(f32::NEG_INFINITY, f32::max);
        let min_score = scores.iter().map(|(_, s)| *s).fold(f32::INFINITY, f32::min);
        
        if (max_score - min_score).abs() < f32::EPSILON {
            // All scores are the same, set all to 1.0
            for (_, score) in scores.iter_mut() {
                *score = 1.0;
            }
        } else {
            for (_, score) in scores.iter_mut() {
                *score = (*score - min_score) / (max_score - min_score);
            }
        }
    }
    
    /// Invert scores (for cases where lower is better, like distances).
    fn invert_scores(scores: &mut Vec<(String, f32)>) {
        for (_, score) in scores.iter_mut() {
            if *score > 0.0 {
                *score = 1.0 / (1.0 + *score);
            } else {
                *score = 1.0;
            }
        }
    }
    
    /// Combine lexical, vector, and graph search results.
    /// 
    /// - `lexical`: Results from lexical search (doc_id, score) where higher is better
    /// - `vector`: Results from vector search (doc_id, distance) where lower is better
    /// - `graph`: Results from graph traversal (doc_id, score) where higher is better
    pub fn combine(
        &self,
        lexical: Vec<(String, f32)>,
        vector: Vec<(String, f32)>,
        graph: Vec<(String, f32)>,
    ) -> Vec<(String, f32)> {
        // Collect all unique document IDs
        let mut all_docs: HashMap<String, (f32, f32, f32)> = HashMap::new();
        
        // Process lexical results (higher is better)
        let mut lexical_normalized = lexical.clone();
        if self.config.normalize && !lexical_normalized.is_empty() {
            Self::normalize_scores(&mut lexical_normalized);
        }
        for (doc_id, score) in lexical_normalized {
            let entry = all_docs.entry(doc_id).or_insert((0.0, 0.0, 0.0));
            entry.0 = score;
        }
        
        // Process vector results (lower distance is better, so we invert)
        let mut vector_normalized = vector.clone();
        if !vector_normalized.is_empty() {
            // Invert distances so higher is better
            Self::invert_scores(&mut vector_normalized);
            if self.config.normalize {
                Self::normalize_scores(&mut vector_normalized);
            }
        }
        for (doc_id, score) in vector_normalized {
            let entry = all_docs.entry(doc_id).or_insert((0.0, 0.0, 0.0));
            entry.1 = score;
        }
        
        // Process graph results (higher is better)
        let mut graph_normalized = graph.clone();
        if self.config.normalize && !graph_normalized.is_empty() {
            Self::normalize_scores(&mut graph_normalized);
        }
        for (doc_id, score) in graph_normalized {
            let entry = all_docs.entry(doc_id).or_insert((0.0, 0.0, 0.0));
            entry.2 = score;
        }
        
        // Combine scores with weights
        let mut results: Vec<(String, f32)> = all_docs
            .into_iter()
            .map(|(doc_id, (lex_score, vec_score, graph_score))| {
                let combined = (lex_score * self.config.lexical_weight)
                    + (vec_score * self.config.vector_weight)
                    + (graph_score * self.config.graph_weight);
                (doc_id, combined)
            })
            .collect();
        
        // Apply minimum score threshold if set
        if let Some(min_score) = self.config.min_score {
            results.retain(|(_, score)| *score >= min_score);
        }
        
        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        results
    }
    
    /// Combine only lexical and vector results (no graph).
    pub fn combine_lexical_vector(
        &self,
        lexical: Vec<(String, f32)>,
        vector: Vec<(String, f32)>,
    ) -> Vec<(String, f32)> {
        let graph: Vec<(String, f32)> = Vec::new();
        self.combine(lexical, vector, graph)
    }
    
    /// Combine only lexical and graph results (no vector).
    pub fn combine_lexical_graph(
        &self,
        lexical: Vec<(String, f32)>,
        graph: Vec<(String, f32)>,
    ) -> Vec<(String, f32)> {
        let vector: Vec<(String, f32)> = Vec::new();
        self.combine(lexical, vector, graph)
    }
    
    /// Combine only vector and graph results (no lexical).
    pub fn combine_vector_graph(
        &self,
        vector: Vec<(String, f32)>,
        graph: Vec<(String, f32)>,
    ) -> Vec<(String, f32)> {
        let lexical: Vec<(String, f32)> = Vec::new();
        self.combine(lexical, vector, graph)
    }
    
    /// Get the current configuration.
    pub fn get_config(&self) -> &HybridScorerConfig {
        &self.config
    }
    
    /// Update the configuration.
    pub fn set_config(&mut self, config: HybridScorerConfig) -> std::result::Result<(), ScoringError> {
        config.validate()?;
        self.config = config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hybrid_scorer_config_validation() {
        let mut config = HybridScorerConfig::default();
        assert!(config.validate().is_ok());
        
        // Invalid: weight > 1.0
        config.lexical_weight = 1.5;
        assert!(config.validate().is_err());
        
        // Invalid: weights don't sum to 1.0
        config.lexical_weight = 0.5;
        config.vector_weight = 0.5;
        config.graph_weight = 0.5;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_combine_results() {
        let scorer = HybridScorer::default();
        
        let lexical = vec![
            ("doc1".to_string(), 2.5),
            ("doc2".to_string(), 1.8),
        ];
        
        let vector = vec![
            ("doc1".to_string(), 0.3), // distance, lower is better
            ("doc3".to_string(), 0.1),
        ];
        
        let graph = vec![
            ("doc2".to_string(), 0.9),
            ("doc3".to_string(), 0.7),
        ];
        
        let results = scorer.combine(lexical, vector, graph);
        
        assert!(!results.is_empty());
        assert_eq!(results.len(), 3); // doc1, doc2, doc3
        
        // Results should be sorted by score descending
        for i in 1..results.len() {
            assert!(results[i-1].1 >= results[i].1);
        }
    }
    
    #[test]
    fn test_combine_empty_results() {
        let scorer = HybridScorer::default();
        
        let lexical: Vec<(String, f32)> = Vec::new();
        let vector: Vec<(String, f32)> = Vec::new();
        let graph: Vec<(String, f32)> = Vec::new();
        
        let results = scorer.combine(lexical, vector, graph);
        assert!(results.is_empty());
    }
    
    #[test]
    fn test_normalize_scores() {
        let mut scores = vec![
            ("doc1".to_string(), 10.0),
            ("doc2".to_string(), 5.0),
            ("doc3".to_string(), 0.0),
        ];
        
        HybridScorer::normalize_scores(&mut scores);
        
        // After normalization, scores should be in 0-1 range
        for (_, score) in &scores {
            assert!(*score >= 0.0 && *score <= 1.0);
        }
        
        // Check specific values
        assert_eq!(scores[0].1, 1.0); // max
        assert_eq!(scores[2].1, 0.0); // min
    }
    
    #[test]
    fn test_invert_scores() {
        let mut scores = vec![
            ("doc1".to_string(), 0.0),
            ("doc2".to_string(), 1.0),
            ("doc3".to_string(), 3.0),
        ];
        
        HybridScorer::invert_scores(&mut scores);
        
        // After inversion, lower distances should have higher scores
        assert!(scores[0].1 > scores[1].1);
        assert!(scores[1].1 > scores[2].1);
    }
}
