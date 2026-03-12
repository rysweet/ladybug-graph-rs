use crate::convert;
use crate::cypher::params::property_to_cypher_literal;
use crate::error::{Error, Result};
use crate::graph::Graph;
use crate::node::{Node, NodeId};
use crate::property::PropertyMap;

impl Graph {
    /// Create a new node in the given table with the specified properties.
    ///
    /// Returns the internal NodeId of the created node.
    pub fn create_node(&self, table: &str, props: PropertyMap) -> Result<NodeId> {
        if table.is_empty() {
            return Err(Error::InvalidArgument("table name cannot be empty".into()));
        }

        let cypher = if props.is_empty() {
            format!("CREATE (n:{table}) RETURN id(n)")
        } else {
            let pairs: Vec<String> = props
                .iter()
                .map(|(k, v)| format!("{k}: {}", property_to_cypher_literal(v)))
                .collect();
            format!("CREATE (n:{table} {{{}}}) RETURN id(n)", pairs.join(", "))
        };

        let rows = self.query_raw(&cypher)?;
        if rows.is_empty() || rows[0].is_empty() {
            return Err(Error::query("CREATE did not return a node id"));
        }
        convert::extract_node_id_from_value(&rows[0][0])
    }

    /// Get a node by its ID using a primary key lookup.
    pub fn get_node(&self, table: &str, id: NodeId) -> Result<Option<Node>> {
        let cypher = format!("MATCH (n:{table}) WHERE n.id = {} RETURN n", id.offset);
        let rows = self.query_raw(&cypher)?;
        if rows.is_empty() || rows[0].is_empty() {
            return Ok(None);
        }
        convert::extract_node_from_value(&rows[0][0])
    }

    /// Update properties on a node.
    pub fn update_node(&self, table: &str, id: NodeId, props: PropertyMap) -> Result<()> {
        if props.is_empty() {
            return Ok(());
        }

        let set_clauses: Vec<String> = props
            .iter()
            .map(|(k, v)| format!("n.{k} = {}", property_to_cypher_literal(v)))
            .collect();

        let cypher = format!(
            "MATCH (n:{table}) WHERE n.id = {} SET {}",
            id.offset,
            set_clauses.join(", ")
        );

        self.execute_cypher(&cypher)
    }

    /// Delete a node by id (detaches edges first).
    pub fn delete_node(&self, table: &str, id: NodeId) -> Result<()> {
        let cypher = format!(
            "MATCH (n:{table}) WHERE n.id = {} DETACH DELETE n",
            id.offset
        );
        self.execute_cypher(&cypher)
    }

