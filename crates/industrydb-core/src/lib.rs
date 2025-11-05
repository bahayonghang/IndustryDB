//! IndustryDB Core Library
//!
//! Core abstractions and traits for database connectivity.
//! This crate defines the interface that all database connectors must implement.

pub mod config;
pub mod error;
pub mod factory;
pub mod traits;

pub use config::{ConnectionConfig, DatabaseConfig, DatabaseType};
pub use error::{IndustryDbError, Result};
pub use factory::ConnectionFactory;
pub use traits::{CrudOperations, DatabaseConnector};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_format() {
        // VERSION should follow semantic versioning format (e.g., "0.1.0")
        let parts: Vec<&str> = VERSION.split('.').collect();
        assert!(
            parts.len() >= 2,
            "Version should have at least major.minor format"
        );
    }
}
