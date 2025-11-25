use std::sync::Arc;

use industrydb_core::error::Result;
use polars::prelude::DataFrame;

use super::query_df;
use crate::connector::MssqlConnector;

/// Quality database client.
pub struct QualityClient {
    connector: Arc<MssqlConnector>,
}

impl QualityClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    /// 训练输出数据窗口。
    pub async fn get_output_data_train(
        &self,
        output_table_name: &str,
        begin_time: &str,
        end_time: &str,
    ) -> Result<DataFrame> {
        let sql = format!(
            "SELECT * FROM {table} WHERE TagTime BETWEEN '{begin}' AND '{end}' ORDER BY TagTime",
            table = output_table_name,
            begin = begin_time,
            end = end_time
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 历史输出数据。
    pub async fn get_output_data_history(
        &self,
        output_table_name: &str,
        begin_time: &str,
        end_time: &str,
    ) -> Result<DataFrame> {
        self.get_output_data_train(output_table_name, begin_time, end_time)
            .await
    }

    /// 最新真实值（TagTime 降序取一行）。
    pub async fn get_latest_true_value(&self, table_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT TOP (1) * FROM {table} WHERE TagTime IS NOT NULL ORDER BY TagTime DESC",
            table = table_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 实时质量最新真实值。
    pub async fn get_latest_true_value_quality_rt(&self, table_name: &str) -> Result<DataFrame> {
        self.get_latest_true_value(table_name).await
    }

    /// 指定列的最新真实值。
    pub async fn get_latest_true_value_by_column(
        &self,
        column_name: &str,
        table_name: &str,
    ) -> Result<DataFrame> {
        let sql = format!(
            "SELECT TOP (1) TagTime, [{col}] FROM {table} WHERE TagTime IS NOT NULL AND [{col}] IS NOT NULL ORDER BY TagTime DESC",
            col = column_name,
            table = table_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 在线输出数据（最新一行）。
    pub async fn get_output_data_online(&self, output_table_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT TOP (1) * FROM {table} ORDER BY TagTime DESC",
            table = output_table_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 输出在线数据（指定列）。
    pub async fn get_output_data_online_by_column(
        &self,
        output_table_name: &str,
        column_name: &str,
    ) -> Result<DataFrame> {
        let sql = format!(
            "SELECT TOP (1) TagTime, [{col}] FROM {table} WHERE [{col}] IS NOT NULL ORDER BY TagTime DESC",
            col = column_name,
            table = output_table_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }
}
