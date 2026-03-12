use std::path::Path;

use lbug::{Connection, Database};

use crate::config::GraphConfig;
use crate::convert::value_to_property;
use crate::error::Result;
use crate::property::Property;
use crate::schema::{NodeSchema, RelSchema};

/// The main entry point for interacting with a LadybugDB graph database.
///
/// `Graph` owns the underlying database and creates short-lived connections
/// for each operation. Database is `Send + Sync` in lbug, so Graph is too.
pub struct Graph {
    pub(crate) db: Database,
}

impl Graph {
    /// Open or create a database at the given path with default config.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_config(path, GraphConfig::default())
    }

    /// Open or create a database at the given path with custom config.
    pub fn open_with_config(path: impl AsRef<Path>, config: GraphConfig) -> Result<Self> {
        let db = Database::new(path, config.to_system_config())?;
        Ok(Self { db })
    }

    /// Create an in-memory database with default config.
    pub fn in_memory() -> Result<Self> {
        Self::in_memory_with_config(GraphConfig::default())
    }

    /// Create an in-memory database with custom config.
    pub fn in_memory_with_config(config: GraphConfig) -> Result<Self> {
        let db = Database::in_memory(config.to_system_config())?;
        Ok(Self { db })
    }

    /// Create a new connection to the database.
    pub(crate) fn connection(&self) -> Result<Connection<'_>> {
        Ok(Connection::new(&self.db)?)
    }

    /// Create a node table from a schema definition.
    pub fn create_node_table(&self, name: &str, schema: &NodeSchema) -> Result<()> {
        let cypher = schema.to_cypher(name)?;
        self.execute_cypher(&cypher)
    }

    /// Create a relationship table from a schema definition.
    pub fn create_rel_table(
        &self,
        name: &str,
        from: &str,
        to: &str,
        schema: &RelSchema,
    ) -> Result<()> {
        let cypher = schema.to_cypher(name, from, to)?;
        self.execute_cypher(&cypher)
    }

    /// Drop a table by name.
    pub fn drop_table(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(crate::error::Error::InvalidArgument(
                "table name cannot be empty".into(),
            ));
        }
        self.execute_cypher(&format!("DROP TABLE {name}"))
    }

    /// Execute a raw Cypher query and return rows of Property values.
    pub fn query(&self, cypher: &str) -> Result<Vec<Vec<Property>>> {
        let conn = self.connection()?;
        let result = conn.query(cypher)?;
        let mut rows = Vec::new();
        for row in result {
            let props: Vec<Property> = row.iter().map(value_to_property).collect();
            rows.push(props);
        }
        Ok(rows)
    }

    /// Execute a raw Cypher query and return raw lbug Values.
    ///
    /// Used internally when we need NodeVal/RelVal access.
    pub(crate) fn query_raw(&self, cypher: &str) -> Result<Vec<Vec<lbug::Value>>> {
        let conn = self.connection()?;
        let result = conn.query(cypher)?;
        Ok(result.collect())
    }

    /// Execute a raw Cypher statement (no results expected).
    pub fn execute_cypher(&self, cypher: &str) -> Result<()> {
        let conn = self.connection()?;
        conn.query(cypher)?;
        Ok(())
    }

    /// Execute a Cypher query with parameter substitution.
    ///
    /// Parameters are substituted as `$name` in the query string.
    /// Uses inline literal substitution for maximum compatibility.
    pub fn execute(&self, cypher: &str, params: &[(&str, Property)]) -> Result<Vec<Vec<Property>>> {
        let mut query = cypher.to_string();
        for (name, value) in params {
            let literal = crate::cypher::params::property_to_cypher_literal(value);
            query = query.replace(&format!("${name}"), &literal);
        }
        self.query(&query)
    }
}

// Verify Send + Sync at compile time
#[cfg(test)]
fn _assert_send_sync() {
    fn check<T: Send + Sync>() {}
    check::<Graph>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn in_memory_creates_graph() {
        let g = Graph::in_memory().unwrap();
        let _ = g.query("RETURN 1").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn in_memory_with_config() {
        let cfg = GraphConfig::new().max_num_threads(2);
        let g = Graph::in_memory_with_config(cfg).unwrap();
        let _ = g.query("RETURN 1").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn open_with_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_db");
        let g = Graph::open(&path).unwrap();
        let _ = g.query("RETURN 1").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn open_with_config_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_db2");
        let cfg = GraphConfig::new().buffer_pool_size(256 * 1024 * 1024);
        let g = Graph::open_with_config(&path, cfg).unwrap();
        let _ = g.query("RETURN 1").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn query_return_literal() {
        let g = Graph::in_memory().unwrap();
        let rows = g.query("RETURN 42").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0][0].as_i64(), Some(42));
    }

    #[test]
    #[serial_test::serial]
    fn query_return_string() {
        let g = Graph::in_memory().unwrap();
        let rows = g.query("RETURN 'hello'").unwrap();
        assert_eq!(rows[0][0].as_str(), Some("hello"));
    }

    #[test]
    #[serial_test::serial]
    fn query_return_bool() {
        let g = Graph::in_memory().unwrap();
        let rows = g.query("RETURN true").unwrap();
        assert_eq!(rows[0][0].as_bool(), Some(true));
    }

    #[test]
    #[serial_test::serial]
    fn query_return_multiple_columns() {
        let g = Graph::in_memory().unwrap();
        let rows = g.query("RETURN 1, 'two', true").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 3);
    }

    #[test]
    #[serial_test::serial]
    fn execute_with_params() {
        let g = Graph::in_memory().unwrap();
        let rows = g
            .execute("RETURN $val", &[("val", Property::Int64(99))])
            .unwrap();
        assert_eq!(rows[0][0].as_i64(), Some(99));
    }

    #[test]
    #[serial_test::serial]
    fn execute_cypher_no_result() {
        let g = Graph::in_memory().unwrap();
        g.execute_cypher("RETURN 1").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn graph_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Graph>();
    }

    #[test]
    #[serial_test::serial]
    fn create_and_drop_node_table() {
        let g = Graph::in_memory().unwrap();
        let schema = crate::schema::NodeSchema::new("id", crate::schema::ColumnType::Serial)
            .column("name", crate::schema::ColumnType::String);
        g.create_node_table("TestNode", &schema).unwrap();
        g.drop_table("TestNode").unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn create_rel_table() {
        let g = Graph::in_memory().unwrap();
        let ns = crate::schema::NodeSchema::new("id", crate::schema::ColumnType::Serial);
        g.create_node_table("A", &ns).unwrap();
        g.create_node_table("B", &ns).unwrap();
        let rs = crate::schema::RelSchema::new();
        g.create_rel_table("LINKS", "A", "B", &rs).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn query_raw_returns_values() {
        let g = Graph::in_memory().unwrap();
        let rows = g.query_raw("RETURN 42").unwrap();
        assert_eq!(rows.len(), 1);
    }
}
