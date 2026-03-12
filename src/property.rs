use std::collections::HashMap;
use std::fmt;

/// A typed property value stored in graph nodes and edges.
#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    Null,
    Bool(bool),
    Int64(i64),
    Double(f64),
    String(String),
    List(Vec<Property>),
    Map(HashMap<String, Property>),
}

impl Property {
    /// Returns `true` if this property is `Null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Property::Null)
    }

    /// Try to extract a bool value.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Property::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract an i64 value.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Property::Int64(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract an f64 value.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Property::Double(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to extract a string reference.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Property::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    /// Try to extract a list reference.
    pub fn as_list(&self) -> Option<&[Property]> {
        match self {
            Property::List(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    /// Try to extract a map reference.
    pub fn as_map(&self) -> Option<&HashMap<String, Property>> {
        match self {
            Property::Map(v) => Some(v),
            _ => None,
        }
    }
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Property::Null => write!(f, "NULL"),
            Property::Bool(v) => write!(f, "{v}"),
            Property::Int64(v) => write!(f, "{v}"),
            Property::Double(v) => write!(f, "{v}"),
            Property::String(v) => write!(f, "{v}"),
            Property::List(v) => write!(f, "{v:?}"),
            Property::Map(v) => write!(f, "{v:?}"),
        }
    }
}

// Convenience From impls
impl From<bool> for Property {
    fn from(v: bool) -> Self {
        Property::Bool(v)
    }
}

impl From<i64> for Property {
    fn from(v: i64) -> Self {
        Property::Int64(v)
    }
}

impl From<i32> for Property {
    fn from(v: i32) -> Self {
        Property::Int64(v as i64)
    }
}

impl From<f64> for Property {
    fn from(v: f64) -> Self {
        Property::Double(v)
    }
}

impl From<&str> for Property {
    fn from(v: &str) -> Self {
        Property::String(v.to_string())
    }
}

impl From<String> for Property {
    fn from(v: String) -> Self {
        Property::String(v)
    }
}

impl<T: Into<Property>> From<Vec<T>> for Property {
    fn from(v: Vec<T>) -> Self {
        Property::List(v.into_iter().map(Into::into).collect())
    }
}

/// A map of property names to values.
pub type PropertyMap = HashMap<String, Property>;

/// Helper to build a `PropertyMap` from key-value pairs.
pub fn props<I, K, V>(pairs: I) -> PropertyMap
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<Property>,
{
    pairs
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_is_null() {
        assert!(Property::Null.is_null());
        assert!(!Property::Bool(true).is_null());
    }

    #[test]
    fn as_bool_extracts() {
        assert_eq!(Property::Bool(true).as_bool(), Some(true));
        assert_eq!(Property::Int64(1).as_bool(), None);
    }

    #[test]
    fn as_i64_extracts() {
        assert_eq!(Property::Int64(42).as_i64(), Some(42));
        assert_eq!(Property::Bool(true).as_i64(), None);
    }

    #[test]
    fn as_f64_extracts() {
        assert_eq!(Property::Double(2.72).as_f64(), Some(2.72));
        assert_eq!(Property::Int64(1).as_f64(), None);
    }

    #[test]
    fn as_str_extracts() {
        let p = Property::String("hello".into());
        assert_eq!(p.as_str(), Some("hello"));
        assert_eq!(Property::Null.as_str(), None);
    }

    #[test]
    fn as_list_extracts() {
        let p = Property::List(vec![Property::Int64(1), Property::Int64(2)]);
        assert_eq!(p.as_list().unwrap().len(), 2);
        assert_eq!(Property::Null.as_list(), None);
    }

    #[test]
    fn as_map_extracts() {
        let mut m = HashMap::new();
        m.insert("key".to_string(), Property::Bool(true));
        let p = Property::Map(m);
        assert!(p.as_map().is_some());
        assert_eq!(Property::Null.as_map(), None);
    }

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", Property::Null), "NULL");
    }

    #[test]
    fn display_bool() {
        assert_eq!(format!("{}", Property::Bool(true)), "true");
    }

    #[test]
    fn display_int64() {
        assert_eq!(format!("{}", Property::Int64(99)), "99");
    }

    #[test]
    fn display_string() {
        assert_eq!(format!("{}", Property::String("hi".into())), "hi");
    }

    #[test]
    fn from_bool() {
        assert_eq!(Property::from(true), Property::Bool(true));
    }

    #[test]
    fn from_i64() {
        assert_eq!(Property::from(42i64), Property::Int64(42));
    }

    #[test]
    fn from_i32() {
        assert_eq!(Property::from(10i32), Property::Int64(10));
    }

    #[test]
    fn from_f64() {
        assert_eq!(Property::from(1.5f64), Property::Double(1.5));
    }

    #[test]
    fn from_str() {
        assert_eq!(Property::from("test"), Property::String("test".into()));
    }

    #[test]
    fn from_string() {
        assert_eq!(
            Property::from("owned".to_string()),
            Property::String("owned".into())
        );
    }

    #[test]
    fn from_vec() {
        let p: Property = vec![1i64, 2, 3].into();
        assert_eq!(
            p,
            Property::List(vec![
                Property::Int64(1),
                Property::Int64(2),
                Property::Int64(3)
            ])
        );
    }

    #[test]
    fn props_helper_builds_map() {
        let m = props([
            ("name", Property::from("Alice")),
            ("age", Property::from(30i64)),
        ]);
        assert_eq!(m.get("name").unwrap().as_str(), Some("Alice"));
        assert_eq!(m.get("age").unwrap().as_i64(), Some(30));
    }

    #[test]
    fn props_helper_empty() {
        let m = props(std::iter::empty::<(&str, Property)>());
        assert!(m.is_empty());
    }

    #[test]
    fn property_clone() {
        let p = Property::String("clone me".into());
        let p2 = p.clone();
        assert_eq!(p, p2);
    }
}
