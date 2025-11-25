use std::sync::Arc;

use industrydb_core::error::Result;

use super::execute_modify;
use crate::connector::MssqlConnector;

/// Decision making client.
pub struct DecisionMakingClient {
    connector: Arc<MssqlConnector>,
}

impl DecisionMakingClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    /// 检查/创建 decision 表。
    pub async fn check_decision_table(&self) -> Result<()> {
        let sql = r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'decision')
            BEGIN
                CREATE TABLE [dbo].[decision](
                    [DateTime] [datetime] NULL,
                    [variable_name] [varchar](50) NULL,
                    [optimization_project_name] [varchar](100) NULL,
                    [optimization_type] [varchar](50) NULL,
                    [decision_value] [float] NULL
                ) ON [PRIMARY]
            END
        "#;
        execute_modify(self.connector.as_ref(), sql).await?;
        Ok(())
    }
}
