use std::sync::Arc;

use industrydb_core::error::Result;
use polars::prelude::DataFrame;

use super::{execute_modify, query_df};
use crate::connector::MssqlConnector;

/// Operation (training status) client.
pub struct OperationClient {
    connector: Arc<MssqlConnector>,
}

impl OperationClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    pub async fn check_operation_table(&self, project_name: &str) -> Result<()> {
        let sql = format!(
            r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = '{table}')
            BEGIN
                CREATE TABLE {table} (
                    DateTime datetime,
                    project_name NVARCHAR(255),
                    epoch int,
                    train_loss float,
                    vali_loss float,
                    sample_size int,
                    start_time datetime
                )
            END
        "#,
            table = project_name
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    pub async fn insert_operation_data(
        &self,
        project_name: &str,
        epoch: i32,
        train_loss: f64,
        vali_loss: f64,
        sample_size: i32,
        start_time: &str,
    ) -> Result<()> {
        let sql = format!(
            "INSERT INTO {table} (DateTime, project_name, epoch, train_loss, vali_loss, sample_size, start_time) \
             VALUES (GETDATE(), '{project}', {epoch}, {train}, {vali}, {sample}, '{start_time}')",
            table = project_name,
            project = project_name,
            epoch = epoch,
            train = train_loss,
            vali = vali_loss,
            sample = sample_size,
            start_time = start_time
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    pub async fn get_operation_data(&self, project_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT * FROM {table} WHERE DateTime >= DATEADD(month, -1, GETDATE())",
            table = project_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }
}
