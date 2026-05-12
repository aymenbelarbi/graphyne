use graphyne_core::graph::GraphStore;
use tempfile::TempDir;

fn create_test_store() -> (GraphStore, TempDir) {
    let dir = TempDir::new().unwrap();
    let db = sled::open(dir.path()).unwrap();
    let store = GraphStore::new(&db).unwrap();
    (store, dir)
}

#[test]
fn test_graph_crud() {
    let (mut store, _dir) = create_test_store();

    // Add nodes
    store.add_node("n1", "Person", "Alice", serde_json::json!({"age": 30}), None).unwrap();
    store.add_node("n2", "Person", "Bob", serde_json::json!({"age": 25}), None).unwrap();

    // Get node
    let node = store.get_node("n1").unwrap();
    assert!(node.is_some());
    let node = node.unwrap();
    assert_eq!(node.id, "n1");
    assert_eq!(node.node_type, "Person");
    assert_eq!(node.label, "Alice");

    // Add edge
    let edge_id = store.add_edge("n1", "n2", "knows", serde_json::json!({"since": 2020}), 1.0).unwrap();

    // Get edge
    let edge = store.get_edge(&edge_id).unwrap();
    assert!(edge.is_some());
    let edge = edge.unwrap();
    assert_eq!(edge.from, "n1");
    assert_eq!(edge.to, "n2");
    assert_eq!(edge.edge_type, "knows");
    assert!((edge.weight - 1.0).abs() < f32::EPSILON);

    // Remove edge first (before removing nodes, to avoid index invalidation issues)
    store.remove_edge(&edge_id).unwrap();
    let edge_gone = store.get_edge(&edge_id).unwrap();
    assert!(edge_gone.is_none());

    // Now remove both nodes (no edges, so no cascade concerns)
    store.remove_node("n1").unwrap();
    assert!(store.get_node("n1").unwrap().is_none());

    store.remove_node("n2").unwrap();
    assert!(store.get_node("n2").unwrap().is_none());
}

#[test]
fn test_traversal_from_node() {
    let (mut store, _dir) = create_test_store();

    // Create chain: A -> B -> C -> D
    store.add_node("A", "Entity", "Alpha", serde_json::json!({}), None).unwrap();
    store.add_node("B", "Entity", "Beta", serde_json::json!({}), None).unwrap();
    store.add_node("C", "Entity", "Gamma", serde_json::json!({}), None).unwrap();
    store.add_node("D", "Entity", "Delta", serde_json::json!({}), None).unwrap();

    store.add_edge("A", "B", "connects", serde_json::json!({}), 1.0).unwrap();
    store.add_edge("B", "C", "connects", serde_json::json!({}), 1.0).unwrap();
    store.add_edge("C", "D", "connects", serde_json::json!({}), 1.0).unwrap();

    // Traverse from A with depth 3
    let paths = store.traverse("A", 3).unwrap();

    // Should have paths to B (1 hop), C (2 hops), D (3 hops)
    assert!(!paths.is_empty());

    // Find the path that reaches D (3 hops = 4 nodes)
    let max_hop_path = paths.iter().max_by_key(|p| p.nodes.len()).unwrap();
    assert!(max_hop_path.nodes.len() >= 4, "Expected at least 4 nodes in longest path, got {}", max_hop_path.nodes.len());

    // Verify the path goes through the chain
    let node_ids: Vec<&str> = max_hop_path.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(node_ids.contains(&"A"));
    assert!(node_ids.contains(&"D"));
}

#[test]
fn test_shortest_path() {
    let (mut store, _dir) = create_test_store();

    // Create a simple chain: A -> B -> C
    store.add_node("A", "Entity", "Alpha", serde_json::json!({}), None).unwrap();
    store.add_node("B", "Entity", "Beta", serde_json::json!({}), None).unwrap();
    store.add_node("C", "Entity", "Gamma", serde_json::json!({}), None).unwrap();

    store.add_edge("A", "B", "step1", serde_json::json!({}), 1.0).unwrap();
    store.add_edge("B", "C", "step2", serde_json::json!({}), 1.0).unwrap();

    let path = store.shortest_path("A", "C").unwrap();
    assert!(path.is_some());
    let path = path.unwrap();

    // Path should be A -> B -> C with total weight 2.0
    assert_eq!(path.nodes.len(), 3); // A, B, C
    assert_eq!(path.nodes[0].id, "A");
    assert_eq!(path.nodes[1].id, "B");
    assert_eq!(path.nodes[2].id, "C");
    assert!((path.total_weight - 2.0).abs() < f32::EPSILON, "Total weight should be 2.0, got {}", path.total_weight);
    assert_eq!(path.edges.len(), 2);
}

