//! SQLite connector implementation using sqlx with connection pooling

use async_trait::async_trait;
use industrydb_core::{
    config::ConnectionConfig,
    error::{IndustryDbError, Result},
    traits::DatabaseConnector,
};
use polars::prelude::*;
use sqlx::{sqlite::SqliteRow, Column as SqlxColumn, Row, SqlitePool};

/// SQLite database connector with connection pool
pub struct SqliteConnector {
    pool: SqlitePool,
    db_type: String,
}

impl SqliteConnector {
    /// Create a new SQLite connector with connection pool
    pub async fn new(config: &ConnectionConfig) -> Result<Self> {
        let database_url = format!(
            "sqlite://{}",
            config.database.as_deref().unwrap_or(":memory:")
        );

        let pool = SqlitePool::connect(&database_url)
            .await
            .map_err(|e| IndustryDbError::ConnectionError(e.to_string()))?;

        Ok(Self {
            pool,
            db_type: "sqlite".to_string(),
        })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl DatabaseConnector for SqliteConnector {
    fn db_type(&self) -> &str {
        &self.db_type
    }

    async fn execute(&self, sql: &str) -> Result<DataFrame> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| IndustryDbError::QueryError(e.to_string()))?;

        if rows.is_empty() {
            return Ok(DataFrame::empty());
        }

        rows_to_dataframe(rows)
    }

    async fn is_alive(&self) -> bool {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await.is_ok()
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    fn is_closed(&self) -> bool {
        self.pool.is_closed()
    }
}

fn rows_to_dataframe(rows: Vec<SqliteRow>) -> Result<DataFrame> {
    if rows.is_empty() {
        return Ok(DataFrame::empty());
    }

    let columns = rows[0].columns();
    let mut series_vec: Vec<Series> = Vec::new();

    for column in columns {
        let col_name = column.name();

        // SQLite is dynamically typed, try different types
        let series = if let Ok(values) = rows
            .iter()
            .map(|row| row.try_get::<Option<i64>, _>(col_name))
            .collect::<sqlx::Result<Vec<_>>>()
        {
            Series::new(col_name.into(), values)
        } else if let Ok(values) = rows
            .iter()
            .map(|row| row.try_get::<Option<f64>, _>(col_name))
            .collect::<sqlx::Result<Vec<_>>>()
        {
            Series::new(col_name.into(), values)
        } else if let Ok(values) = rows
            .iter()
            .map(|row| row.try_get::<Option<String>, _>(col_name))
            .collect::<sqlx::Result<Vec<_>>>()
        {
            Series::new(col_name.into(), values)
        } else {
            // Fallback to string
            let values: Vec<Option<String>> =
                rows.iter().map(|row| row.try_get(col_name).ok()).collect();
            Series::new(col_name.into(), values)
        };

        series_vec.push(series);
    }

    let columns: Vec<_> = series_vec.into_iter().map(|s| s.into_column()).collect();
    DataFrame::new(columns).map_err(|e| IndustryDbError::PolarsError(e.to_string()))
}
