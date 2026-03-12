use std::fmt;

/// All errors produced by ladybug-graph-rs.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error originating from the underlying lbug FFI layer.
    #[error("database error: {0}")]
    Database(String),

    /// A query returned unexpected or malformed results.
    #[error("query error: {0}")]
    Query(String),

    /// A schema definition error (e.g. duplicate column, invalid type).
    #[error("schema error: {0}")]
    Schema(String),

    /// A type conversion error between Property and lbug::Value.
    #[error("conversion error: {0}")]
    Conversion(String),

    /// An invalid argument was passed to an API method.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

impl From<lbug::Error> for Error {
    fn from(e: lbug::Error) -> Self {
        Error::Database(e.to_string())
    }
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Printable wrapper so error chains are readable in logs.
impl Error {
    pub fn query(msg: impl fmt::Display) -> Self {
        Error::Query(msg.to_string())
    }

    pub fn schema(msg: impl fmt::Display) -> Self {
        Error::Schema(msg.to_string())
    }

    pub fn conversion(msg: impl fmt::Display) -> Self {
        Error::Conversion(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_database() {
        let e = Error::Database("connection failed".into());
        assert_eq!(e.to_string(), "database error: connection failed");
    }

    #[test]
    fn error_display_query() {
        let e = Error::query("syntax error at line 1");
        assert_eq!(e.to_string(), "query error: syntax error at line 1");
    }

    #[test]
    fn error_display_schema() {
        let e = Error::schema("duplicate column name");
        assert_eq!(e.to_string(), "schema error: duplicate column name");
    }

    #[test]
    fn error_display_conversion() {
        let e = Error::conversion("cannot convert Bool to Int64");
        assert_eq!(
            e.to_string(),
            "conversion error: cannot convert Bool to Int64"
        );
    }

    #[test]
    fn error_display_invalid_argument() {
        let e = Error::InvalidArgument("empty table name".into());
        assert_eq!(e.to_string(), "invalid argument: empty table name");
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn result_type_alias_works() {
        let ok: Result<i32> = Ok(42);
        assert!(ok.is_ok());

        let err: Result<i32> = Err(Error::query("fail"));
        assert!(err.is_err());
    }
}
