use graphyne_core::lexical::LexicalIndex;
use tempfile::TempDir;

fn create_test_index() -> (LexicalIndex, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = sled::open(dir.path()).unwrap();
    let index = LexicalIndex::new(&db).unwrap();
    (index, dir)
}

#[test]
fn test_index_and_search() {
    let (mut index, _dir) = create_test_index();

    // Push text documents
    index.push_text("docs", "articles", "doc1", "The quick brown fox jumps over the lazy dog").unwrap();
    index.push_text("docs", "articles", "doc2", "A quick brown dog outpaces a swift fox").unwrap();
    index.push_text("docs", "articles", "doc3", "Lazy dogs and quick foxes are common in fables").unwrap();

    // Search for "quick fox"
    let results = index.search("docs", "articles", "quick fox", 10).unwrap();
    assert!(!results.is_empty(), "Should find results for 'quick fox'");

    // All results should have positive scores
    for (doc_id, score) in &results {
        assert!(*score > 0.0, "Score should be positive for doc {}", doc_id);
    }

    // Verify that documents containing both terms score higher than documents with fewer matches
    // doc1 has both "quick" and "fox", doc2 has both, doc3 has both
    // BM25 considers term frequency and document length, so shorter docs with same terms score higher
    let doc_ids: Vec<&str> = results.iter().map(|(id, _)| id.as_str()).collect();
    assert!(doc_ids.contains(&"doc1"), "doc1 should be in results");
    assert!(doc_ids.contains(&"doc2"), "doc2 should be in results");
    assert!(doc_ids.contains(&"doc3"), "doc3 should be in results");

    // Verify results are sorted by score (descending)
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1, "Results should be sorted by score descending");
    }
}

#[test]
fn test_prefix_autocomplete() {
    let (mut index, _dir) = create_test_index();

    // Push text with various terms
    index.push_text("vocab", "terms", "doc1", "algorithm algorithmic algebra algebraic").unwrap();
    index.push_text("vocab", "terms", "doc2", "binary bit byte buffer").unwrap();
    index.push_text("vocab", "terms", "doc3", "compute computer computational computing").unwrap();

    // Prefix search for "alg"
    let terms = index.prefix_search("vocab", "terms", "alg", 10).unwrap();
    assert!(terms.contains(&"algorithm".to_string()), "Should find 'algorithm'");
    assert!(terms.contains(&"algorithmic".to_string()), "Should find 'algorithmic'");
    assert!(terms.contains(&"algebra".to_string()), "Should find 'algebra'");
    assert!(terms.contains(&"algebraic".to_string()), "Should find 'algebraic'");

    // Prefix search for "comp"
    let terms = index.prefix_search("vocab", "terms", "comp", 10).unwrap();
    assert!(terms.contains(&"compute".to_string()));
    assert!(terms.contains(&"computer".to_string()));
    assert!(terms.contains(&"computational".to_string()));
    assert!(terms.contains(&"computing".to_string()));

    // Prefix search with no match
    let terms = index.prefix_search("vocab", "terms", "xyz", 10).unwrap();
    assert!(terms.is_empty());
}

#[test]
fn test_multiple_documents() {
    let (mut index, _dir) = create_test_index();

    // Push to multiple documents
    index.push_text("coll", "bucket", "rust_doc", "Rust is a systems programming language").unwrap();
    index.push_text("coll", "bucket", "python_doc", "Python is a high-level programming language").unwrap();
    index.push_text("coll", "bucket", "go_doc", "Go is designed for concurrent programming").unwrap();

    // Search for "programming language" - should match rust_doc and python_doc
    let results = index.search("coll", "bucket", "programming language", 10).unwrap();
    assert!(results.len() >= 2, "Should find at least 2 documents with 'programming language'");

    let doc_ids: Vec<&str> = results.iter().map(|(id, _)| id.as_str()).collect();
    assert!(doc_ids.contains(&"rust_doc"), "Should find rust_doc");
    assert!(doc_ids.contains(&"python_doc"), "Should find python_doc");

    // Search for "concurrent" - should only match go_doc
    let results = index.search("coll", "bucket", "concurrent", 10).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].0, "go_doc", "go_doc should be top result for 'concurrent'");
}

#[test]
fn test_term_frequency_affects_score() {
    let (mut index, _dir) = create_test_index();

    // Push documents with different term frequencies
    // doc1: "rust" appears once
    index.push_text("bench", "freq", "doc1", "rust is fast").unwrap();
    // doc2: "rust" appears 5 times
    index.push_text("bench", "freq", "doc2", "rust rust rust rust rust is very fast").unwrap();
    // doc3: "rust" appears 3 times
    index.push_text("bench", "freq", "doc3", "rust rust rust is also fast").unwrap();

    // Search for "rust"
    let results = index.search("bench", "freq", "rust", 10).unwrap();
    assert_eq!(results.len(), 3, "Should find all 3 documents");

    // doc2 should score highest (5 occurrences of "rust")
    assert_eq!(results[0].0, "doc2", "doc2 should score highest (5 occurrences)");

    // Verify scores are ordered correctly
    let scores: Vec<f32> = results.iter().map(|(_, s)| *s).collect();
    for i in 1..scores.len() {
        assert!(scores[i - 1] >= scores[i], "Scores should be in descending order");
    }
}
