use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Import types from other memory modules
use crate::memory::RetentionPolicy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    Working,    // Short-term, immediate context
    Episodic,   // Event-based memories with timestamps
    Semantic,   // Factual knowledge
    Procedural, // How-to knowledge and skills
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryType::Working => write!(f, "Working"),
            MemoryType::Episodic => write!(f, "Episodic"),
            MemoryType::Semantic => write!(f, "Semantic"),
            MemoryType::Procedural => write!(f, "Procedural"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub importance: f32, // 0.0 to 1.0
}

impl MemoryEntry {
    pub fn new(memory_type: MemoryType, content: String, importance: f32) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            memory_type,
            content,
            embedding: None,
            metadata: serde_json::Value::Null,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            importance: importance.clamp(0.0, 1.0),
        }
    }

    pub fn access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpace {
    pub name: String,
    pub memory_types: Vec<MemoryType>,
    pub retention_policy: RetentionPolicy,
    pub max_entries: Option<usize>,
    pub scoring_config: ScoringConfig,
}

impl MemorySpace {
    pub fn new(name: String) -> Self {
        Self {
            name,
            memory_types: vec![MemoryType::Working, MemoryType::Episodic, MemoryType::Semantic, MemoryType::Procedural],
            retention_policy: RetentionPolicy::default(),
            max_entries: None,
            scoring_config: ScoringConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    pub recency_weight: f32,
    pub importance_weight: f32,
    pub access_weight: f32,
    pub relevance_weight: f32,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            recency_weight: 0.25,
            importance_weight: 0.25,
            access_weight: 0.25,
            relevance_weight: 0.25,
        }
    }
}
