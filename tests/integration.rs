//! Integration tests for ladybug-graph-rs.
//!
//! Tests full end-to-end workflows: schema creation, CRUD, traversal.

use ladybug_graph_rs::*;
use serial_test::serial;

/// Helper: create a social graph with Person nodes and KNOWS edges.
fn social_graph() -> (Graph, NodeId, NodeId, NodeId) {
    let g = Graph::in_memory().unwrap();

    let person = NodeSchema::new("id", ColumnType::Serial)
        .column("name", ColumnType::String)
        .column("age", ColumnType::Int64);
    g.create_node_table("Person", &person).unwrap();

    let knows = RelSchema::new().column("since", ColumnType::Int64);
    g.create_rel_table("KNOWS", "Person", "Person", &knows)
        .unwrap();

    let alice = g
        .create_node(
            "Person",
            props([
                ("name", Property::from("Alice")),
                ("age", Property::from(30i64)),
            ]),
        )
        .unwrap();
    let bob = g
        .create_node(
            "Person",
            props([
                ("name", Property::from("Bob")),
                ("age", Property::from(25i64)),
            ]),
        )
        .unwrap();
    let charlie = g
        .create_node(
            "Person",
            props([
                ("name", Property::from("Charlie")),
                ("age", Property::from(35i64)),
            ]),
        )
        .unwrap();

    g.create_edge(
        "KNOWS",
        alice,
        bob,
        props([("since", Property::from(2020i64))]),
    )
    .unwrap();
    g.create_edge(
        "KNOWS",
        bob,
        charlie,
        props([("since", Property::from(2021i64))]),
    )
    .unwrap();

    (g, alice, bob, charlie)
}

#[test]
#[serial]
fn full_crud_lifecycle() {
    let g = Graph::in_memory().unwrap();
    let schema = NodeSchema::new("id", ColumnType::Serial)
        .column("name", ColumnType::String)
        .column("score", ColumnType::Double);
    g.create_node_table("Item", &schema).unwrap();

    // Create
    let id = g
        .create_node(
            "Item",
            props([
                ("name", Property::from("Widget")),
                ("score", Property::from(9.5)),
            ]),
        )
        .unwrap();

    // Read
    let node = g.get_node("Item", id).unwrap().unwrap();
    assert_eq!(node.get_str("name"), Some("Widget"));

    // Update
    g.update_node("Item", id, props([("score", Property::from(8.0))]))
        .unwrap();

    // Verify update
    let rows = g
        .query(&format!(
            "MATCH (n:Item) WHERE n.id = {} RETURN n.score",
            id.offset,
        ))
        .unwrap();
    assert_eq!(rows[0][0].as_f64(), Some(8.0));

    // Delete
    g.delete_node("Item", id).unwrap();
    assert_eq!(g.count_nodes("Item").unwrap(), 0);
}

#[test]
#[serial]
fn edge_crud_lifecycle() {
    let (g, alice, bob, _) = social_graph();

    // Verify edge exists
    let edges = g.get_edges(alice, "KNOWS").unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].label, "KNOWS");

    // Delete edge
    g.delete_edge("KNOWS", alice, bob).unwrap();
    let edges = g.get_edges(alice, "KNOWS").unwrap();
    assert!(edges.is_empty());
}

#[test]
#[serial]
fn traversal_workflow() {
    let (g, alice, bob, _charlie) = social_graph();

    // Outgoing neighbors of Alice
    let neighbors = g.neighbors("Person", alice, Direction::Outgoing).unwrap();
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].get_str("name"), Some("Bob"));

    // BFS from Alice
    let reachable = g.bfs(alice, 5).unwrap();
    assert_eq!(reachable.len(), 2); // Bob and Charlie

    // Incoming neighbors of Bob
    let incoming = g.neighbors("Person", bob, Direction::Incoming).unwrap();
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].get_str("name"), Some("Alice"));
}

#[test]
#[serial]
fn raw_cypher_queries() {
    let (g, ..) = social_graph();

    let rows = g
        .query("MATCH (n:Person) RETURN n.name ORDER BY n.name")
        .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][0].as_str(), Some("Alice"));
    assert_eq!(rows[1][0].as_str(), Some("Bob"));
    assert_eq!(rows[2][0].as_str(), Some("Charlie"));
}