    /// Find nodes matching a Cypher WHERE filter expression.
    ///
    /// The filter uses `n` as the variable name, e.g. `"n.age > 20"`.
    ///
    /// # Safety
    ///
    /// The `filter` string is interpolated directly into the Cypher query.
    /// Never pass unsanitized user input as the filter.
    pub fn find_nodes(&self, table: &str, filter: &str) -> Result<Vec<Node>> {
        let cypher = format!("MATCH (n:{table}) WHERE {filter} RETURN n");
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

    /// Get all nodes in a table.
    pub fn all_nodes(&self, table: &str) -> Result<Vec<Node>> {
        let cypher = format!("MATCH (n:{table}) RETURN n");
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

    /// Count nodes in a table.
    pub fn count_nodes(&self, table: &str) -> Result<u64> {
        let rows = self.query(&format!("MATCH (n:{table}) RETURN count(n)"))?;
        if rows.is_empty() || rows[0].is_empty() {
            return Ok(0);
        }
        rows[0][0]
            .as_i64()
            .map(|n| n as u64)
            .ok_or_else(|| Error::conversion("count did not return an integer"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::{props, Property};
    use crate::schema::{ColumnType, NodeSchema};

    fn setup_person_graph() -> Graph {
        let g = Graph::in_memory().unwrap();
        let schema = NodeSchema::new("id", ColumnType::Serial)
            .column("name", ColumnType::String)
            .column("age", ColumnType::Int64);
        g.create_node_table("Person", &schema).unwrap();
        g
    }

    #[test]
    #[serial_test::serial]
    fn create_node_returns_id() {
        let g = setup_person_graph();
        let id = g
            .create_node(
                "Person",
                props([
                    ("name", Property::from("Alice")),
                    ("age", Property::from(30i64)),
                ]),
            )
            .unwrap();
        assert_eq!(id.offset, 0);
    }

    #[test]
    #[serial_test::serial]
    fn create_node_empty_table_errors() {
        let g = Graph::in_memory().unwrap();
        let result = g.create_node("", PropertyMap::new());
        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn create_multiple_nodes_distinct_ids() {
        let g = setup_person_graph();
        let id1 = g
            .create_node(
                "Person",
                props([
                    ("name", Property::from("Alice")),
                    ("age", Property::from(30i64)),
                ]),
            )
            .unwrap();
        let id2 = g
            .create_node(
                "Person",
                props([
                    ("name", Property::from("Bob")),
                    ("age", Property::from(25i64)),
                ]),
            )
            .unwrap();
        assert_ne!(id1.offset, id2.offset);
    }

    #[test]
    #[serial_test::serial]
    fn get_node_found() {
        let g = setup_person_graph();
        let id = g
            .create_node(
                "Person",
                props([
                    ("name", Property::from("Alice")),
                    ("age", Property::from(30i64)),
                ]),
            )
            .unwrap();
        let node = g.get_node("Person", id).unwrap();
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.get_str("name"), Some("Alice"));
        assert_eq!(node.get_i64("age"), Some(30));
    }

    #[test]
    #[serial_test::serial]
    fn delete_node_succeeds() {
        let g = setup_person_graph();
        let id = g
            .create_node(
                "Person",
                props([
                    ("name", Property::from("Alice")),
                    ("age", Property::from(30i64)),
                ]),
            )
            .unwrap();
        g.delete_node("Person", id).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn find_nodes_by_filter() {
        let g = setup_person_graph();
        g.create_node(
            "Person",
            props([
                ("name", Property::from("Alice")),
                ("age", Property::from(30i64)),
            ]),
        )
        .unwrap();
        g.create_node(
            "Person",
            props([
                ("name", Property::from("Bob")),
                ("age", Property::from(25i64)),
            ]),
        )
        .unwrap();
        let found = g.find_nodes("Person", "n.age > 28").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].get_str("name"), Some("Alice"));
    }

    #[test]
    #[serial_test::serial]
    fn update_node_empty_props_is_noop() {
        let g = setup_person_graph();
        let id = g
            .create_node(
                "Person",
                props([
                    ("name", Property::from("Alice")),
                    ("age", Property::from(30i64)),
                ]),
            )
            .unwrap();
        g.update_node("Person", id, PropertyMap::new()).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn update_node_changes_property() {
        let g = setup_person_graph();
        let id = g
            .create_node(
                "Person",
                props([
                    ("name", Property::from("Alice")),
                    ("age", Property::from(30i64)),
                ]),
            )
            .unwrap();
        g.update_node("Person", id, props([("age", Property::from(31i64))]))
            .unwrap();
        let rows = g
            .query(&format!(
                "MATCH (n:Person) WHERE n.id = {} RETURN n.age",
                id.offset
            ))
            .unwrap();
        assert_eq!(rows[0][0].as_i64(), Some(31));
    }

    #[test]
    #[serial_test::serial]
    fn all_nodes_returns_all() {
        let g = setup_person_graph();
        g.create_node(
            "Person",
            props([
                ("name", Property::from("Alice")),
                ("age", Property::from(30i64)),
            ]),
        )
        .unwrap();
        g.create_node(
            "Person",
            props([
                ("name", Property::from("Bob")),
                ("age", Property::from(25i64)),
            ]),
        )
        .unwrap();
        let nodes = g.all_nodes("Person").unwrap();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    #[serial_test::serial]
    fn count_nodes_empty() {
        let g = setup_person_graph();
        assert_eq!(g.count_nodes("Person").unwrap(), 0);
    }

    #[test]
    #[serial_test::serial]
    fn count_nodes_after_insert() {
        let g = setup_person_graph();
        g.create_node(
            "Person",
            props([
                ("name", Property::from("Alice")),
                ("age", Property::from(30i64)),
            ]),
        )
        .unwrap();
        assert_eq!(g.count_nodes("Person").unwrap(), 1);
    }

    #[test]
    #[serial_test::serial]
    fn find_nodes_empty_result() {
        let g = setup_person_graph();
        let found = g.find_nodes("Person", "n.age > 100").unwrap();
        assert!(found.is_empty());
    }
}
