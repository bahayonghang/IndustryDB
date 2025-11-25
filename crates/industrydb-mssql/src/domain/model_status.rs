use std::sync::Arc;

use industrydb_core::error::Result;
use polars::prelude::DataFrame;

use super::{execute_modify, query_df};
use crate::connector::MssqlConnector;

/// Model status client.
pub struct ModelStatusClient {
    connector: Arc<MssqlConnector>,
}

impl ModelStatusClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    pub async fn init_model_status_table(&self, project_name: &str) -> Result<()> {
        let sql = format!(
            r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = '{table}')
            BEGIN
                CREATE TABLE {table} (
                    DateTime datetime,
                    project_name NVARCHAR(255),
                    algorithm_name NVARCHAR(255),
                    model_type NVARCHAR(255),
                    model_parameter NVARCHAR(MAX),
                    start_time datetime,
                    end_time datetime,
                    sample_size int,
                    mae float,
                    rmse float,
                    mape float
                )
            END
        "#,
            table = project_name
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn insert_model_status(
        &self,
        project_name: &str,
        algorithm_name: &str,
        model_type: &str,
        model_parameter_json: &str,
        start_time: &str,
        end_time: &str,
        sample_size: i32,
        mae: f64,
        rmse: f64,
        mape: f64,
    ) -> Result<()> {
        let sql = format!(
            "INSERT INTO {table} (DateTime, project_name, algorithm_name, model_type, model_parameter, start_time, end_time, sample_size, mae, rmse, mape) \
             VALUES (GETDATE(), '{project}', '{algo}', '{model_type}', '{param}', '{start}', '{end}', {sample}, {mae}, {rmse}, {mape})",
            table = project_name,
            project = project_name,
            algo = algorithm_name,
            model_type = model_type,
            param = model_parameter_json.replace('\'', "''"),
            start = start_time,
            end = end_time,
            sample = sample_size,
            mae = mae,
            rmse = rmse,
            mape = mape
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    pub async fn get_model_status(&self, project_name: &str) -> Result<DataFrame> {
        let sql = format!("SELECT * FROM {}", project_name);
        query_df(self.connector.as_ref(), &sql).await
    }
}
