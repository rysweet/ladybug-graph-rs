use std::fmt;

/// Unique identifier for a node in the graph.
///
/// Wraps the underlying table_id + offset pair from lbug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId {
    pub table_id: u64,
    pub offset: u64,
}

impl NodeId {
    /// Create a new NodeId from table_id and offset.
    pub fn new(table_id: u64, offset: u64) -> Self {
        Self { table_id, offset }
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.table_id, self.offset)
    }
}

impl From<lbug::InternalID> for NodeId {
    fn from(id: lbug::InternalID) -> Self {
        Self {
            table_id: id.table_id,
            offset: id.offset,
        }
    }
}

impl From<NodeId> for lbug::InternalID {
    fn from(id: NodeId) -> Self {
        lbug::InternalID {
            table_id: id.table_id,
            offset: id.offset,
        }
    }
}

use crate::property::PropertyMap;

/// A graph node with its label, id, and properties.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub properties: PropertyMap,
}

impl Node {
    /// Get a property by name.
    pub fn get(&self, key: &str) -> Option<&crate::property::Property> {
        self.properties.get(key)
    }

    /// Get a property value as a string.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.properties.get(key).and_then(|p| p.as_str())
    }

    /// Get a property value as an i64.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.properties.get(key).and_then(|p| p.as_i64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::Property;
    use std::collections::HashMap;

    #[test]
    fn node_id_new() {
        let id = NodeId::new(1, 42);
        assert_eq!(id.table_id, 1);
        assert_eq!(id.offset, 42);
    }

    #[test]
    fn node_id_display() {
        let id = NodeId::new(3, 7);
        assert_eq!(format!("{id}"), "3:7");
    }

    #[test]
    fn node_id_equality() {
        let a = NodeId::new(1, 2);
        let b = NodeId::new(1, 2);
        let c = NodeId::new(1, 3);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn node_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NodeId::new(0, 0));
        set.insert(NodeId::new(0, 0));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn node_id_copy() {
        let a = NodeId::new(1, 2);
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn node_id_roundtrip_lbug() {
        let original = NodeId::new(5, 10);
        let lbug_id: lbug::InternalID = original.into();
        let back: NodeId = lbug_id.into();
        assert_eq!(original, back);
    }

    #[test]
    fn node_get_property() {
        let mut props = HashMap::new();
        props.insert("name".to_string(), Property::String("Alice".into()));
        let node = Node {
            id: NodeId::new(0, 0),
            label: "Person".into(),
            properties: props,
        };
        assert_eq!(node.get("name").unwrap().as_str(), Some("Alice"));
        assert!(node.get("missing").is_none());
    }

    #[test]
    fn node_get_str() {
        let mut props = HashMap::new();
        props.insert("city".to_string(), Property::String("NYC".into()));
        let node = Node {
            id: NodeId::new(0, 0),
            label: "Place".into(),
            properties: props,
        };
        assert_eq!(node.get_str("city"), Some("NYC"));
        assert_eq!(node.get_str("missing"), None);
    }

    #[test]
    fn node_get_i64() {
        let mut props = HashMap::new();
        props.insert("age".to_string(), Property::Int64(30));
        let node = Node {
            id: NodeId::new(0, 0),
            label: "Person".into(),
            properties: props,
        };
        assert_eq!(node.get_i64("age"), Some(30));
        assert_eq!(node.get_i64("missing"), None);
    }
}