#[test]
fn test_recommendations() {
    let (mut store, _dir) = create_test_store();

    // Create star graph: center -> A, center -> B, center -> C
    // With A -> D (2-hop from center)
    store.add_node("center", "Hub", "Center", serde_json::json!({}), None).unwrap();
    store.add_node("A", "Spoke", "NodeA", serde_json::json!({}), None).unwrap();
    store.add_node("B", "Spoke", "NodeB", serde_json::json!({}), None).unwrap();
    store.add_node("C", "Spoke", "NodeC", serde_json::json!({}), None).unwrap();
    store.add_node("D", "Spoke", "NodeD", serde_json::json!({}), None).unwrap();

    store.add_edge("center", "A", "connects", serde_json::json!({}), 1.0).unwrap();
    store.add_edge("center", "B", "connects", serde_json::json!({}), 0.8).unwrap();
    store.add_edge("center", "C", "connects", serde_json::json!({}), 0.5).unwrap();
    store.add_edge("A", "D", "connects", serde_json::json!({}), 0.9).unwrap();

    let recommendations = store.recommend_nodes("center", 10).unwrap();
    assert!(!recommendations.is_empty());

    // A should be top recommendation (highest direct edge weight: 1.0)
    assert_eq!(recommendations[0].0.id, "A");

    // Should include 2-hop neighbor D
    let rec_ids: Vec<&str> = recommendations.iter().map(|(n, _)| n.id.as_str()).collect();
    assert!(rec_ids.contains(&"D"), "Should include 2-hop neighbor D");
}

#[test]
fn test_node_centrality_distribution() {
    let (mut store, _dir) = create_test_store();

    // Create a graph with known degree distribution
    store.add_node("n1", "Entity", "One", serde_json::json!({}), None).unwrap();
    store.add_node("n2", "Entity", "Two", serde_json::json!({}), None).unwrap();
    store.add_node("n3", "Entity", "Three", serde_json::json!({}), None).unwrap();
    store.add_node("n4", "Entity", "Four", serde_json::json!({}), None).unwrap();

    // n1 connects to n2, n3, n4 (out-degree 3)
    store.add_edge("n1", "n2", "link", serde_json::json!({}), 1.0).unwrap();
    store.add_edge("n1", "n3", "link", serde_json::json!({}), 1.0).unwrap();
    store.add_edge("n1", "n4", "link", serde_json::json!({}), 1.0).unwrap();

    // n2 connects back to n1 (in-degree 1 for n1)
    store.add_edge("n2", "n1", "link", serde_json::json!({}), 1.0).unwrap();

    let cent_n1 = store.node_centrality("n1").unwrap();
    // n1: in_degree=1 (from n2), out_degree=3 (to n2,n3,n4) => (1+3)/2 = 2.0
    assert!((cent_n1 - 2.0).abs() < f32::EPSILON, "n1 centrality should be 2.0, got {}", cent_n1);

    let cent_n4 = store.node_centrality("n4").unwrap();
    // n4: in_degree=1 (from n1), out_degree=0 => (1+0)/2 = 0.5
    assert!((cent_n4 - 0.5).abs() < f32::EPSILON, "n4 centrality should be 0.5, got {}", cent_n4);

    // Verify centrality values are non-negative
    for id in &["n1", "n2", "n3", "n4"] {
        let c = store.node_centrality(id).unwrap();
        assert!(c >= 0.0, "Centrality should be non-negative, got {} for {}", c, id);
    }
}

#[test]
fn test_graph_persistence() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().to_path_buf();

    // Create graph and populate
    {
        let db = sled::open(&path).unwrap();
        let mut store = GraphStore::new(&db).unwrap();

        store.add_node("p1", "Person", "Alice", serde_json::json!({"role": "engineer"}), None).unwrap();
        store.add_node("p2", "Person", "Bob", serde_json::json!({"role": "designer"}), None).unwrap();
        store.add_edge("p1", "p2", "knows", serde_json::json!({"since": 2020}), 1.0).unwrap();

        assert_eq!(store.node_count(), 2);
        assert_eq!(store.edge_count(), 1);
    }

    // Drop and reload from same sled DB
    {
        let db = sled::open(&path).unwrap();
        let store = GraphStore::new(&db).unwrap();

        // Verify data persisted
        assert_eq!(store.node_count(), 2);
        assert_eq!(store.edge_count(), 1);

        let alice = store.get_node("p1").unwrap();
        assert!(alice.is_some());
        let alice = alice.unwrap();
        assert_eq!(alice.label, "Alice");
        assert_eq!(alice.node_type, "Person");

        let bob = store.get_node("p2").unwrap();
        assert!(bob.is_some());

        let edges = store.get_edges_from("p1").unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].edge_type, "knows");
    }
}
