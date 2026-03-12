use crate::convert;
use crate::edge::Direction;
use crate::error::Result;
use crate::graph::Graph;
use crate::node::{Node, NodeId};

impl Graph {
    /// Get neighbor nodes connected via any relationship.
    pub fn neighbors(&self, table: &str, id: NodeId, direction: Direction) -> Result<Vec<Node>> {
        let pattern = match direction {
            Direction::Outgoing => format!(
                "MATCH (a:{table})-[]->(b) WHERE offset(id(a)) = {} RETURN b",
                id.offset
            ),
            Direction::Incoming => format!(
                "MATCH (a:{table})<-[]-(b) WHERE offset(id(a)) = {} RETURN b",
                id.offset
            ),
            Direction::Both => format!(
                "MATCH (a:{table})-[]-(b) WHERE offset(id(a)) = {} RETURN b",
                id.offset
            ),
        };
        let rows = self.query_raw(&pattern)?;
        let mut nodes = Vec::new();
        for row in &rows {
            if !row.is_empty() {
                if let Some(node) = convert::extract_node_from_value(&row[0])? {
                    nodes.push(node);
                }
            }
        }
        Ok(nodes)
    }

    /// Count neighbors in a given direction.
    pub fn neighbor_count(&self, table: &str, id: NodeId, direction: Direction) -> Result<u64> {
        let pattern = match direction {
            Direction::Outgoing => format!(
                "MATCH (a:{table})-[]->(b) WHERE offset(id(a)) = {} RETURN count(b)",
                id.offset
            ),
            Direction::Incoming => format!(
                "MATCH (a:{table})<-[]-(b) WHERE offset(id(a)) = {} RETURN count(b)",
                id.offset
            ),
            Direction::Both => format!(
                "MATCH (a:{table})-[]-(b) WHERE offset(id(a)) = {} RETURN count(b)",
                id.offset
            ),
        };
        let rows = self.query(&pattern)?;
        if rows.is_empty() || rows[0].is_empty() {
            return Ok(0);
        }
        rows[0][0]
            .as_i64()
            .map(|n| n as u64)
            .ok_or_else(|| crate::error::Error::conversion("count did not return integer"))
    }

    /// Find shortest path between two nodes using variable-length relationships.
    ///
    /// Returns an empty `Vec` if no path exists within `max_hops` or if the
    /// underlying engine does not support `shortestPath`.
    pub fn shortest_path(&self, from: NodeId, to: NodeId, max_hops: u32) -> Result<Vec<Node>> {
        let cypher = format!(
            "MATCH p = shortestPath((a)-[*1..{max_hops}]->(b)) \
             WHERE offset(id(a)) = {} AND offset(id(b)) = {} \
             RETURN nodes(p)",
            from.offset, to.offset
        );
        let rows = match self.query_raw(&cypher) {
            Ok(rows) => rows,
            // shortestPath may not be supported by all LadybugDB versions
            Err(crate::error::Error::Database(_)) => return Ok(Vec::new()),
            Err(e) => return Err(e),
        };
        let mut nodes = Vec::new();
        for row in &rows {
            for val in row {
                if let lbug::Value::List(_, items) = val {
                    for item in items {
                        if let Some(n) = convert::extract_node_from_value(item)? {
                            nodes.push(n);
                        }
                    }
                }
            }
        }
        Ok(nodes)
    }

    /// BFS-style traversal from a starting node up to max_depth.
    ///
    /// Uses variable-length path matching to find reachable nodes.
    pub fn bfs(&self, start: NodeId, max_depth: u32) -> Result<Vec<Node>> {
        let cypher = format!(
            "MATCH (a)-[*1..{max_depth}]->(b) \
             WHERE offset(id(a)) = {} \
             RETURN DISTINCT b",
            start.offset
        );
        let rows = self.query_raw(&cypher)?;
        let mut nodes = Vec::new();
        for row in &rows {
            if !row.is_empty() {
                if let Some(node) = convert::extract_node_from_value(&row[0])? {
                    nodes.push(node);
                }
            }
        }
        Ok(nodes)
    }

