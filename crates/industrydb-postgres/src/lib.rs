//! PostgreSQL connector implementation for IndustryDB

mod connector;
mod operations;

pub use connector::PostgresConnector;

// Re-export for convenience
pub use industrydb_core::traits::{CrudOperations, DatabaseConnector};
