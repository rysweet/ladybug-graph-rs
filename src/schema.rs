use crate::error::{Error, Result};

/// A column type for schema definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    String,
    Int64,
    Int32,
    Int16,
    Double,
    Float,
    Bool,
    Date,
    Timestamp,
    Blob,
    Uuid,
    Serial,
}

impl ColumnType {
    /// Returns the Cypher type string.
    pub fn as_cypher(&self) -> &'static str {
        match self {
            ColumnType::String => "STRING",
            ColumnType::Int64 => "INT64",
            ColumnType::Int32 => "INT32",
            ColumnType::Int16 => "INT16",
            ColumnType::Double => "DOUBLE",
            ColumnType::Float => "FLOAT",
            ColumnType::Bool => "BOOL",
            ColumnType::Date => "DATE",
            ColumnType::Timestamp => "TIMESTAMP",
            ColumnType::Blob => "BLOB",
            ColumnType::Uuid => "UUID",
            ColumnType::Serial => "SERIAL",
        }
    }
}

/// A column definition: name + type.
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub col_type: ColumnType,
}

/// Schema definition for a node table.
#[derive(Debug, Clone)]
pub struct NodeSchema {
    pub primary_key: String,
    pub primary_key_type: ColumnType,
    pub columns: Vec<Column>,
}

impl NodeSchema {
    /// Create a new node schema with the given primary key.
    pub fn new(pk_name: &str, pk_type: ColumnType) -> Self {
        Self {
            primary_key: pk_name.to_string(),
            primary_key_type: pk_type,
            columns: Vec::new(),
        }
    }

    /// Add a column to the schema.
    pub fn column(mut self, name: &str, col_type: ColumnType) -> Self {
        self.columns.push(Column {
            name: name.to_string(),
            col_type,
        });
        self
    }

    /// Generate the CREATE NODE TABLE Cypher statement.
    pub fn to_cypher(&self, table_name: &str) -> Result<String> {
        if table_name.is_empty() {
            return Err(Error::schema("table name cannot be empty"));
        }
        let mut parts = vec![format!(
            "{} {} PRIMARY KEY",
            self.primary_key,
            self.primary_key_type.as_cypher()
        )];
        for col in &self.columns {
            parts.push(format!("{} {}", col.name, col.col_type.as_cypher()));
        }
        Ok(format!(
            "CREATE NODE TABLE {} ({})",
            table_name,
            parts.join(", ")
        ))
    }
}

/// Schema definition for a relationship table.
#[derive(Debug, Clone)]
pub struct RelSchema {
    pub columns: Vec<Column>,
}

impl RelSchema {
    /// Create an empty relationship schema.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    /// Add a column to the schema.
    pub fn column(mut self, name: &str, col_type: ColumnType) -> Self {
        self.columns.push(Column {
            name: name.to_string(),
            col_type,
        });
        self
    }

    /// Generate the CREATE REL TABLE Cypher statement.
    pub fn to_cypher(&self, table_name: &str, from: &str, to: &str) -> Result<String> {
        if table_name.is_empty() {
            return Err(Error::schema("table name cannot be empty"));
        }
        if from.is_empty() || to.is_empty() {
            return Err(Error::schema("from/to table names cannot be empty"));
        }
        let col_defs: Vec<String> = self
            .columns
            .iter()
            .map(|c| format!("{} {}", c.name, c.col_type.as_cypher()))
            .collect();

        let cols = if col_defs.is_empty() {
            String::new()
        } else {
            format!(", {}", col_defs.join(", "))
        };

        Ok(format!(
            "CREATE REL TABLE {} (FROM {} TO {}{})",
            table_name, from, to, cols
        ))
    }
}

impl Default for RelSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_type_as_cypher() {
        assert_eq!(ColumnType::String.as_cypher(), "STRING");
        assert_eq!(ColumnType::Int64.as_cypher(), "INT64");
        assert_eq!(ColumnType::Double.as_cypher(), "DOUBLE");
        assert_eq!(ColumnType::Bool.as_cypher(), "BOOL");
        assert_eq!(ColumnType::Date.as_cypher(), "DATE");
        assert_eq!(ColumnType::Timestamp.as_cypher(), "TIMESTAMP");
        assert_eq!(ColumnType::Blob.as_cypher(), "BLOB");
        assert_eq!(ColumnType::Uuid.as_cypher(), "UUID");
        assert_eq!(ColumnType::Serial.as_cypher(), "SERIAL");
        assert_eq!(ColumnType::Int32.as_cypher(), "INT32");
        assert_eq!(ColumnType::Int16.as_cypher(), "INT16");
        assert_eq!(ColumnType::Float.as_cypher(), "FLOAT");
    }

    #[test]
    fn node_schema_basic() {
        let schema = NodeSchema::new("id", ColumnType::Serial);
        let cypher = schema.to_cypher("Person").unwrap();
        assert_eq!(cypher, "CREATE NODE TABLE Person (id SERIAL PRIMARY KEY)");
    }

    #[test]
    fn node_schema_with_columns() {
        let schema = NodeSchema::new("id", ColumnType::Serial)
            .column("name", ColumnType::String)
            .column("age", ColumnType::Int64);
        let cypher = schema.to_cypher("Person").unwrap();
        assert_eq!(
            cypher,
            "CREATE NODE TABLE Person (id SERIAL PRIMARY KEY, name STRING, age INT64)"
        );
    }

    #[test]
    fn node_schema_empty_table_name_errors() {
        let schema = NodeSchema::new("id", ColumnType::Serial);
        assert!(schema.to_cypher("").is_err());
    }

    #[test]
    fn rel_schema_no_columns() {
        let schema = RelSchema::new();
        let cypher = schema.to_cypher("KNOWS", "Person", "Person").unwrap();
        assert_eq!(cypher, "CREATE REL TABLE KNOWS (FROM Person TO Person)");
    }

    #[test]
    fn rel_schema_with_columns() {
        let schema = RelSchema::new()
            .column("since", ColumnType::Int64)
            .column("weight", ColumnType::Double);
        let cypher = schema.to_cypher("KNOWS", "Person", "Person").unwrap();
        assert_eq!(
            cypher,
            "CREATE REL TABLE KNOWS (FROM Person TO Person, since INT64, weight DOUBLE)"
        );
    }

    #[test]
    fn rel_schema_empty_table_name_errors() {
        let schema = RelSchema::new();
        assert!(schema.to_cypher("", "A", "B").is_err());
    }

    #[test]
    fn rel_schema_empty_from_errors() {
        let schema = RelSchema::new();
        assert!(schema.to_cypher("KNOWS", "", "B").is_err());
    }

    #[test]
    fn rel_schema_empty_to_errors() {
        let schema = RelSchema::new();
        assert!(schema.to_cypher("KNOWS", "A", "").is_err());
    }

    #[test]
    fn rel_schema_default() {
        let schema = RelSchema::default();
        assert!(schema.columns.is_empty());
    }

    #[test]
    fn node_schema_clone() {
        let schema = NodeSchema::new("id", ColumnType::Serial).column("name", ColumnType::String);
        let schema2 = schema.clone();
        assert_eq!(schema2.primary_key, "id");
        assert_eq!(schema2.columns.len(), 1);
    }

    #[test]
    fn column_type_clone_eq() {
        let t = ColumnType::Int64;
        let t2 = t.clone();
        assert_eq!(t, t2);
    }
}
