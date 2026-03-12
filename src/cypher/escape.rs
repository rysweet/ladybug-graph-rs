/// Escape a string value for use in Cypher literals.
///
/// Wraps in single quotes and escapes internal quotes and backslashes.
pub fn escape_string(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
    format!("'{}'", escaped)
}

/// Escape an identifier (table name, column name) for Cypher.
///
/// Wraps in backticks if the name contains special characters.
pub fn escape_identifier(name: &str) -> String {
    if name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.is_empty() {
        name.to_string()
    } else {
        format!("`{}`", name.replace('`', "``"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_simple_string() {
        assert_eq!(escape_string("hello"), "'hello'");
    }

    #[test]
    fn escape_string_with_quotes() {
        assert_eq!(escape_string("it's"), "'it\\'s'");
    }

    #[test]
    fn escape_string_with_backslash() {
        assert_eq!(escape_string("a\\b"), "'a\\\\b'");
    }

    #[test]
    fn escape_string_with_both() {
        assert_eq!(escape_string("a\\'b"), "'a\\\\\\'b'");
    }

    #[test]
    fn escape_empty_string() {
        assert_eq!(escape_string(""), "''");
    }

    #[test]
    fn escape_identifier_simple() {
        assert_eq!(escape_identifier("Person"), "Person");
    }

    #[test]
    fn escape_identifier_with_underscore() {
        assert_eq!(escape_identifier("my_table"), "my_table");
    }

    #[test]
    fn escape_identifier_with_space() {
        assert_eq!(escape_identifier("my table"), "`my table`");
    }

    #[test]
    fn escape_identifier_with_backtick() {
        assert_eq!(escape_identifier("my`table"), "`my``table`");
    }

    #[test]
    fn escape_identifier_empty() {
        assert_eq!(escape_identifier(""), "``");
    }

    #[test]
    fn escape_identifier_numeric() {
        assert_eq!(escape_identifier("123"), "123");
    }
}
