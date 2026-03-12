use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::node::NodeId;
use crate::property::Property;

/// Convert a `lbug::Value` to our `Property` type.
pub fn value_to_property(value: &lbug::Value) -> Property {
    match value {
        lbug::Value::Null(_) => Property::Null,
        lbug::Value::Bool(b) => Property::Bool(*b),
        lbug::Value::Int16(n) => Property::Int64(*n as i64),
        lbug::Value::Int32(n) => Property::Int64(*n as i64),
        lbug::Value::Int64(n) => Property::Int64(*n),
        lbug::Value::Float(f) => Property::Double(*f as f64),
        lbug::Value::Double(f) => Property::Double(*f),
        lbug::Value::String(s) => Property::String(s.clone()),
        lbug::Value::List(_, items) => {
            Property::List(items.iter().map(value_to_property).collect())
        }
        lbug::Value::Struct(fields) => {
            let map: HashMap<String, Property> = fields
                .iter()
                .map(|(k, v)| (k.clone(), value_to_property(v)))
                .collect();
            Property::Map(map)
        }
        lbug::Value::InternalID(id) => Property::String(format!("{}:{}", id.table_id, id.offset)),
        lbug::Value::Node(node) => {
            let mut map = HashMap::new();
            map.insert(
                "_label".to_string(),
                Property::String(node.get_label_name().clone()),
            );
            let id = node.get_node_id();
            map.insert(
                "_id".to_string(),
                Property::String(format!("{}:{}", id.table_id, id.offset)),
            );
            for (k, v) in node.get_properties() {
                map.insert(k.clone(), value_to_property(v));
            }
            Property::Map(map)
        }
        lbug::Value::Rel(rel) => {
            let mut map = HashMap::new();
            map.insert(
                "_label".to_string(),
                Property::String(rel.get_label_name().clone()),
            );
            let src = rel.get_src_node();
            let dst = rel.get_dst_node();
            map.insert(
                "_src".to_string(),
                Property::String(format!("{}:{}", src.table_id, src.offset)),
            );
            map.insert(
                "_dst".to_string(),
                Property::String(format!("{}:{}", dst.table_id, dst.offset)),
            );
            for (k, v) in rel.get_properties() {
                map.insert(k.clone(), value_to_property(v));
            }
            Property::Map(map)
        }
        _ => Property::String(format!("{:?}", value)),
    }
}

/// Infer the `lbug::LogicalType` from a `Property` variant.
fn infer_logical_type(prop: &Property) -> lbug::LogicalType {
    match prop {
        Property::Bool(_) => lbug::LogicalType::Bool,
        Property::Int64(_) => lbug::LogicalType::Int64,
        Property::Double(_) => lbug::LogicalType::Double,
        Property::String(_) => lbug::LogicalType::String,
        _ => lbug::LogicalType::String,
    }
}

/// Convert a `Property` to a `lbug::Value`.
pub fn property_to_value(prop: &Property) -> lbug::Value {
    match prop {
        Property::Null => lbug::Value::Null(lbug::LogicalType::Any),
        Property::Bool(b) => lbug::Value::Bool(*b),
        Property::Int64(n) => lbug::Value::Int64(*n),
        Property::Double(f) => lbug::Value::Double(*f),
        Property::String(s) => lbug::Value::String(s.clone()),
        Property::List(items) => {
            let values: Vec<lbug::Value> = items.iter().map(property_to_value).collect();
            let logical_type = items
                .first()
                .map(infer_logical_type)
                .unwrap_or(lbug::LogicalType::String);
            lbug::Value::List(logical_type, values)
        }
        Property::Map(_) => lbug::Value::String(format!("{:?}", prop)),
    }
}

/// Extract a NodeId from a lbug NodeVal.
pub fn node_val_to_id(node: &lbug::NodeVal) -> NodeId {
    let id = node.get_node_id();
    NodeId::new(id.table_id, id.offset)
}

/// Extract properties from a lbug NodeVal.
pub fn node_val_properties(node: &lbug::NodeVal) -> HashMap<String, Property> {
    node.get_properties()
        .iter()
        .map(|(k, v)| (k.clone(), value_to_property(v)))
        .collect()
}

/// Build a `crate::node::Node` from a `lbug::NodeVal`.
pub fn node_val_to_node(node: &lbug::NodeVal) -> crate::node::Node {
    crate::node::Node {
        id: node_val_to_id(node),
        label: node.get_label_name().clone(),
        properties: node_val_properties(node),
    }
}

/// Build a `crate::edge::Edge` from a `lbug::RelVal`.
pub fn rel_val_to_edge(rel: &lbug::RelVal) -> crate::edge::Edge {
    let props: HashMap<String, Property> = rel
        .get_properties()
        .iter()
        .map(|(k, v)| (k.clone(), value_to_property(v)))
        .collect();
    let src = rel.get_src_node();
    let dst = rel.get_dst_node();
    crate::edge::Edge {
        src: NodeId::new(src.table_id, src.offset),
        dst: NodeId::new(dst.table_id, dst.offset),
        label: rel.get_label_name().clone(),
        properties: props,
    }
}

