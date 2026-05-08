use chrono::{DateTime, Utc};
use super::types::MemoryEntry;

/// Scores memory entries based on multiple factors.
pub struct MemoryScorer {
    pub recency_weight: f32,
    pub importance_weight: f32,
    pub access_weight: f32,
    pub relevance_weight: f32,
}

impl Default for MemoryScorer {
    fn default() -> Self {
        Self {
            recency_weight: 0.25,
            importance_weight: 0.25,
            access_weight: 0.25,
            relevance_weight: 0.25,
        }
    }
}

impl MemoryScorer {
    /// Create a new MemoryScorer with custom weights.
    pub fn new(recency_weight: f32, importance_weight: f32, access_weight: f32, relevance_weight: f32) -> Self {
        Self {
            recency_weight,
            importance_weight,
            access_weight,
            relevance_weight,
        }
    }

    /// Create a MemoryScorer from a ScoringConfig.
    pub fn from_config(config: &super::types::ScoringConfig) -> Self {
        Self {
            recency_weight: config.recency_weight,
            importance_weight: config.importance_weight,
            access_weight: config.access_weight,
            relevance_weight: config.relevance_weight,
        }
    }

    /// Score a memory entry combining all factors.
    pub fn score(&self, entry: &MemoryEntry, relevance: f32) -> f32 {
        let recency_score = self.compute_recency(entry);
        let access_score = self.compute_access_frequency(entry);
        
        let total = self.recency_weight * recency_score
            + self.importance_weight * entry.importance
            + self.access_weight * access_score
            + self.relevance_weight * relevance;

        // Normalize to 0-1 range
        total.min(1.0).max(0.0)
    }

    /// Compute recency score based on last access time.
    /// More recent = higher score (0.0 to 1.0).
    fn compute_recency(&self, entry: &MemoryEntry) -> f32 {
        let now = Utc::now();
        let duration = now.signed_duration_since(entry.last_accessed);
        
        // Convert to hours
        let hours_ago = duration.num_hours() as f32;
        
        // Score decays over time
        // Within 1 hour: 1.0
        // Within 24 hours: 0.8
        // Within 7 days: 0.5
        // Within 30 days: 0.2
        // Beyond: 0.1
        if hours_ago < 1.0 {
            1.0
        } else if hours_ago < 24.0 {
            1.0 - (hours_ago / 24.0) * 0.2  // 1.0 to 0.8
        } else if hours_ago < 24.0 * 7.0 {
            0.8 - ((hours_ago - 24.0) / (24.0 * 6.0)) * 0.3  // 0.8 to 0.5
        } else if hours_ago < 24.0 * 30.0 {
            0.5 - ((hours_ago - 24.0 * 7.0) / (24.0 * 23.0)) * 0.3  // 0.5 to 0.2
        } else {
            0.1  // Very old memories get minimal recency score
        }
    }

    /// Compute access frequency score.
    /// More accesses = higher score (0.0 to 1.0).
    fn compute_access_frequency(&self, entry: &MemoryEntry) -> f32 {
        // Logarithmic scale to prevent unbounded growth
        // 0 accesses: 0.0
        // 1 access: 0.5
        // 10 accesses: 0.75
        // 100 accesses: 1.0
        if entry.access_count == 0 {
            0.0
        } else {
            let count = entry.access_count as f32;
            (1.0 - 1.0 / (1.0 + count.ln())).min(1.0).max(0.0)
        }
    }

    /// Update weights.
    pub fn update_weights(&mut self, recency: Option<f32>, importance: Option<f32>, access: Option<f32>, relevance: Option<f32>) {
        if let Some(w) = recency {
            self.recency_weight = w;
        }
        if let Some(w) = importance {
            self.importance_weight = w;
        }
        if let Some(w) = access {
            self.access_weight = w;
        }
        if let Some(w) = relevance {
            self.relevance_weight = w;
        }
        
        // Normalize weights to sum to 1.0
        let sum = self.recency_weight + self.importance_weight + self.access_weight + self.relevance_weight;
        if sum > 0.0 {
            self.recency_weight /= sum;
            self.importance_weight /= sum;
            self.access_weight /= sum;
            self.relevance_weight /= sum;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use crate::memory::types::MemoryEntry;
    use crate::memory::types::MemoryType;

    fn create_test_entry(access_count: u32, importance: f32, hours_ago: i64) -> MemoryEntry {
        let now = Utc::now();
        MemoryEntry {
            id: "test-id".to_string(),
            memory_type: MemoryType::Semantic,
            content: "Test content".to_string(),
            embedding: None,
            metadata: serde_json::Value::Null,
            created_at: now - Duration::hours(hours_ago),
            last_accessed: now - Duration::hours(hours_ago),
            access_count,
            importance,
        }
    }

    #[test]
    fn test_score_combines_factors() {
        let scorer = MemoryScorer::default();
        let entry = create_test_entry(10, 0.8, 0); // Recent, high importance, accessed many times
        let relevance = 0.9;
        
        let score = scorer.score(&entry, relevance);
        assert!(score > 0.5); // Should be high due to all factors
    }

    #[test]
    fn test_recency_score() {
        let scorer = MemoryScorer::default();
        let recent = create_test_entry(0, 0.5, 0);
        let old = create_test_entry(0, 0.5, 48); // 48 hours ago
        
        let recent_score = scorer.compute_recency(&recent);
        let old_score = scorer.compute_recency(&old);
        
        assert!(recent_score > old_score);
    }

    #[test]
    fn test_access_frequency_score() {
        let scorer = MemoryScorer::default();
        let low_access = create_test_entry(1, 0.5, 0);
        let high_access = create_test_entry(100, 0.5, 0);
        
        let low_score = scorer.compute_access_frequency(&low_access);
        let high_score = scorer.compute_access_frequency(&high_access);
        
        assert!(high_score > low_score);
    }

    #[test]
    fn test_score_normalization() {
        let mut scorer = MemoryScorer::new(0.5, 0.5, 0.5, 0.5);
        scorer.update_weights(None, None, None, None);
        
        // Weights should be normalized to sum to 1.0
        let sum = scorer.recency_weight + scorer.importance_weight + scorer.access_weight + scorer.relevance_weight;
        assert!((sum - 1.0).abs() < 0.001);
    }
}
