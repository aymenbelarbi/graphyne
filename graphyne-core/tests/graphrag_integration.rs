use graphyne_core::graph::GraphStore;
use graphyne_core::graph::rag::GraphRAG;
use graphyne_core::lexical::LexicalIndex;
use graphyne_core::vector::VectorIndex;
use graphyne_core::vector::HnswConfig;
use tempfile::TempDir;

fn create_test_graphrag() -> (GraphRAG, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = sled::open(dir.path()).unwrap();

    let graph_store = GraphStore::new(&db).unwrap();
    let vector_index = VectorIndex::new(&db, Some(HnswConfig {
        max_connections: 16,
        num_layers: 5,
        ef_construction: 200,
        dimension: 3,
    })).unwrap();
    let lexical_index = LexicalIndex::new(&db).unwrap();

    let graphrag = GraphRAG::new(graph_store, vector_index, lexical_index, None);
    (graphrag, dir)
}

fn create_test_graphrag_with_data() -> (GraphRAG, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = sled::open(dir.path()).unwrap();

    let mut graph_store = GraphStore::new(&db).unwrap();
    let vector_index = VectorIndex::new(&db, Some(HnswConfig {
        max_connections: 16,
        num_layers: 5,
        ef_construction: 200,
        dimension: 3,
    })).unwrap();
    let mut lexical_index = LexicalIndex::new(&db).unwrap();

    // Create a chain: A -> B -> C -> D
    graph_store.add_node("A", "Entity", "Alpha node", serde_json::json!({"type": "start"}), None).unwrap();
    graph_store.add_node("B", "Entity", "Beta node", serde_json::json!({"type": "middle"}), None).unwrap();
    graph_store.add_node("C", "Entity", "Gamma node", serde_json::json!({"type": "middle"}), None).unwrap();
    graph_store.add_node("D", "Entity", "Delta node", serde_json::json!({"type": "end"}), None).unwrap();

    graph_store.add_edge("A", "B", "connects", serde_json::json!({}), 1.0).unwrap();
    graph_store.add_edge("B", "C", "connects", serde_json::json!({}), 1.0).unwrap();
    graph_store.add_edge("C", "D", "connects", serde_json::json!({}), 1.0).unwrap();

    // Index node labels in lexical search so seed nodes can be found
    lexical_index.push_text("default", "GraphNode", "A", "Alpha node").unwrap();
    lexical_index.push_text("default", "GraphNode", "B", "Beta node").unwrap();
    lexical_index.push_text("default", "GraphNode", "C", "Gamma node").unwrap();
    lexical_index.push_text("default", "GraphNode", "D", "Delta node").unwrap();

    let graphrag = GraphRAG::new(graph_store, vector_index, lexical_index, None);
    (graphrag, dir)
}

#[test]
fn test_graphrag_end_to_end() {
    let (graphrag, _dir) = create_test_graphrag_with_data();

    // Query for "Alpha" which should match node A via lexical search
    let result = graphrag.query("Alpha", Some(2), Some(10)).unwrap();

    // Should find at least the seed node
    assert!(result.confidence > 0.0, "Confidence should be > 0 when nodes are found");
    assert!(!result.nodes.is_empty(), "Should find at least one node matching 'Alpha'");

    // Verify context contains useful information
    assert!(!result.context.is_empty(), "Context should not be empty");
    assert!(result.context.contains("Alpha"), "Context should mention the query term");

    // Verify explanation
    assert!(!result.explanation.is_empty(), "Explanation should not be empty");
}

#[test]
fn test_graphrag_multi_hop_expansion() {
    let (graphrag, _dir) = create_test_graphrag_with_data();

    // Query from A with 3 hops should reach D (A -> B -> C -> D)
    let result = graphrag.query("Alpha", Some(3), Some(10)).unwrap();

    assert!(result.confidence > 0.0);
    assert!(!result.nodes.is_empty());

    // With 3 hops from A, should reach B, C, and D
    let node_ids: Vec<&str> = result.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(node_ids.contains(&"A"), "Should contain seed node A");
    assert!(node_ids.contains(&"B"), "Should contain 1-hop neighbor B");
    assert!(node_ids.contains(&"C"), "Should contain 2-hop neighbor C");
    assert!(node_ids.contains(&"D"), "Should contain 3-hop neighbor D");

    // Should have edges connecting the nodes
    assert!(!result.edges.is_empty(), "Should have edges in the result");
}

#[test]
fn test_graphrag_empty_graph_returns_gracefully() {
    let (graphrag, _dir) = create_test_graphrag();

    // Query empty graph - should not panic
    let result = graphrag.query("anything", None, None).unwrap();

    assert_eq!(result.confidence, 0.0);
    assert!(result.nodes.is_empty());
    assert!(result.edges.is_empty());
    assert!(!result.context.is_empty(), "Should return a graceful message");
}