/// Extract a NodeId from a single lbug::Value (InternalID or Node).
pub fn extract_node_id_from_value(value: &lbug::Value) -> Result<NodeId> {
    match value {
        lbug::Value::InternalID(id) => Ok(NodeId::new(id.table_id, id.offset)),
        lbug::Value::Node(node) => Ok(node_val_to_id(node)),
        other => Err(Error::conversion(format!(
            "expected InternalID or Node, got {:?}",
            other
        ))),
    }
}

/// Extract a Node from a single lbug::Value::Node.
pub fn extract_node_from_value(value: &lbug::Value) -> Result<Option<crate::node::Node>> {
    match value {
        lbug::Value::Node(node) => Ok(Some(node_val_to_node(node))),
        lbug::Value::Null(_) => Ok(None),
        other => Err(Error::conversion(format!(
            "expected Node value, got {:?}",
            other
        ))),
    }
}

/// Extract an Edge from a single lbug::Value::Rel.
pub fn extract_edge_from_value(value: &lbug::Value) -> Result<Option<crate::edge::Edge>> {
    match value {
        lbug::Value::Rel(rel) => Ok(Some(rel_val_to_edge(rel))),
        lbug::Value::Null(_) => Ok(None),
        other => Err(Error::conversion(format!(
            "expected Rel value, got {:?}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_null() {
        assert_eq!(
            value_to_property(&lbug::Value::Null(lbug::LogicalType::Any)),
            Property::Null
        );
    }

    #[test]
    fn convert_bool() {
        assert_eq!(
            value_to_property(&lbug::Value::Bool(true)),
            Property::Bool(true)
        );
    }

    #[test]
    fn convert_int64() {
        assert_eq!(
            value_to_property(&lbug::Value::Int64(42)),
            Property::Int64(42)
        );
    }

    #[test]
    fn convert_int32() {
        assert_eq!(
            value_to_property(&lbug::Value::Int32(10)),
            Property::Int64(10)
        );
    }

    #[test]
    fn convert_int16() {
        assert_eq!(
            value_to_property(&lbug::Value::Int16(5)),
            Property::Int64(5)
        );
    }

    #[test]
    fn convert_double() {
        assert_eq!(
            value_to_property(&lbug::Value::Double(2.72)),
            Property::Double(2.72)
        );
    }

    #[test]
    fn convert_float() {
        let p = value_to_property(&lbug::Value::Float(1.5));
        match p {
            Property::Double(v) => assert!((v - 1.5).abs() < 0.01),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn convert_string() {
        assert_eq!(
            value_to_property(&lbug::Value::String("hello".into())),
            Property::String("hello".into())
        );
    }

    #[test]
    fn convert_list() {
        let val = lbug::Value::List(
            lbug::LogicalType::Int64,
            vec![lbug::Value::Int64(1), lbug::Value::Int64(2)],
        );
        let prop = value_to_property(&val);
        assert_eq!(
            prop,
            Property::List(vec![Property::Int64(1), Property::Int64(2)])
        );
    }

    #[test]
    fn convert_internal_id() {
        let val = lbug::Value::InternalID(lbug::InternalID {
            table_id: 3,
            offset: 7,
        });
        assert_eq!(value_to_property(&val), Property::String("3:7".into()));
    }

    #[test]
    fn roundtrip_null() {
        let v = property_to_value(&Property::Null);
        assert!(matches!(v, lbug::Value::Null(_)));
    }

    #[test]
    fn roundtrip_bool() {
        let v = property_to_value(&Property::Bool(false));
        assert!(matches!(v, lbug::Value::Bool(false)));
    }

    #[test]
    fn roundtrip_int64() {
        let v = property_to_value(&Property::Int64(99));
        assert!(matches!(v, lbug::Value::Int64(99)));
    }

    #[test]
    fn roundtrip_double() {
        let v = property_to_value(&Property::Double(2.5));
        match v {
            lbug::Value::Double(f) => assert!((f - 2.5).abs() < f64::EPSILON),
            _ => panic!("expected Double"),
        }
    }

    #[test]
    fn roundtrip_string() {
        let v = property_to_value(&Property::String("test".into()));
        match v {
            lbug::Value::String(s) => assert_eq!(s, "test"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn extract_node_id_from_internal_id() {
        let val = lbug::Value::InternalID(lbug::InternalID {
            table_id: 1,
            offset: 5,
        });
        let id = extract_node_id_from_value(&val).unwrap();
        assert_eq!(id, NodeId::new(1, 5));
    }

    #[test]
    fn extract_node_id_wrong_type() {
        let val = lbug::Value::Bool(true);
        assert!(extract_node_id_from_value(&val).is_err());
    }
}
