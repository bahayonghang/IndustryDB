use industrydb_core::error::{IndustryDbError, Result};

use crate::connector::MssqlConnector;

/// Execute non-query SQL and return rows affected.
pub async fn execute_rows_affected(connector: &MssqlConnector, sql: &str) -> Result<u64> {
    let mut conn = connector
        .pool()
        .get()
        .await
        .map_err(|e| IndustryDbError::ConnectionError(e.to_string()))?;

    let result = conn
        .execute(sql, &[])
        .await
        .map_err(|e| IndustryDbError::QueryError(e.to_string()))?;

    Ok(result.rows_affected().iter().sum())
}
