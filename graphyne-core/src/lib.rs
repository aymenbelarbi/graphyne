pub mod error;
pub mod storage;
pub mod lexical;
pub mod vector;
pub mod graph;
pub mod scoring;
pub mod memory;  // New module for Phase 4
pub mod metrics;   // New module for Phase 6
pub mod logging;   // New module for Phase 6
pub mod health;    // New module for Phase 6
pub mod admin;     // New module for Phase 6

#[cfg(feature = "embeddings")]
pub mod embeddings;

#[cfg(feature = "plugins")]
pub mod plugins;

pub use error::{GraphyneError, Result};
