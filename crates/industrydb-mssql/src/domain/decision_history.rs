use std::sync::Arc;

use industrydb_core::error::Result;
use polars::prelude::DataFrame;

use super::{execute_modify, query_df};
use crate::connector::MssqlConnector;

/// Decision history client.
pub struct DecisionHistoryClient {
    connector: Arc<MssqlConnector>,
}

impl DecisionHistoryClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    pub async fn check_table_decision_history(&self) -> Result<()> {
        let sql = r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'decision_history')
            BEGIN
                CREATE TABLE decision_history (
                    [DateTime] DATETIME,
                    [变量名称] NVARCHAR(255),
                    [optimization_project_name] NVARCHAR(255),
                    [optimization_type] NVARCHAR(255),
                    [反馈值] DECIMAL(10, 4),
                    [实际设定值] DECIMAL(10, 4),
                    [AI决策值] DECIMAL(10, 4),
                    [consistency] DECIMAL(10, 4)
                );
            END
        "#;
        execute_modify(self.connector.as_ref(), sql).await?;
        Ok(())
    }

    pub async fn insert_decision_history(
        &self,
        datetime: &str,
        variable_name: &str,
        project_name: &str,
        opt_type: &str,
        decision_value: f64,
    ) -> Result<()> {
        let sql = format!(
            "INSERT INTO decision_history(DateTime, [变量名称], optimization_project_name, optimization_type, [AI决策值]) \
             VALUES ('{dt}', '{var}', '{proj}', '{opt}', {val})",
            dt = datetime,
            var = variable_name,
            proj = project_name,
            opt = opt_type,
            val = decision_value
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    pub async fn query_decision_history_recent_1h(&self) -> Result<DataFrame> {
        let sql = "SELECT * FROM decision_history WHERE DateTime >= DATEADD(hour, -1, GETDATE())";
        query_df(self.connector.as_ref(), sql).await
    }

    /// 检查/创建决策一致性表。
    pub async fn check_table_decision_consistency(
        &self,
        energy_consumption_list: &[&str],
    ) -> Result<()> {
        let exist_sql = "SELECT COUNT(*) AS cnt FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'decision_consistency'";
        let df = query_df(self.connector.as_ref(), exist_sql).await?;
        let exists = df
            .column("cnt")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string().parse::<i64>().unwrap_or(0) > 0)
            .unwrap_or(false);
        if exists {
            return Ok(());
        }
        let mut cols = vec![
            "[DateTime] DATETIME".to_string(),
            "[optimization_project_name] NVARCHAR(255)".to_string(),
            "[optimization_type] NVARCHAR(255)".to_string(),
            "[consistency] DECIMAL(10, 4)".to_string(),
        ];
        cols.extend(
            energy_consumption_list
                .iter()
                .map(|n| format!("[{}] DECIMAL(10, 4)", n)),
        );
        let sql = format!("CREATE TABLE decision_consistency ({})", cols.join(", "));
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 更新决策一致性（插入一行）。
    pub async fn update_decision_consistency(
        &self,
        current_time: &str,
        project_name: &str,
        opt_type: &str,
        consistency: f64,
        energy_values: &[(&str, f64)],
    ) -> Result<()> {
        let mut cols = vec![
            "DateTime".to_string(),
            "optimization_project_name".to_string(),
            "optimization_type".to_string(),
            "consistency".to_string(),
        ];
        let mut vals = vec![
            format!("'{}'", current_time),
            format!("'{}'", project_name),
            format!("'{}'", opt_type),
            consistency.to_string(),
        ];
        for (name, val) in energy_values {
            cols.push(format!("[{}]", name));
            vals.push(val.to_string());
        }
        let sql = format!(
            "INSERT INTO decision_consistency ({cols}) VALUES ({vals})",
            cols = cols.join(", "),
            vals = vals.join(", ")
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 查询最近一条决策一致性数据。
    pub async fn select_recent_decision_data(&self) -> Result<DataFrame> {
        let sql = "SELECT TOP (1) * FROM decision_consistency ORDER BY DateTime DESC";
        query_df(self.connector.as_ref(), sql).await
    }
}
