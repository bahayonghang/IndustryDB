//! SQLite connector implementation for IndustryDB

mod connector;
mod operations;

pub use connector::SqliteConnector;
pub use industrydb_core::traits::{CrudOperations, DatabaseConnector};
