use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::types::MemoryEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub max_age: Option<Duration>,
    pub max_entries: Option<usize>,
    pub importance_threshold: Option<f32>,
    pub auto_archive: bool,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_age: Some(Duration::days(30)),
            max_entries: Some(10000),
            importance_threshold: Some(0.3),
            auto_archive: false,
        }
    }
}

impl RetentionPolicy {
    pub fn should_retain(&self, entry: &MemoryEntry) -> bool {
        // Check importance threshold
        if let Some(threshold) = self.importance_threshold {
            if entry.importance < threshold {
                return false;
            }
        }

        // Check max age
        if let Some(max_age) = self.max_age {
            let age = Utc::now() - entry.created_at;
            if age > max_age {
                return false;
            }
        }

        true
    }

    pub fn prune(&self, entries: &mut Vec<MemoryEntry>) {
        // First, remove entries that don't meet retention criteria
        entries.retain(|entry| self.should_retain(entry));

        // Then, if max_entries is set, keep only the most important/recent ones
        if let Some(max_entries) = self.max_entries {
            if entries.len() > max_entries {
                // Sort by importance (descending) and then by last_accessed (descending)
                entries.sort_by(|a, b| {
                    match b.importance.partial_cmp(&a.importance) {
                        Some(std::cmp::Ordering::Equal) | None => {
                            b.last_accessed.cmp(&a.last_accessed)
                        }
                        Some(ordering) => ordering,
                    }
                });
                entries.truncate(max_entries);
            }
        }
    }

    pub fn is_expired(&self, entry: &MemoryEntry) -> bool {
        if let Some(max_age) = self.max_age {
            let age = Utc::now() - entry.created_at;
            if age > max_age {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_entry(age_hours: i64, importance: f32) -> MemoryEntry {
        let now = Utc::now();
        MemoryEntry {
            id: uuid::Uuid::new_v4().to_string(),
            memory_type: super::super::types::MemoryType::Working,
            content: "Test content".to_string(),
            embedding: None,
            metadata: serde_json::Value::Null,
            created_at: now - Duration::hours(age_hours),
            last_accessed: now - Duration::hours(age_hours),
            access_count: 0,
            importance,
        }
    }

    #[test]
    fn test_should_retain_by_importance() {
        let policy = RetentionPolicy {
            max_age: None,
            max_entries: None,
            importance_threshold: Some(0.5),
            auto_archive: false,
        };

        let low_importance = create_test_entry(0, 0.3);
        let high_importance = create_test_entry(0, 0.7);

        assert!(!policy.should_retain(&low_importance));
        assert!(policy.should_retain(&high_importance));
    }

    #[test]
    fn test_should_retain_by_age() {
        let policy = RetentionPolicy {
            max_age: Some(Duration::hours(24)),
            max_entries: None,
            importance_threshold: None,
            auto_archive: false,
        };

        let old_entry = create_test_entry(48, 0.8);
        let new_entry = create_test_entry(12, 0.8);

        assert!(!policy.should_retain(&old_entry));
        assert!(policy.should_retain(&new_entry));
    }

    #[test]
    fn test_prune_max_entries() {
        let policy = RetentionPolicy {
            max_age: None,
            max_entries: Some(2),
            importance_threshold: None,
            auto_archive: false,
        };

        let mut entries = vec![
            create_test_entry(0, 0.3),
            create_test_entry(0, 0.8),
            create_test_entry(0, 0.5),
        ];

        policy.prune(&mut entries);
        assert_eq!(entries.len(), 2);
        // Should keep highest importance entries
        assert!(entries.iter().all(|e| e.importance >= 0.5));
    }
}