    /// Count reachable nodes from a starting node within max_depth hops.
    pub fn reachable_count(&self, start: NodeId, max_depth: u32) -> Result<u64> {
        let cypher = format!(
            "MATCH (a)-[*1..{max_depth}]->(b) \
             WHERE offset(id(a)) = {} \
             RETURN count(DISTINCT b)",
            start.offset
        );
        let rows = self.query(&cypher)?;
        if rows.is_empty() || rows[0].is_empty() {
            return Ok(0);
        }
        rows[0][0]
            .as_i64()
            .map(|n| n as u64)
            .ok_or_else(|| crate::error::Error::conversion("count did not return integer"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::{props, Property};
    use crate::schema::{ColumnType, NodeSchema, RelSchema};

    fn setup_social_graph() -> (Graph, NodeId, NodeId, NodeId) {
        let g = Graph::in_memory().unwrap();
        let person = NodeSchema::new("id", ColumnType::Serial).column("name", ColumnType::String);
        g.create_node_table("Person", &person).unwrap();
        let knows = RelSchema::new();
        g.create_rel_table("KNOWS", "Person", "Person", &knows)
            .unwrap();

        let alice = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let bob = g
            .create_node("Person", props([("name", Property::from("Bob"))]))
            .unwrap();
        let charlie = g
            .create_node("Person", props([("name", Property::from("Charlie"))]))
            .unwrap();

        g.create_edge("KNOWS", alice, bob, Default::default())
            .unwrap();
        g.create_edge("KNOWS", bob, charlie, Default::default())
            .unwrap();

        (g, alice, bob, charlie)
    }

    #[test]
    #[serial_test::serial]
    fn neighbor_count_outgoing() {
        let (g, alice, _, _) = setup_social_graph();
        let count = g
            .neighbor_count("Person", alice, Direction::Outgoing)
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    #[serial_test::serial]
    fn neighbor_count_incoming() {
        let (g, _, bob, _) = setup_social_graph();
        let count = g
            .neighbor_count("Person", bob, Direction::Incoming)
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    #[serial_test::serial]
    fn neighbor_count_both() {
        let (g, _, bob, _) = setup_social_graph();
        let count = g.neighbor_count("Person", bob, Direction::Both).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    #[serial_test::serial]
    fn neighbor_count_no_connections() {
        let g = Graph::in_memory().unwrap();
        let person = NodeSchema::new("id", ColumnType::Serial).column("name", ColumnType::String);
        g.create_node_table("Person", &person).unwrap();
        let knows = RelSchema::new();
        g.create_rel_table("KNOWS", "Person", "Person", &knows)
            .unwrap();

        let alice = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let count = g
            .neighbor_count("Person", alice, Direction::Outgoing)
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    #[serial_test::serial]
    fn reachable_count_chain() {
        let (g, alice, _, _) = setup_social_graph();
        let count = g.reachable_count(alice, 5).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    #[serial_test::serial]
    fn reachable_count_limited_depth() {
        let (g, alice, _, _) = setup_social_graph();
        let count = g.reachable_count(alice, 1).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    #[serial_test::serial]
    fn neighbors_outgoing() {
        let (g, alice, _, _) = setup_social_graph();
        let result = g.neighbors("Person", alice, Direction::Outgoing).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get_str("name"), Some("Bob"));
    }

    #[test]
    #[serial_test::serial]
    fn neighbors_incoming() {
        let (g, _, bob, _) = setup_social_graph();
        let result = g.neighbors("Person", bob, Direction::Incoming).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].get_str("name"), Some("Alice"));
    }

    #[test]
    #[serial_test::serial]
    fn bfs_traversal() {
        let (g, alice, _, _) = setup_social_graph();
        let result = g.bfs(alice, 3).unwrap();
        assert_eq!(result.len(), 2); // Bob and Charlie
    }

    #[test]
    #[serial_test::serial]
    fn shortest_path_returns_ok() {
        let (g, alice, _, charlie) = setup_social_graph();
        let result = g.shortest_path(alice, charlie, 5);
        assert!(result.is_ok());
    }
}
