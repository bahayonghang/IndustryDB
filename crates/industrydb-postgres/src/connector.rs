//! PostgreSQL connector implementation using sqlx with connection pooling

use async_trait::async_trait;
use industrydb_core::{
    config::ConnectionConfig,
    error::{IndustryDbError, Result},
    traits::DatabaseConnector,
};
use polars::prelude::*;
use sqlx::{postgres::PgRow, Column as SqlxColumn, PgPool, Row, TypeInfo};

/// PostgreSQL database connector with connection pool
pub struct PostgresConnector {
    pool: PgPool,
    db_type: String,
}

impl PostgresConnector {
    /// Create a new PostgreSQL connector with connection pool
    pub async fn new(config: &ConnectionConfig) -> Result<Self> {
        let database_url = format!(
            "postgresql://{}:{}@{}:{}/{}",
            config.username.as_deref().unwrap_or("postgres"),
            config.password.as_deref().unwrap_or(""),
            config.host.as_deref().unwrap_or("localhost"),
            config.port.unwrap_or(5432),
            config.database.as_deref().unwrap_or("postgres")
        );

        let pool = PgPool::connect(&database_url)
            .await
            .map_err(|e| IndustryDbError::ConnectionError(e.to_string()))?;

        Ok(Self {
            pool,
            db_type: "postgres".to_string(),
        })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl DatabaseConnector for PostgresConnector {
    fn db_type(&self) -> &str {
        &self.db_type
    }

    async fn execute(&self, sql: &str) -> Result<DataFrame> {
        // Execute query and fetch all rows
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| IndustryDbError::QueryError(e.to_string()))?;

        if rows.is_empty() {
            return Ok(DataFrame::empty());
        }

        // Convert rows to Polars DataFrame
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

/// Convert PostgreSQL rows to Polars DataFrame
fn rows_to_dataframe(rows: Vec<PgRow>) -> Result<DataFrame> {
    if rows.is_empty() {
        return Ok(DataFrame::empty());
    }

    let columns = rows[0].columns();
    let mut series_vec: Vec<Series> = Vec::new();

    for column in columns {
        let col_name = column.name();
        let col_type = column.type_info();

        // Extract values based on type
        let series = match col_type.name() {
            "INT2" | "SMALLINT" => {
                let values: Vec<Option<i16>> =
                    rows.iter().map(|row| row.try_get(col_name).ok()).collect();
                Series::new(col_name.into(), values)
            }
            "INT4" | "INT" | "INTEGER" => {
                let values: Vec<Option<i32>> =
                    rows.iter().map(|row| row.try_get(col_name).ok()).collect();
                Series::new(col_name.into(), values)
            }
            "INT8" | "BIGINT" => {
                let values: Vec<Option<i64>> =
                    rows.iter().map(|row| row.try_get(col_name).ok()).collect();
                Series::new(col_name.into(), values)
            }
            "FLOAT4" | "REAL" => {
                let values: Vec<Option<f32>> =
                    rows.iter().map(|row| row.try_get(col_name).ok()).collect();
                Series::new(col_name.into(), values)
            }
            "FLOAT8" | "DOUBLE PRECISION" => {
                let values: Vec<Option<f64>> =
                    rows.iter().map(|row| row.try_get(col_name).ok()).collect();
                Series::new(col_name.into(), values)
            }
            "BOOL" | "BOOLEAN" => {
                let values: Vec<Option<bool>> =
                    rows.iter().map(|row| row.try_get(col_name).ok()).collect();
                Series::new(col_name.into(), values)
            }
            _ => {
                // Default to string for unsupported types
                let values: Vec<Option<String>> =
                    rows.iter().map(|row| row.try_get(col_name).ok()).collect();
                Series::new(col_name.into(), values)
            }
        };

        series_vec.push(series);
    }

    let columns: Vec<_> = series_vec.into_iter().map(|s| s.into_column()).collect();
    DataFrame::new(columns).map_err(|e| IndustryDbError::PolarsError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use industrydb_core::config::DatabaseType;

    #[tokio::test]
    async fn test_connector_creation() {
        let config = ConnectionConfig {
            db_type: DatabaseType::Postgres,
            host: Some("localhost".to_string()),
            port: Some(5432),
            database: Some("test".to_string()),
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            server: None,
            path: None,
            trusted_connection: None,
            timeout: None,
            extra: Default::default(),
        };

        let connector = PostgresConnector::new(&config).await;
        assert!(connector.is_ok());
    }
}
