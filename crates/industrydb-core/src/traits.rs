//! Core traits for database connectors

use async_trait::async_trait;
use polars::prelude::*;
use std::collections::HashMap;

use crate::error::Result;

/// Core trait that all database connectors must implement
#[async_trait]
pub trait DatabaseConnector: Send + Sync {
    /// Get the database type name
    fn db_type(&self) -> &str;

    /// Execute a raw SQL query and return a DataFrame
    async fn execute(&self, sql: &str) -> Result<DataFrame>;

    /// Check if the connection is alive
    async fn is_alive(&self) -> bool;

    /// Close the connection
    async fn close(&mut self) -> Result<()>;

    /// Check if connection is closed
    fn is_closed(&self) -> bool;
}

/// CRUD operations trait
#[async_trait]
pub trait CrudOperations: DatabaseConnector {
    /// Insert data into a table
    async fn insert(&self, table: &str, data: DataFrame) -> Result<usize>;

    /// Select data from a table
    async fn select(
        &self,
        table: &str,
        columns: Option<&[String]>,
        where_clause: Option<&str>,
        limit: Option<usize>,
    ) -> Result<DataFrame>;

    /// Update rows in a table
    async fn update(
        &self,
        table: &str,
        values: &HashMap<String, String>,
        where_clause: Option<&str>,
    ) -> Result<usize>;

    /// Delete rows from a table
    async fn delete(&self, table: &str, where_clause: Option<&str>) -> Result<usize>;
}

/// Result of an operation
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// Number of rows affected
    pub rows_affected: usize,
    /// Success status
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
}

impl OperationResult {
    /// Create a success result
    pub fn success(rows_affected: usize) -> Self {
        Self {
            rows_affected,
            success: true,
            message: None,
        }
    }

    /// Create a failure result
    pub fn failure(message: String) -> Self {
        Self {
            rows_affected: 0,
            success: false,
            message: Some(message),
        }
    }
}
