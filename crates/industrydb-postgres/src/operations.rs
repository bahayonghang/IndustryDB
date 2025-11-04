//! CRUD operations for PostgreSQL

use crate::connector::PostgresConnector;
use async_trait::async_trait;
use industrydb_core::{
    error::{IndustryDbError, Result},
    traits::{CrudOperations, DatabaseConnector},
};
use polars::prelude::*;
use std::collections::HashMap;

#[async_trait]
impl CrudOperations for PostgresConnector {
    async fn insert(&self, table: &str, data: DataFrame) -> Result<usize> {
        if data.height() == 0 {
            return Ok(0);
        }

        let columns: Vec<String> = data
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut rows_inserted = 0;

        for row_idx in 0..data.height() {
            let mut values = Vec::new();

            for col_name in columns.iter() {
                let column = data.column(col_name)?;
                let series = column.as_materialized_series();
                let value = format_value(series, row_idx)?;
                values.push(value);
            }

            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                columns.join(", "),
                values.join(", ")
            );

            match self.execute(&sql).await {
                Ok(_) => rows_inserted += 1,
                Err(e) => {
                    return Err(IndustryDbError::query_error(format!(
                        "Insert failed at row {}: {}",
                        row_idx, e
                    )));
                }
            }
        }

        Ok(rows_inserted)
    }

    async fn select(
        &self,
        table: &str,
        columns: Option<&[String]>,
        where_clause: Option<&str>,
        limit: Option<usize>,
    ) -> Result<DataFrame> {
        let cols = columns
            .map(|c| c.join(", "))
            .unwrap_or_else(|| "*".to_string());

        let mut sql = format!("SELECT {} FROM {}", cols, table);

        if let Some(where_cond) = where_clause {
            sql.push_str(&format!(" WHERE {}", where_cond));
        }

        if let Some(lim) = limit {
            sql.push_str(&format!(" LIMIT {}", lim));
        }

        self.execute(&sql).await
    }

    async fn update(
        &self,
        table: &str,
        values: &HashMap<String, String>,
        where_clause: Option<&str>,
    ) -> Result<usize> {
        if values.is_empty() {
            return Err(IndustryDbError::invalid_parameter("No values to update"));
        }

        let set_clause: Vec<String> = values
            .iter()
            .map(|(col, val)| format!("{} = {}", col, val))
            .collect();

        let mut sql = format!("UPDATE {} SET {}", table, set_clause.join(", "));

        if let Some(where_cond) = where_clause {
            sql.push_str(&format!(" WHERE {}", where_cond));
        }

        let result = sqlx::query(&sql)
            .execute(self.pool())
            .await
            .map_err(|e| IndustryDbError::QueryError(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }

    async fn delete(&self, table: &str, where_clause: Option<&str>) -> Result<usize> {
        let mut sql = format!("DELETE FROM {}", table);

        if let Some(where_cond) = where_clause {
            sql.push_str(&format!(" WHERE {}", where_cond));
        }

        let result = sqlx::query(&sql)
            .execute(self.pool())
            .await
            .map_err(|e| IndustryDbError::QueryError(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }
}

fn format_value(series: &Series, idx: usize) -> Result<String> {
    if series.is_null().get(idx).unwrap_or(false) {
        return Ok("NULL".to_string());
    }

    match series.dtype() {
        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Float32
        | DataType::Float64 => {
            let val = series.get(idx).unwrap();
            Ok(format!("{}", val))
        }
        DataType::String => {
            let val = series.get(idx).unwrap();
            let s = val.to_string().replace('\'', "''");
            Ok(format!("'{}'", s))
        }
        DataType::Boolean => {
            let val = series.get(idx).unwrap();
            Ok(if val.to_string() == "true" {
                "TRUE"
            } else {
                "FALSE"
            }
            .to_string())
        }
        _ => {
            let val = series.get(idx).unwrap();
            Ok(format!("'{}'", val.to_string().replace('\'', "''")))
        }
    }
}
