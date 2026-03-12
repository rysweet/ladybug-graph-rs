use crate::node::NodeId;
use crate::property::{Property, PropertyMap};

/// Direction of edge traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Outgoing,
    Incoming,
    Both,
}

/// A graph edge connecting two nodes.
#[derive(Debug, Clone)]
pub struct Edge {
    pub src: NodeId,
    pub dst: NodeId,
    pub label: String,
    pub properties: PropertyMap,
}

impl Edge {
    /// Get a property by name.
    pub fn get(&self, key: &str) -> Option<&Property> {
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
    use std::collections::HashMap;

    fn make_edge() -> Edge {
        let mut props = HashMap::new();
        props.insert("since".to_string(), Property::Int64(2020));
        props.insert("weight".to_string(), Property::Double(0.8));
        Edge {
            src: NodeId::new(0, 1),
            dst: NodeId::new(0, 2),
            label: "KNOWS".into(),
            properties: props,
        }
    }

    #[test]
    fn edge_src_dst() {
        let e = make_edge();
        assert_eq!(e.src, NodeId::new(0, 1));
        assert_eq!(e.dst, NodeId::new(0, 2));
    }

    #[test]
    fn edge_label() {
        let e = make_edge();
        assert_eq!(e.label, "KNOWS");
    }

    #[test]
    fn edge_get_property() {
        let e = make_edge();
        assert_eq!(e.get("since").unwrap().as_i64(), Some(2020));
        assert!(e.get("missing").is_none());
    }

    #[test]
    fn edge_get_str_returns_none_for_non_string() {
        let e = make_edge();
        assert_eq!(e.get_str("since"), None);
    }

    #[test]
    fn edge_get_i64() {
        let e = make_edge();
        assert_eq!(e.get_i64("since"), Some(2020));
        assert_eq!(e.get_i64("missing"), None);
    }

    #[test]
    fn direction_equality() {
        assert_eq!(Direction::Outgoing, Direction::Outgoing);
        assert_ne!(Direction::Outgoing, Direction::Incoming);
        assert_ne!(Direction::Incoming, Direction::Both);
    }

    #[test]
    fn direction_copy() {
        let d = Direction::Both;
        let d2 = d;
        assert_eq!(d, d2);
    }

    #[test]
    fn edge_clone() {
        let e = make_edge();
        let e2 = e.clone();
        assert_eq!(e2.label, "KNOWS");
        assert_eq!(e2.src, e.src);
    }
}
