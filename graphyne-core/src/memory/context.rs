use super::types::MemoryEntry;

/// Strategy for truncating text when token budget is exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TruncationStrategy {
    /// Truncate from the beginning (keep the most recent)
    Head,
    /// Truncate from the end (keep the beginning)
    Tail,
    /// Truncate the middle, keeping beginning and end
    Middle,
    /// Summarize (placeholder for future implementation)
    Summarize,
}

impl Default for TruncationStrategy {
    fn default() -> Self {
        TruncationStrategy::Middle
    }
}

/// Packs memories into a token budget for LLM context.
pub struct ContextPacker {
    max_tokens: usize,
    truncation_strategy: TruncationStrategy,
    /// Rough characters per token (default: 4)
    chars_per_token: f32,
}

impl Default for ContextPacker {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            truncation_strategy: TruncationStrategy::default(),
            chars_per_token: 4.0,
        }
    }
}

impl ContextPacker {
    /// Create a new ContextPacker with the specified token budget.
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            truncation_strategy: TruncationStrategy::default(),
            chars_per_token: 4.0,
        }
    }

    /// Set the truncation strategy.
    pub fn with_truncation_strategy(mut self, strategy: TruncationStrategy) -> Self {
        self.truncation_strategy = strategy;
        self
    }

    /// Pack memories into a string within the token budget.
    pub fn pack(&self, memories: Vec<(MemoryEntry, f32)>) -> String {
        if memories.is_empty() {
            return String::new();
        }

        // Sort by score (descending - highest score first)
        let mut sorted_memories = memories;
        sorted_memories.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Build context string, respecting token budget
        let mut result = String::new();
        let mut total_tokens = 0;

        for (entry, score) in &sorted_memories {
            let entry_text = self.format_memory_entry(entry, *score);
            let entry_tokens = self.estimate_tokens(&entry_text);

            if total_tokens + entry_tokens <= self.max_tokens {
                result.push_str(&entry_text);
                result.push('\n');
                total_tokens += entry_tokens;
            } else {
                // Try to fit partial content
                let remaining_tokens = self.max_tokens - total_tokens;
                if remaining_tokens > 10 {
                    // Add truncated version
                    let truncated = self.truncate_to_tokens(&entry_text, remaining_tokens);
                    result.push_str(&truncated);
                    result.push('\n');
                }
                break;
            }
        }

        result
    }

    /// Format a memory entry for context.
    fn format_memory_entry(&self, entry: &MemoryEntry, score: f32) -> String {
        format!(
            "[{memory_type}] (score: {score:.3}, importance: {importance:.2})\n{content}\n",
            memory_type = entry.memory_type,
            score = score,
            importance = entry.importance,
            content = entry.content
        )
    }

    /// Estimate the number of tokens in a text.
    pub fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f32 / self.chars_per_token).ceil() as usize
    }

    /// Truncate text to fit within a token budget.
    fn truncate_to_tokens(&self, text: &str, max_tokens: usize) -> String {
        let max_chars = (max_tokens as f32 * self.chars_per_token) as usize;
        
        if text.len() <= max_chars {
            return text.to_string();
        }

        match self.truncation_strategy {
            TruncationStrategy::Head => {
                // Keep the end
                let start = text.len() - max_chars;
                format!("...{}", &text[start..])
            }
            TruncationStrategy::Tail => {
                // Keep the beginning
                format!("{}...", &text[..max_chars])
            }
            TruncationStrategy::Middle => {
                // Keep beginning and end
                let half = max_chars / 2;
                let start = &text[..half];
                let end_start = text.len() - half;
                let end = &text[end_start..];
                format!("{}...{}", start, end)
            }
            TruncationStrategy::Summarize => {
                // Placeholder - for now, just truncate from middle
                let half = max_chars / 2;
                let start = &text[..half];
                let end_start = text.len() - half;
                let end = &text[end_start..];
                format!("{}...[truncated]...{}", start, end)
            }
        }
    }

    /// Get the maximum token budget.
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }

    /// Set the maximum token budget.
    pub fn set_max_tokens(&mut self, max_tokens: usize) {
        self.max_tokens = max_tokens;
    }

    /// Set the characters per token ratio.
    pub fn set_chars_per_token(&mut self, ratio: f32) {
        self.chars_per_token = ratio.max(1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::memory::types::MemoryType;

    fn create_test_entry(content: &str, importance: f32) -> (MemoryEntry, f32) {
        let entry = MemoryEntry {
            id: "test-id".to_string(),
            memory_type: MemoryType::Semantic,
            content: content.to_string(),
            embedding: None,
            metadata: serde_json::Value::Null,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            importance,
        };
        (entry, 0.5)
    }

    #[test]
    fn test_estimate_tokens() {
        let packer = ContextPacker::new(4096);
        assert_eq!(packer.estimate_tokens("Hello"), 2); // 5 chars / 4 = 1.25 -> 2
        assert_eq!(packer.estimate_tokens("This is a test"), 4); // 14 chars / 4 = 3.5 -> 4
    }

    #[test]
    fn test_pack_within_budget() {
        let packer = ContextPacker::new(100);
        let memories = vec![
            create_test_entry("Short text", 0.8),
            create_test_entry("Another short text", 0.6),
        ];
        let result = packer.pack(memories);
        assert!(!result.is_empty());
        assert!(packer.estimate_tokens(&result) <= 100);
    }

    #[test]
    fn test_pack_exceeds_budget() {
        let packer = ContextPacker::new(10); // Very small budget
        let memories = vec![
            create_test_entry("This is a very long text that will exceed the token budget", 0.9),
            create_test_entry("Another long text that should not be included", 0.7),
        ];
        let result = packer.pack(memories);
        assert!(packer.estimate_tokens(&result) <= 10);
    }
}