#[test]
#[serial]
fn parameterized_queries() {
    let (g, ..) = social_graph();

    let rows = g
        .execute(
            "MATCH (n:Person) WHERE n.age > $min_age RETURN n.name ORDER BY n.name",
            &[("min_age", Property::from(28i64))],
        )
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0].as_str(), Some("Alice"));
    assert_eq!(rows[1][0].as_str(), Some("Charlie"));
}

#[test]
#[serial]
fn schema_drop_table() {
    let g = Graph::in_memory().unwrap();
    let schema = NodeSchema::new("id", ColumnType::Serial);
    g.create_node_table("Temp", &schema).unwrap();
    g.drop_table("Temp").unwrap();
    // Creating a node should fail after drop
    let result = g.create_node("Temp", PropertyMap::new());
    assert!(result.is_err());
}

#[test]
#[serial]
fn find_nodes_with_complex_filter() {
    let (g, ..) = social_graph();

    let found = g
        .find_nodes("Person", "n.age >= 25 AND n.age <= 30")
        .unwrap();
    assert_eq!(found.len(), 2);
    let names: Vec<_> = found.iter().filter_map(|n| n.get_str("name")).collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Bob"));
}

#[test]
#[serial]
fn multiple_edge_types() {
    let g = Graph::in_memory().unwrap();
    let person = NodeSchema::new("id", ColumnType::Serial).column("name", ColumnType::String);
    g.create_node_table("Person", &person).unwrap();

    let knows = RelSchema::new();
    g.create_rel_table("KNOWS", "Person", "Person", &knows)
        .unwrap();
    let likes = RelSchema::new();
    g.create_rel_table("LIKES", "Person", "Person", &likes)
        .unwrap();

    let a = g
        .create_node("Person", props([("name", Property::from("A"))]))
        .unwrap();
    let b = g
        .create_node("Person", props([("name", Property::from("B"))]))
        .unwrap();

    g.create_edge("KNOWS", a, b, PropertyMap::new()).unwrap();
    g.create_edge("LIKES", a, b, PropertyMap::new()).unwrap();

    assert_eq!(g.count_edges(a, "KNOWS").unwrap(), 1);
    assert_eq!(g.count_edges(a, "LIKES").unwrap(), 1);
}

#[test]
#[serial]
fn persistent_database_with_tempdir() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_persist");

    // Create and populate
    {
        let g = Graph::open(&path).unwrap();
        let schema = NodeSchema::new("id", ColumnType::Serial).column("val", ColumnType::Int64);
        g.create_node_table("Data", &schema).unwrap();
        g.create_node("Data", props([("val", Property::from(42i64))]))
            .unwrap();
    }

    // Reopen and verify
    {
        let g = Graph::open(&path).unwrap();
        let rows = g.query("MATCH (n:Data) RETURN n.val").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0].as_i64(), Some(42));
    }
}

#[test]
#[serial]
fn cypher_builder_integration() {
    let (g, ..) = social_graph();

    let cypher = CypherBuilder::new()
        .match_pattern("(n:Person)")
        .where_clause("n.age > 28")
        .return_expr("n.name")
        .order_by("n.name")
        .build();

    let rows = g.query(&cypher).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0].as_str(), Some("Alice"));
    assert_eq!(rows[1][0].as_str(), Some("Charlie"));
}

#[test]
#[serial]
fn config_affects_database() {
    let cfg = GraphConfig::new().max_num_threads(2);
    let g = Graph::in_memory_with_config(cfg).unwrap();
    let rows = g.query("RETURN 1 + 1").unwrap();
    assert_eq!(rows[0][0].as_i64(), Some(2));
}

#[test]
#[serial]
fn empty_property_map_create() {
    let g = Graph::in_memory().unwrap();
    let schema = NodeSchema::new("id", ColumnType::Serial);
    g.create_node_table("Bare", &schema).unwrap();
    let id = g.create_node("Bare", PropertyMap::new()).unwrap();
    assert_eq!(id.offset, 0);
}

#[test]
#[serial]
fn node_properties_accessible() {
    let (g, alice, ..) = social_graph();
    let node = g.get_node("Person", alice).unwrap().unwrap();
    assert_eq!(node.label, "Person");
    assert_eq!(node.get_str("name"), Some("Alice"));
    assert_eq!(node.get_i64("age"), Some(30));
    assert!(node.get_str("nonexistent").is_none());
}
