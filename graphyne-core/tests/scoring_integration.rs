use graphyne_core::scoring::{HybridScorer, HybridScorerConfig};
use graphyne_core::memory::{MemoryEntry, MemoryScorer, MemoryType};
use chrono::{Duration, Utc};

#[test]
fn test_hybrid_combine_all_sources() {
    let config = HybridScorerConfig {
        lexical_weight: 0.4,
        vector_weight: 0.4,
        graph_weight: 0.2,
        normalize: true,
        min_score: None,
    };
    let scorer = HybridScorer::new(config).unwrap();

    let lexical = vec![
        ("doc1".to_string(), 2.5),
        ("doc2".to_string(), 1.8),
        ("doc3".to_string(), 1.2),
    ];

    let vector = vec![
        ("doc1".to_string(), 0.2), // distance (lower is better)
        ("doc2".to_string(), 0.5),
        ("doc4".to_string(), 0.1),
    ];

    let graph = vec![
        ("doc1".to_string(), 0.9),
        ("doc3".to_string(), 0.7),
        ("doc5".to_string(), 0.5),
    ];

    let results = scorer.combine(lexical, vector, graph);

    // Should have 5 unique documents: doc1, doc2, doc3, doc4, doc5
    assert_eq!(results.len(), 5);

    // doc1 appears in all 3 sources, should be top
    assert_eq!(results[0].0, "doc1", "doc1 should be top (appears in all sources)");

    // Results should be sorted descending
    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results should be sorted descending: {:?}",
            results
        );
    }
}

#[test]
fn test_scorer_normalization() {
    let config = HybridScorerConfig {
        lexical_weight: 0.5,
        vector_weight: 0.5,
        graph_weight: 0.0,
        normalize: true,
        min_score: None,
    };
    let scorer = HybridScorer::new(config).unwrap();

    // Provide scores with different ranges — doc1 is better in both sources
    // (higher lexical score, lower vector distance) so it should rank first
    // after normalization.
    let lexical = vec![
        ("doc1".to_string(), 10.0),
        ("doc2".to_string(), 2.0),
    ];

    let vector = vec![
        ("doc1".to_string(), 0.2),
        ("doc2".to_string(), 0.8),
    ];

    let results = scorer.combine(lexical, vector, vec![]);

    // After normalization, all scores should be in a reasonable range
    for (doc_id, score) in &results {
        assert!(
            *score >= 0.0 && *score <= 2.0,
            "Score for {} should be in reasonable range, got {}",
            doc_id,
            score
        );
    }

    // Results should be sorted in descending order
    assert_eq!(results.len(), 2);
    assert!(results[0].1 >= results[1].1, "Results should be sorted descending");

    // doc1 should appear before doc2: after normalization both sources give
    // doc1=1.0, doc2=0.0 (lexical) and doc1=1.0, doc2=0.0 (vector inverted+normalized),
    // so doc1 combined = 1.0*0.5 + 1.0*0.5 = 1.0, doc2 = 0.0
    assert_eq!(results[0].0, "doc1", "doc1 should be ranked first");
    assert_eq!(results[1].0, "doc2", "doc2 should be ranked second");
}

#[test]
fn test_memory_scorer_with_time_decay() {
    let scorer = MemoryScorer::default();

    // Create a recent entry (accessed just now)
    let recent_entry = MemoryEntry {
        id: "recent".to_string(),
        memory_type: MemoryType::Semantic,
        content: "Recent memory".to_string(),
        embedding: None,
        metadata: serde_json::Value::Null,
        created_at: Utc::now(),
        last_accessed: Utc::now(),
        access_count: 5,
        importance: 0.8,
    };

    // Create an old entry (accessed 30 days ago)
    let old_entry = MemoryEntry {
        id: "old".to_string(),
        memory_type: MemoryType::Semantic,
        content: "Old memory".to_string(),
        embedding: None,
        metadata: serde_json::Value::Null,
        created_at: Utc::now() - Duration::days(60),
        last_accessed: Utc::now() - Duration::days(30),
        access_count: 5,
        importance: 0.8,
    };

    let relevance = 0.5;
    let recent_score = scorer.score(&recent_entry, relevance);
    let old_score = scorer.score(&old_entry, relevance);

    // Recent entry should score higher due to recency
    assert!(
        recent_score > old_score,
        "Recent entry should score higher than old entry. recent={}, old={}",
        recent_score,
        old_score
    );

    // Both scores should be in 0-1 range
    assert!(recent_score >= 0.0 && recent_score <= 1.0);
    assert!(old_score >= 0.0 && old_score <= 1.0);
}
