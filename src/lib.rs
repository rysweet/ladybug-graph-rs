//! # ladybug-graph-rs
//!
//! High-level, ergonomic Rust wrapper for LadybugDB, an embedded graph
//! database that uses the Cypher query language.
//!
//! ## Quick Start
//!
//! ```no_run
//! use ladybug_graph_rs::*;
//!
//! let g = Graph::in_memory().unwrap();
//!
//! // Create a node table
//! let schema = NodeSchema::new("id", ColumnType::Serial)
//!     .column("name", ColumnType::String)
//!     .column("age", ColumnType::Int64);
//! g.create_node_table("Person", &schema).unwrap();
//!
//! // Create a node
//! let id = g.create_node("Person", props([
//!     ("name", Property::from("Alice")),
//!     ("age", Property::from(30i64)),
//! ])).unwrap();
//! ```

pub mod config;
pub mod convert;
pub mod cypher;
pub mod edge;
pub mod error;
pub mod graph;
pub mod node;
pub mod property;
pub mod schema;

mod edge_ops;
mod node_ops;
mod traversal;

// Re-exports for ergonomic usage
pub use config::GraphConfig;
pub use cypher::CypherBuilder;
pub use edge::{Direction, Edge};
pub use error::{Error, Result};
pub use graph::Graph;
pub use node::{Node, NodeId};
pub use property::{props, Property, PropertyMap};
pub use schema::{ColumnType, NodeSchema, RelSchema};
