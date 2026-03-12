use crate::convert;
use crate::cypher::params::property_to_cypher_literal;
use crate::edge::Edge;
use crate::error::{Error, Result};
use crate::graph::Graph;
use crate::node::NodeId;
use crate::property::PropertyMap;

impl Graph {
    /// Create an edge between two nodes.
    pub fn create_edge(
        &self,
        table: &str,
        from: NodeId,
        to: NodeId,
        props: PropertyMap,
    ) -> Result<()> {
        if table.is_empty() {
            return Err(Error::InvalidArgument("table name cannot be empty".into()));
        }

        let prop_clause = if props.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = props
                .iter()
                .map(|(k, v)| format!("{k}: {}", property_to_cypher_literal(v)))
                .collect();
            format!(" {{{}}}", pairs.join(", "))
        };

        let cypher = format!(
            "MATCH (a), (b) WHERE offset(id(a)) = {} AND offset(id(b)) = {} \
             CREATE (a)-[r:{table}{prop_clause}]->(b)",
            from.offset, to.offset
        );
        self.execute_cypher(&cypher)
    }

    /// Get outgoing edges from a node for a specific relationship table.
    pub fn get_edges(&self, from: NodeId, edge_table: &str) -> Result<Vec<Edge>> {
        let cypher = format!(
            "MATCH (a)-[r:{edge_table}]->(b) \
             WHERE offset(id(a)) = {} \
             RETURN r",
            from.offset
        );
        let rows = self.query_raw(&cypher)?;
        let mut edges = Vec::new();
        for row in &rows {
            if !row.is_empty() {
                if let Some(edge) = convert::extract_edge_from_value(&row[0])? {
                    edges.push(edge);
                }
            }
        }
        Ok(edges)
    }

    /// Delete an edge between two nodes.
    pub fn delete_edge(&self, table: &str, from: NodeId, to: NodeId) -> Result<()> {
        let cypher = format!(
            "MATCH (a)-[r:{table}]->(b) \
             WHERE offset(id(a)) = {} AND offset(id(b)) = {} \
             DELETE r",
            from.offset, to.offset
        );
        self.execute_cypher(&cypher)
    }

    /// Count edges of a given type from a node.
    pub fn count_edges(&self, from: NodeId, edge_table: &str) -> Result<u64> {
        let cypher = format!(
            "MATCH (a)-[r:{edge_table}]->() \
             WHERE offset(id(a)) = {} \
             RETURN count(r)",
            from.offset
        );
        let rows = self.query(&cypher)?;
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
    use crate::schema::{ColumnType, NodeSchema, RelSchema};

    fn setup_graph_with_rel() -> Graph {
        let g = Graph::in_memory().unwrap();
        let person = NodeSchema::new("id", ColumnType::Serial).column("name", ColumnType::String);
        g.create_node_table("Person", &person).unwrap();
        let knows = RelSchema::new().column("since", ColumnType::Int64);
        g.create_rel_table("KNOWS", "Person", "Person", &knows)
            .unwrap();
        g
    }

    #[test]
    #[serial_test::serial]
    fn create_edge_succeeds() {
        let g = setup_graph_with_rel();
        let a = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let b = g
            .create_node("Person", props([("name", Property::from("Bob"))]))
            .unwrap();
        g.create_edge("KNOWS", a, b, props([("since", Property::from(2020i64))]))
            .unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn create_edge_empty_table_errors() {
        let g = Graph::in_memory().unwrap();
        let result = g.create_edge("", NodeId::new(0, 0), NodeId::new(0, 1), PropertyMap::new());
        assert!(result.is_err());
    }

    #[test]
    #[serial_test::serial]
    fn create_edge_no_props() {
        let g = Graph::in_memory().unwrap();
        let person = NodeSchema::new("id", ColumnType::Serial).column("name", ColumnType::String);
        g.create_node_table("Person", &person).unwrap();
        let knows = RelSchema::new();
        g.create_rel_table("KNOWS", "Person", "Person", &knows)
            .unwrap();

        let a = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let b = g
            .create_node("Person", props([("name", Property::from("Bob"))]))
            .unwrap();
        g.create_edge("KNOWS", a, b, PropertyMap::new()).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn delete_edge_succeeds() {
        let g = setup_graph_with_rel();
        let a = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let b = g
            .create_node("Person", props([("name", Property::from("Bob"))]))
            .unwrap();
        g.create_edge("KNOWS", a, b, props([("since", Property::from(2020i64))]))
            .unwrap();
        g.delete_edge("KNOWS", a, b).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn count_edges_zero() {
        let g = setup_graph_with_rel();
        let a = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let count = g.count_edges(a, "KNOWS").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    #[serial_test::serial]
    fn count_edges_after_create() {
        let g = setup_graph_with_rel();
        let a = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let b = g
            .create_node("Person", props([("name", Property::from("Bob"))]))
            .unwrap();
        g.create_edge("KNOWS", a, b, props([("since", Property::from(2020i64))]))
            .unwrap();
        let count = g.count_edges(a, "KNOWS").unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    #[serial_test::serial]
    fn count_edges_multiple() {
        let g = setup_graph_with_rel();
        let a = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let b = g
            .create_node("Person", props([("name", Property::from("Bob"))]))
            .unwrap();
        let c = g
            .create_node("Person", props([("name", Property::from("Charlie"))]))
            .unwrap();
        g.create_edge("KNOWS", a, b, props([("since", Property::from(2020i64))]))
            .unwrap();
        g.create_edge("KNOWS", a, c, props([("since", Property::from(2021i64))]))
            .unwrap();
        let count = g.count_edges(a, "KNOWS").unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    #[serial_test::serial]
    fn get_edges_returns_results() {
        let g = setup_graph_with_rel();
        let a = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let b = g
            .create_node("Person", props([("name", Property::from("Bob"))]))
            .unwrap();
        g.create_edge("KNOWS", a, b, props([("since", Property::from(2020i64))]))
            .unwrap();
        let edges = g.get_edges(a, "KNOWS").unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].label, "KNOWS");
    }

    #[test]
    #[serial_test::serial]
    fn get_edges_empty() {
        let g = setup_graph_with_rel();
        let a = g
            .create_node("Person", props([("name", Property::from("Alice"))]))
            .unwrap();
        let edges = g.get_edges(a, "KNOWS").unwrap();
        assert!(edges.is_empty());
    }
}
