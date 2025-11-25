//! MSSQL connector implementation for IndustryDB

mod connector;
pub mod domain;
mod operations;

pub use connector::MssqlConnector;
pub use industrydb_core::traits::{CrudOperations, DatabaseConnector};
