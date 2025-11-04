//! Error types for IndustryDB

use thiserror::Error;

/// Result type alias for IndustryDB operations
pub type Result<T> = std::result::Result<T, IndustryDbError>;

/// Main error type for IndustryDB
#[derive(Error, Debug)]
pub enum IndustryDbError {
    /// Database connection error
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    /// Query execution error
    #[error("Query execution error: {0}")]
    QueryError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// ConnectorX error
    #[error("ConnectorX error: {0}")]
    ConnectorXError(String),

    /// Polars error
    #[error("Polars error: {0}")]
    PolarsError(String),

    /// Unsupported database type
    #[error("Unsupported database type: {0}")]
    UnsupportedDatabase(String),

    /// Connection closed error
    #[error("Connection is closed")]
    ConnectionClosed,

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Constraint violation error
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Invalid parameter error
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Not implemented error
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

// Convert from polars errors
impl From<polars::prelude::PolarsError> for IndustryDbError {
    fn from(err: polars::prelude::PolarsError) -> Self {
        IndustryDbError::PolarsError(err.to_string())
    }
}

// Convert from connectorx errors
// Conversion from ConnectorXError is implemented in database-specific crates to avoid
// coupling core to optional dependencies.

// Implement custom Display if needed for special formatting
impl IndustryDbError {
    /// Create a connection error
    pub fn connection_error<S: Into<String>>(msg: S) -> Self {
        IndustryDbError::ConnectionError(msg.into())
    }

    /// Create a query error
    pub fn query_error<S: Into<String>>(msg: S) -> Self {
        IndustryDbError::QueryError(msg.into())
    }

    /// Create a config error
    pub fn config_error<S: Into<String>>(msg: S) -> Self {
        IndustryDbError::ConfigError(msg.into())
    }

    /// Create a constraint violation error
    pub fn constraint_violation<S: Into<String>>(msg: S) -> Self {
        IndustryDbError::ConstraintViolation(msg.into())
    }

    /// Create an invalid parameter error
    pub fn invalid_parameter<S: Into<String>>(msg: S) -> Self {
        IndustryDbError::InvalidParameter(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = IndustryDbError::connection_error("Test connection error");
        assert!(matches!(err, IndustryDbError::ConnectionError(_)));
        assert_eq!(
            err.to_string(),
            "Database connection error: Test connection error"
        );
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let err: IndustryDbError = io_err.into();
        assert!(matches!(err, IndustryDbError::IoError(_)));
    }
}
