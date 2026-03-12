use crate::property::Property;

/// A named parameter for parameterized Cypher queries.
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub value: Property,
}

impl Param {
    /// Create a new parameter.
    pub fn new(name: impl Into<String>, value: impl Into<Property>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Convenience function to create a list of parameters.
pub fn params<I, N, V>(pairs: I) -> Vec<Param>
where
    I: IntoIterator<Item = (N, V)>,
    N: Into<String>,
    V: Into<Property>,
{
    pairs.into_iter().map(|(n, v)| Param::new(n, v)).collect()
}

/// Convert a Property value to a Cypher literal string for inline queries.
pub fn property_to_cypher_literal(prop: &Property) -> String {
    match prop {
        Property::Null => "NULL".to_string(),
        Property::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Property::Int64(n) => n.to_string(),
        Property::Double(f) => format!("{f}"),
        Property::String(s) => super::escape::escape_string(s),
        Property::List(items) => {
            let inner: Vec<String> = items.iter().map(property_to_cypher_literal).collect();
            format!("[{}]", inner.join(", "))
        }
        Property::Map(map) => {
            let entries: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, property_to_cypher_literal(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::Property;

    #[test]
    fn param_new() {
        let p = Param::new("name", "Alice");
        assert_eq!(p.name, "name");
        assert_eq!(p.value, Property::String("Alice".into()));
    }

    #[test]
    fn param_new_with_int() {
        let p = Param::new("age", 30i64);
        assert_eq!(p.name, "age");
        assert_eq!(p.value, Property::Int64(30));
    }

    #[test]
    fn params_helper() {
        let ps = params([
            ("name", Property::from("Bob")),
            ("age", Property::from(25i64)),
        ]);
        assert_eq!(ps.len(), 2);
        assert_eq!(ps[0].name, "name");
        assert_eq!(ps[1].name, "age");
    }

    #[test]
    fn params_empty() {
        let ps = params(std::iter::empty::<(&str, Property)>());
        assert!(ps.is_empty());
    }

    #[test]
    fn literal_null() {
        assert_eq!(property_to_cypher_literal(&Property::Null), "NULL");
    }

    #[test]
    fn literal_bool_true() {
        assert_eq!(property_to_cypher_literal(&Property::Bool(true)), "true");
    }

    #[test]
    fn literal_bool_false() {
        assert_eq!(property_to_cypher_literal(&Property::Bool(false)), "false");
    }

    #[test]
    fn literal_int64() {
        assert_eq!(property_to_cypher_literal(&Property::Int64(42)), "42");
    }

    #[test]
    fn literal_double() {
        let s = property_to_cypher_literal(&Property::Double(3.14));
        assert!(s.starts_with("3.14"));
    }

    #[test]
    fn literal_string() {
        assert_eq!(
            property_to_cypher_literal(&Property::String("hello".into())),
            "'hello'"
        );
    }

    #[test]
    fn literal_string_with_quote() {
        assert_eq!(
            property_to_cypher_literal(&Property::String("it's".into())),
            "'it\\'s'"
        );
    }

    #[test]
    fn literal_list() {
        let p = Property::List(vec![Property::Int64(1), Property::Int64(2)]);
        assert_eq!(property_to_cypher_literal(&p), "[1, 2]");
    }

    #[test]
    fn literal_empty_list() {
        assert_eq!(property_to_cypher_literal(&Property::List(vec![])), "[]");
    }

    #[test]
    fn param_clone() {
        let p = Param::new("x", 5i64);
        let p2 = p.clone();
        assert_eq!(p2.name, "x");
    }
}
