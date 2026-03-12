# ladybug-graph-rs

High-level, ergonomic Rust wrapper for [LadybugDB](https://github.com/ladybugdb/ladybugdb), an embedded graph database that uses the Cypher query language.

Built on top of the [`lbug`](https://crates.io/crates/lbug) FFI bindings.

## Quick Start

```rust
use ladybug_graph_rs::*;

// Create an in-memory graph database
let g = Graph::in_memory().unwrap();

// Define schema
let schema = NodeSchema::new("id", ColumnType::Serial)
    .column("name", ColumnType::String)
    .column("age", ColumnType::Int64);
g.create_node_table("Person", &schema).unwrap();

let rel = RelSchema::new().column("since", ColumnType::Int64);
g.create_rel_table("KNOWS", "Person", "Person", &rel).unwrap();

// Create nodes
let alice = g.create_node("Person", props([
    ("name", Property::from("Alice")),
    ("age", Property::from(30i64)),
])).unwrap();

let bob = g.create_node("Person", props([
    ("name", Property::from("Bob")),
    ("age", Property::from(25i64)),
])).unwrap();

// Create edges
g.create_edge("KNOWS", alice, bob, props([
    ("since", Property::from(2020i64)),
])).unwrap();

// Query
let friends = g.neighbors("Person", alice, Direction::Outgoing).unwrap();
assert_eq!(friends[0].get_str("name"), Some("Bob"));
```

## Features

- **Type-safe property system** — `Property` enum maps cleanly to/from LadybugDB's `Value` types
- **Schema builder** — Fluent API for creating node and relationship tables
- **Full CRUD** — Create, read, update, delete for both nodes and edges
- **Graph traversal** — Neighbors, BFS, shortest path, reachable count
- **Cypher builder** — Construct queries programmatically with `CypherBuilder`
- **Raw Cypher** — Execute arbitrary Cypher when you need full control
- **Parameterized queries** — Safe parameter substitution via `execute()`
- **In-memory & persistent** — Works with both in-memory and on-disk databases
- **Thread-safe** — `Graph` is `Send + Sync`

## API Overview

### Graph (main entry point)

```rust
Graph::open("path/to/db")           // Open/create persistent DB
Graph::in_memory()                   // In-memory DB
Graph::open_with_config(path, cfg)   // With custom config
Graph::in_memory_with_config(cfg)    // In-memory with config
```

### Schema

```rust
let schema = NodeSchema::new("id", ColumnType::Serial)
    .column("name", ColumnType::String)
    .column("age", ColumnType::Int64);
g.create_node_table("Person", &schema).unwrap();

let rel = RelSchema::new().column("weight", ColumnType::Double);
g.create_rel_table("LIKES", "Person", "Person", &rel).unwrap();
```

Supported column types: `Serial`, `Bool`, `Int64`, `Double`, `String`, `Date`, `Timestamp`, `Blob`.

### Node CRUD

```rust
let id = g.create_node("Person", props([("name", Property::from("Alice"))])).unwrap();
let node = g.get_node("Person", id).unwrap();
g.update_node("Person", id, props([("name", Property::from("Alicia"))])).unwrap();
g.delete_node("Person", id).unwrap();
let found = g.find_nodes("Person", "n.age > 20").unwrap();
let all = g.all_nodes("Person").unwrap();
let count = g.count_nodes("Person").unwrap();
```

### Edge CRUD

```rust
g.create_edge("KNOWS", alice, bob, props([("since", Property::from(2020i64))])).unwrap();
let edges = g.get_edges(alice, "KNOWS").unwrap();
g.delete_edge("KNOWS", alice, bob).unwrap();
let count = g.count_edges(alice, "KNOWS").unwrap();
```

### Traversal

```rust
let neighbors = g.neighbors("Person", alice, Direction::Outgoing).unwrap();
let path = g.shortest_path(alice, charlie, 5).unwrap();
let reachable = g.bfs(alice, 3).unwrap();
let count = g.reachable_count(alice, 5).unwrap();
```

### Raw Cypher

```rust
let rows = g.query("MATCH (n:Person) RETURN n.name ORDER BY n.name").unwrap();
let rows = g.execute(
    "MATCH (n:Person) WHERE n.age > $min RETURN n.name",
    &[("min", Property::from(20i64))],
).unwrap();
```

### CypherBuilder

```rust
let cypher = CypherBuilder::new()
    .match_pattern("(n:Person)")
    .where_clause("n.age > 25")
    .return_expr("n.name, n.age")
    .order_by("n.age DESC")
    .limit(10)
    .build();
```

## Architecture

```
src/
├── lib.rs          — Re-exports
├── error.rs        — Error types (thiserror)
├── config.rs       — GraphConfig builder
├── property.rs     — Property enum + PropertyMap
├── node.rs         — Node, NodeId
├── edge.rs         — Edge, Direction
├── schema.rs       — NodeSchema, RelSchema, ColumnType
├── graph.rs        — Graph struct (open, query, execute)
├── node_ops.rs     — Node CRUD operations
├── edge_ops.rs     — Edge CRUD operations
├── traversal.rs    — BFS, neighbors, shortest path
├── convert.rs      — Value ↔ Property conversion
└── cypher/
    ├── mod.rs      — Re-exports
    ├── builder.rs  — CypherBuilder
    ├── params.rs   — Parameter helpers
    └── escape.rs   — String escaping
```

## Build Requirements

- Rust 1.70+
- CMake (lbug compiles C++ from source)
- C++ compiler (gcc/clang)

## License

MIT
