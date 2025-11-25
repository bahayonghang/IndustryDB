use std::sync::Arc;

use industrydb_core::error::Result;
use polars::prelude::DataFrame;

use super::{execute_modify, query_df};
use crate::connector::MssqlConnector;

/// Sequence realtime predict client.
pub struct RealtimePredictSequenceClient {
    connector: Arc<MssqlConnector>,
}

impl RealtimePredictSequenceClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    /// 创建序列表。
    pub async fn create_sequence_table(&self, table_name: &str, name_list: &[&str]) -> Result<()> {
        let mut columns = vec!["[DateTime] DATETIME PRIMARY KEY".to_string()];
        columns.extend(name_list.iter().map(|n| format!("[{}] FLOAT", n)));
        let sql = format!(
            "CREATE TABLE {table} ({cols})",
            table = table_name,
            cols = columns.join(", ")
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 创建序列预测表。
    pub async fn create_sequence_predict_table(
        &self,
        table_name: &str,
        name_list: &[&str],
    ) -> Result<()> {
        self.create_sequence_table(table_name, name_list).await
    }

    /// 删除序列表。
    pub async fn delete_sequence_table(&self, table_name: &str) -> Result<()> {
        let sql = format!(
            "IF OBJECT_ID('{table}', 'U') IS NOT NULL DROP TABLE {table}",
            table = table_name
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 更新真实值表（插入 DataFrame 行）。
    pub async fn update_realvalue(&self, table_name: &str, data: &DataFrame) -> Result<()> {
        // 简化：直接使用批量插入。
        let mut inserted = 0u64;
        for row_idx in 0..data.height() {
            let mut cols = vec![];
            let mut vals = vec![];
            for col in data.get_column_names() {
                cols.push(format!("[{}]", col));
                let val = data
                    .column(col)?
                    .get(row_idx)
                    .unwrap()
                    .to_string()
                    .replace('\'', "''");
                vals.push(format!("'{}'", val));
            }
            let sql = format!(
                "INSERT INTO {table} ({cols}) VALUES ({vals})",
                table = table_name,
                cols = cols.join(", "),
                vals = vals.join(", ")
            );
            inserted += execute_modify(self.connector.as_ref(), &sql).await?;
        }
        let _ = inserted;
        Ok(())
    }

    /// 更新预测值表（插入 DataFrame 行）。
    pub async fn update_predictvalue(&self, table_name: &str, data: &DataFrame) -> Result<()> {
        self.update_realvalue(table_name, data).await
    }

    /// 批量更新预测表（别名）。
    pub async fn update_sequence_predict_table(
        &self,
        table_name: &str,
        data: &DataFrame,
    ) -> Result<()> {
        self.update_predictvalue(table_name, data).await
    }

    /// 批量更新真实值表（别名）。
    pub async fn update_sequence_real_value_table(
        &self,
        table_name: &str,
        data: &DataFrame,
    ) -> Result<()> {
        self.update_realvalue(table_name, data).await
    }

    /// 综合更新序列数据（先真实值后预测值）。
    pub async fn update_sequence(
        &self,
        real_table: &str,
        real_df: &DataFrame,
        pred_table: &str,
        pred_df: &DataFrame,
    ) -> Result<()> {
        self.update_realvalue(real_table, real_df).await?;
        self.update_predictvalue(pred_table, pred_df).await?;
        Ok(())
    }

    /// 删除预测表中所有数据。
    pub async fn delete_sequence_predict_value(&self, table_name: &str) -> Result<()> {
        let sql = format!("TRUNCATE TABLE {}", table_name);
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 获取预测数据。
    pub async fn get_predict_data_by_table_name(&self, table_name: &str) -> Result<DataFrame> {
        let sql = format!("SELECT * FROM {}", table_name);
        query_df(self.connector.as_ref(), &sql).await
    }
}
