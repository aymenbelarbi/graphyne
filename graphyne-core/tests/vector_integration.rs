use graphyne_core::vector::{VectorIndex, HnswConfig};
use tempfile::TempDir;

fn create_test_index(dim: usize) -> (VectorIndex, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = sled::open(dir.path()).unwrap();
    let config = HnswConfig {
        max_connections: 16,
        num_layers: 5,
        ef_construction: 200,
        dimension: dim,
    };
    let index = VectorIndex::new(&db, Some(config)).unwrap();
    (index, dir)
}

fn random_normalized_vector(dim: usize, seed: u32) -> Vec<f32> {
    // Simple deterministic pseudo-random vector based on seed
    let mut vec = Vec::with_capacity(dim);
    let mut s = seed;
    for _ in 0..dim {
        // Simple LCG
        s = s.wrapping_mul(1103515245).wrapping_add(12345);
        let val = ((s % 1000) as f32 / 1000.0) - 0.5;
        vec.push(val);
    }
    // Normalize
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        vec.iter().map(|x| x / norm).collect()
    } else {
        vec
    }
}

#[test]
fn test_add_and_search_vectors() {
    let (mut index, _dir) = create_test_index(3);

    // Add 10 random vectors
    for i in 0..10 {
        let vec = random_normalized_vector(3, i + 1);
        index.add_embedding(&format!("vec_{}", i), &vec).unwrap();
    }

    // Search for nearest neighbor to vec_0
    let query = random_normalized_vector(3, 1); // Same seed as vec_0
    let results = index.search(&query, 3).unwrap();

    assert!(!results.is_empty(), "Should find results");
    assert!(results.len() <= 3, "Should return at most k results");

    // The nearest neighbor should have a positive similarity score
    for (_, score) in &results {
        assert!(*score > 0.0, "Similarity score should be positive");
    }
}

#[test]
fn test_dimension_mismatch_rejected() {
    let (mut index, _dir) = create_test_index(384);

    // Try to add a 3-dim vector to a 384-dim index
    let wrong_vec = vec![1.0, 0.0, 0.0];
    let result = index.add_embedding("bad_vec", &wrong_vec);
    assert!(result.is_err(), "Should reject vector with wrong dimension");

    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("dimension") || err_str.contains("Dimension"),
        "Error should mention dimension mismatch, got: {}",
        err_str
    );
}

#[test]
fn test_search_returns_k_results() {
    let (mut index, _dir) = create_test_index(3);

    // Add 20 vectors
    for i in 0..20 {
        let vec = random_normalized_vector(3, i + 100);
        index.add_embedding(&format!("item_{}", i), &vec).unwrap();
    }

    // Search with k=5
    let query = random_normalized_vector(3, 100);
    let results = index.search(&query, 5).unwrap();

    assert_eq!(results.len(), 5, "Should return exactly k=5 results");
}

#[test]
fn test_remove_and_rebuild() {
    let (mut index, _dir) = create_test_index(3);

    // Add vectors
    for i in 0..5 {
        let vec = random_normalized_vector(3, i + 1);
        index.add_embedding(&format!("v{}", i), &vec).unwrap();
    }

    // Verify all exist
    for i in 0..5 {
        let emb = index.get_embedding(&format!("v{}", i)).unwrap();
        assert!(emb.is_some(), "v{} should exist after adding", i);
    }

    // Remove one
    index.remove_embedding("v2").unwrap();
    assert!(index.needs_rebuild(), "Should need rebuild after removal");

    // Trigger rebuild
    index.maybe_rebuild().unwrap();
    assert!(!index.needs_rebuild(), "Should not need rebuild after rebuilding");

    // Search should still work
    let query = random_normalized_vector(3, 1);
    let results = index.search(&query, 3).unwrap();
    assert!(!results.is_empty(), "Search should work after rebuild");

    // Removed vector should not be in sled
    let emb = index.get_embedding("v2").unwrap();
    assert!(emb.is_none(), "v2 should not exist after removal");
}
