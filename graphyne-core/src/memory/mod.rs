//! Agent & Memory system for Graphyne.
//!
//! This module provides the core memory system that makes Graphyne unique as an
//! agent-first memory architecture. It supports multiple memory types (Working,
//! Episodic, Semantic, Procedural) with retention policies, scoring, and
//! integration with lexical, vector, and graph stores.

pub mod types;
pub mod retention;
pub mod store;
pub mod context;
pub mod scoring;

pub use types::{MemoryType, MemoryEntry, MemorySpace, ScoringConfig};
pub use retention::RetentionPolicy;
pub use store::MemoryStore;
pub use context::ContextPacker;
pub use scoring::MemoryScorer;

use crate::error::Result;

/// Query for recalling memories from the store.
#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub query_text: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub memory_types: Option<Vec<MemoryType>>,
    pub time_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    pub min_importance: Option<f32>,
    pub limit: usize,
    pub include_metadata: bool,
}

impl Default for MemoryQuery {
    fn default() -> Self {
        Self {
            query_text: None,
            embedding: None,
            memory_types: None,
            time_range: None,
            min_importance: None,
            limit: 10,
            include_metadata: false,
        }
    }
}
