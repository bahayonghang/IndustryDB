use std::sync::Arc;

use industrydb_core::error::Result;
use polars::prelude::DataFrame;

use super::{execute_modify, query_df};
use crate::connector::MssqlConnector;

/// Lightweight realtime predict client (alter).
pub struct RealtimePredictAlterClient {
    connector: Arc<MssqlConnector>,
}

impl RealtimePredictAlterClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    pub async fn check_table(&self, table_name: &str) -> Result<()> {
        let sql = format!(
            r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = '{table}')
            BEGIN
                CREATE TABLE {table} (
                    [DateTime] DATETIME PRIMARY KEY,
                    [真实值] REAL,
                    [AI预测值] REAL,
                    [AI预测指标] REAL
                )
            END
        "#,
            table = table_name
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    pub async fn update_predictvalue(
        &self,
        table_name: &str,
        predictvalue: f64,
        time_predict: &str,
    ) -> Result<()> {
        let sql = format!(
            r#"
            MERGE {table} AS t
            USING (SELECT CAST('{time}' AS DATETIME) AS DateTime) AS s
            ON (t.DateTime = s.DateTime)
            WHEN MATCHED THEN UPDATE SET [AI预测值] = {pred}
            WHEN NOT MATCHED THEN INSERT (DateTime, [AI预测值]) VALUES (s.DateTime, {pred});
        "#,
            table = table_name,
            time = time_predict,
            pred = predictvalue
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    pub async fn get_predict_data_by_table_name(&self, table_name: &str) -> Result<DataFrame> {
        let sql = format!("SELECT * FROM {}", table_name);
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 为表添加 AI 预测列。
    pub async fn add_ai_predict_column(&self, table_name: &str) -> Result<()> {
        let sql = format!(
            "ALTER TABLE {table} ADD [AI预测值] REAL, [AI预测指标] REAL",
            table = table_name
        );
        let _ = execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }
}
