use std::sync::Arc;

use industrydb_core::error::Result;

use super::{execute_modify, query_df};
use crate::connector::MssqlConnector;

/// Standard realtime predict client.
pub struct RealtimePredictClient {
    connector: Arc<MssqlConnector>,
}

impl RealtimePredictClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    /// 检查/创建预测表，包含多列预测值与 AI 指标。
    pub async fn check_table(&self, table_name: &str) -> Result<()> {
        let sql = format!(
            r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = '{table}')
            BEGIN
                CREATE TABLE {table} (
                    [DateTime] DATETIME PRIMARY KEY,
                    [真实值] REAL,
                    [软测量预测值] REAL,
                    [3分钟预测值] REAL,
                    [5分钟预测值] REAL,
                    [30分钟预测值] REAL,
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

    /// 检查/创建 multi-output 表（若存在则保留）。
    pub async fn check_table_multi_output(
        &self,
        table_name: &str,
        name_list: &[&str],
    ) -> Result<()> {
        let exists_sql = format!(
            "SELECT COUNT(*) AS cnt FROM sys.tables WHERE name = '{}'",
            table_name
        );
        let df = query_df(self.connector.as_ref(), &exists_sql).await?;
        let exists = df
            .column("cnt")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string().parse::<i64>().unwrap_or(0) > 0)
            .unwrap_or(false);
        if exists {
            return Ok(());
        }
        self.create_table_multi_output(table_name, name_list).await
    }

    /// 创建 multi-output 表。
    pub async fn create_table_multi_output(
        &self,
        project_name: &str,
        output_name_list: &[&str],
    ) -> Result<()> {
        let name_list: Vec<&str> = output_name_list
            .iter()
            .filter(|n| {
                let lower = n.to_ascii_lowercase();
                lower != "datetime" && lower != "tagtime"
            })
            .copied()
            .collect();
        if name_list.is_empty() {
            return Ok(());
        }
        let mut columns = vec!["[DateTime] DATETIME PRIMARY KEY".to_string()];
        columns.extend(name_list.iter().map(|n| format!("[{}] FLOAT", n)));
        let create_sql = format!("CREATE TABLE {} ({})", project_name, columns.join(", "));
        execute_modify(self.connector.as_ref(), &create_sql).await?;
        Ok(())
    }

    /// 更新真实值（存在则更新，不存在则插入）。
    pub async fn update_realvalue(
        &self,
        table_name: &str,
        realvalue: f64,
        time_real: &str,
    ) -> Result<()> {
        let sql = format!(
            r#"
            MERGE {table} AS t
            USING (SELECT CAST('{time}' AS DATETIME) AS DateTime) AS s
            ON (t.DateTime = s.DateTime)
            WHEN MATCHED THEN UPDATE SET [真实值] = {real}
            WHEN NOT MATCHED THEN INSERT (DateTime, [真实值]) VALUES (s.DateTime, {real});
        "#,
            table = table_name,
            time = time_real,
            real = realvalue
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 更新 AI 预测值。
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

    /// 更新多输出预测值。
    pub async fn update_predictvalue_multi_output(
        &self,
        table_name: &str,
        time_predict: &str,
        predict_df: &polars::prelude::DataFrame,
    ) -> Result<()> {
        // For each column (except DateTime), upsert value at time_predict.
        for col in predict_df.get_columns() {
            if col.name() == "DateTime" || col.name() == "TagTime" {
                continue;
            }
            let val = col.get(0).unwrap().to_string();
            let sql = format!(
                r#"
                MERGE {table} AS t
                USING (SELECT CAST('{time}' AS DATETIME) AS DateTime) AS s
                ON (t.DateTime = s.DateTime)
                WHEN MATCHED THEN UPDATE SET [{col}] = {val}
                WHEN NOT MATCHED THEN INSERT (DateTime, [{col}]) VALUES (s.DateTime, {val});
            "#,
                table = table_name,
                time = time_predict,
                col = col.name(),
                val = val
            );
            execute_modify(self.connector.as_ref(), &sql).await?;
        }
        Ok(())
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

    /// 批量更新 AI 预测值/指标（DataFrame，每行一个时间点）。
    pub async fn update_ai_predict_value(
        &self,
        table_name: &str,
        df: &polars::prelude::DataFrame,
    ) -> Result<()> {
        if df.height() == 0 {
            return Ok(());
        }
        let time_col = if df.get_column_names().iter().any(|c| *c == "DateTime") {
            "DateTime"
        } else if df.get_column_names().iter().any(|c| *c == "TagTime") {
            "TagTime"
        } else {
            return Ok(());
        };

        for row_idx in 0..df.height() {
            let time_val = df
                .column(time_col)?
                .get(row_idx)
                .unwrap()
                .to_string()
                .replace('\'', "''");
            for col in df.get_column_names() {
                if col == time_col {
                    continue;
                }
                let val = df.column(col)?.get(row_idx).unwrap().to_string();
                let sql = format!(
                    r#"
                    MERGE {table} AS t
                    USING (SELECT CAST('{time}' AS DATETIME) AS DateTime) AS s
                    ON (t.DateTime = s.DateTime)
                    WHEN MATCHED THEN UPDATE SET [{col}] = {val}
                    WHEN NOT MATCHED THEN INSERT (DateTime, [{col}]) VALUES (s.DateTime, {val});
                "#,
                    table = table_name,
                    time = time_val,
                    col = col,
                    val = val
                );
                execute_modify(self.connector.as_ref(), &sql).await?;
            }
        }
        Ok(())
    }

    /// 更新 AI 预测指标（单值）。
    pub async fn update_ai_predict_index(
        &self,
        table_name: &str,
        ai_index: f64,
        time_predict: &str,
    ) -> Result<()> {
        let sql = format!(
            r#"
            MERGE {table} AS t
            USING (SELECT CAST('{time}' AS DATETIME) AS DateTime) AS s
            ON (t.DateTime = s.DateTime)
            WHEN MATCHED THEN UPDATE SET [AI预测指标] = {idx}
            WHEN NOT MATCHED THEN INSERT (DateTime, [AI预测指标]) VALUES (s.DateTime, {idx});
        "#,
            table = table_name,
            time = time_predict,
            idx = ai_index
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 查询最新预测值。
    pub async fn query_latest_predict_value(
        &self,
        table_name: &str,
    ) -> Result<polars::prelude::DataFrame> {
        let sql = format!(
            "SELECT TOP (1) * FROM {table} ORDER BY DateTime DESC",
            table = table_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }
}
