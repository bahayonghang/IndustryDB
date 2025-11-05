//! MSSQL connector implementation using tiberius with connection pooling

use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use industrydb_core::{
    config::ConnectionConfig,
    error::{IndustryDbError, Result},
    traits::DatabaseConnector,
};
use polars::prelude::*;
use tiberius::{Config, Row as TiberiusRow};

type TiberiusPool = Pool<ConnectionManager>;

/// MSSQL database connector with connection pool
pub struct MssqlConnector {
    pool: TiberiusPool,
    db_type: String,
}

impl MssqlConnector {
    /// Create a new MSSQL connector with connection pool
    pub async fn new(config: &ConnectionConfig) -> Result<Self> {
        let mut tiberius_config = Config::new();
        tiberius_config.host(config.host.as_deref().unwrap_or("localhost"));
        tiberius_config.port(config.port.unwrap_or(1433));
        tiberius_config.authentication(tiberius::AuthMethod::sql_server(
            config.username.as_deref().unwrap_or("sa"),
            config.password.as_deref().unwrap_or(""),
        ));

        if let Some(db) = &config.database {
            tiberius_config.database(db);
        }

        let manager = ConnectionManager::new(tiberius_config);
        let pool = Pool::builder()
            .build(manager)
            .await
            .map_err(|e| IndustryDbError::ConnectionError(e.to_string()))?;

        Ok(Self {
            pool,
            db_type: "mssql".to_string(),
        })
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &TiberiusPool {
        &self.pool
    }
}

#[async_trait]
impl DatabaseConnector for MssqlConnector {
    fn db_type(&self) -> &str {
        &self.db_type
    }

    async fn execute(&self, sql: &str) -> Result<DataFrame> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| IndustryDbError::ConnectionError(e.to_string()))?;

        let stream = conn
            .query(sql, &[])
            .await
            .map_err(|e| IndustryDbError::QueryError(e.to_string()))?;

        let rows = stream
            .into_results()
            .await
            .map_err(|e| IndustryDbError::QueryError(e.to_string()))?;

        if rows.is_empty() {
            return Ok(DataFrame::empty());
        }

        rows_to_dataframe(&rows[0])
    }

    async fn is_alive(&self) -> bool {
        if let Ok(mut conn) = self.pool.get().await {
            conn.query("SELECT 1", &[]).await.is_ok()
        } else {
            false
        }
    }

    async fn close(&mut self) -> Result<()> {
        // bb8 pool doesn't have explicit close, connections are dropped
        Ok(())
    }

    fn is_closed(&self) -> bool {
        // bb8 pool doesn't track closed state
        false
    }
}

/// Convert tiberius rows to Polars DataFrame
fn rows_to_dataframe(rows: &[TiberiusRow]) -> Result<DataFrame> {
    if rows.is_empty() {
        return Ok(DataFrame::empty());
    }

    let column_count = rows[0].len();
    let mut series_vec: Vec<Series> = Vec::new();

    for col_idx in 0..column_count {
        let col_name = rows[0]
            .columns()
            .get(col_idx)
            .map(|c| c.name())
            .unwrap_or("unknown");

        // Try different types - tiberius doesn't expose ColumnData type easily
        // So we try to decode each type and use the first one that works
        let series = if let Ok(values) = rows
            .iter()
            .map(|row| row.try_get::<i32, _>(col_idx))
            .collect::<std::result::Result<Vec<_>, _>>()
        {
            Series::new(col_name.into(), values)
        } else if let Ok(values) = rows
            .iter()
            .map(|row| row.try_get::<i64, _>(col_idx))
            .collect::<std::result::Result<Vec<_>, _>>()
        {
            Series::new(col_name.into(), values)
        } else if let Ok(values) = rows
            .iter()
            .map(|row| row.try_get::<f64, _>(col_idx))
            .collect::<std::result::Result<Vec<_>, _>>()
        {
            Series::new(col_name.into(), values)
        } else if let Ok(values) = rows
            .iter()
            .map(|row| row.try_get::<bool, _>(col_idx))
            .collect::<std::result::Result<Vec<_>, _>>()
        {
            Series::new(col_name.into(), values)
        } else {
            // Default to string
            let values: Vec<Option<String>> = rows
                .iter()
                .map(|row| {
                    row.try_get::<&str, _>(col_idx)
                        .ok()
                        .flatten()
                        .map(|s| s.to_string())
                })
                .collect();
            Series::new(col_name.into(), values)
        };

        series_vec.push(series);
    }

    let columns: Vec<_> = series_vec.into_iter().map(|s| s.into_column()).collect();
    DataFrame::new(columns).map_err(|e| IndustryDbError::PolarsError(e.to_string()))
}
