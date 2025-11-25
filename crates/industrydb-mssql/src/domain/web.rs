use std::sync::Arc;

use industrydb_core::error::{IndustryDbError, Result};
use polars::prelude::{DataFrame, IntoColumn, NamedFrom, Series};
use regex::Regex;
use serde_json::{json, Map, Value};

use super::{execute_modify, query_df};
use crate::connector::MssqlConnector;

/// Web database client (prediction/optimization metadata, train/thread status, device status).
pub struct WebClient {
    connector: Arc<MssqlConnector>,
}

impl WebClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    /// 获取预测信息 (sample_name, model_name) from yunxing.
    pub async fn get_prediction_info(&self, project_name: &str) -> Result<(String, String)> {
        let sql = format!(
            "SELECT * FROM yunxing WHERE sampletable_name = '{}'",
            project_name
        );
        let df = query_df(self.connector.as_ref(), &sql).await?;
        if df.is_empty() {
            return Err(IndustryDbError::config_error(format!(
                "Project name {} not found in yunxing",
                project_name
            )));
        }

        let sample_name = df
            .column("yangben")?
            .str()?
            .get(0)
            .unwrap_or("")
            .to_string();
        let model_name = df.column("moxing")?.str()?.get(0).unwrap_or("").to_string();
        Ok((sample_name, model_name))
    }

    /// 获取实时运行信息 (全行 DataFrame) from yunxing.
    pub async fn get_rt_info(&self, project_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT * FROM yunxing WHERE sampletable_name = '{}'",
            project_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 获取模型参数：返回 (algorithm_name, parameter_json_string)
    pub async fn get_model_parameter(&self, model_name: &str) -> Result<(String, String)> {
        let sql = format!("SELECT * FROM algoone WHERE project_name='{}'", model_name);
        let df = query_df(self.connector.as_ref(), &sql).await?;
        if df.is_empty() {
            return Err(IndustryDbError::config_error(format!(
                "Model {} not found in algoone",
                model_name
            )));
        }
        let algorithm = df
            .column("model_name")?
            .str()?
            .get(0)
            .unwrap_or("")
            .to_string();
        let parameter_raw = df
            .column("parameter")?
            .str()?
            .get(0)
            .unwrap_or("")
            .to_string();
        let parameter_json = serialize_prediction_parameter(&parameter_raw);
        Ok((algorithm, parameter_json))
    }

    /// 样本表信息。
    pub async fn get_sample_table_information(&self, sample_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT * FROM sampletablezong WHERE sample_table='{}'",
            sample_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    fn parse_input_names(&self, df: &DataFrame) -> Vec<String> {
        df.column("column_namein")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string())
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 获取优化模型的参数/约束/预测模型列表。
    pub async fn get_optimization_info(
        &self,
        project_name: &str,
    ) -> Result<(
        DataFrame,
        DataFrame,
        Vec<String>,
        Vec<String>,
        Vec<Vec<String>>,
    )> {
        let sql_arguments = format!(
            "SELECT * FROM optimization WHERE optimization_project_name='{}'",
            project_name
        );
        let optimization_arguments = query_df(self.connector.as_ref(), &sql_arguments).await?;
        if optimization_arguments.is_empty() {
            return Err(IndustryDbError::config_error(format!(
                "Project {} not found in optimization",
                project_name
            )));
        }

        let constraints = query_df(
            self.connector.as_ref(),
            &format!(
                "SELECT bianliang FROM optimization WHERE optimization_project_name='{}'",
                project_name
            ),
        )
        .await?;
        let constraint_str = constraints
            .column("bianliang")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string())
            .unwrap_or_default();
        let constraint_df = parse_constraints(&constraint_str)?;

        let prediction_models: Vec<String> = optimization_arguments
            .column("prediction_model")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string())
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut prediction_model_table_list = Vec::new();
        let mut prediction_model_list = Vec::new();
        for item in prediction_models.iter() {
            let parts: Vec<&str> = item.split('_').collect();
            if parts.len() >= 3 {
                prediction_model_table_list.push(parts[0].to_string());
                prediction_model_list.push(format!("{}_{}", parts[1], parts[2]));
            }
        }

        let mut input_name_list = Vec::new();
        for sample in prediction_model_table_list.iter() {
            let df = self
                .get_sample_table_information(sample)
                .await
                .unwrap_or_else(|_| DataFrame::default());
            input_name_list.push(self.parse_input_names(&df));
        }

        Ok((
            optimization_arguments,
            constraint_df,
            prediction_model_table_list,
            prediction_model_list,
            input_name_list,
        ))
    }

    /// 解析优化约束（简化：直接返回 optimization.bianliang 列对应的 DataFrame）
    pub async fn get_constraint_table(&self, project_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT bianliang FROM optimization WHERE optimization_project_name='{}'",
            project_name
        );
        let df = query_df(self.connector.as_ref(), &sql).await?;
        let constraints_str = df
            .column("bianliang")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string())
            .unwrap_or_default();
        parse_constraints(&constraints_str)
    }

    /// 完整约束视图（包含单变量幅度），当前等同于 get_constraint_table，解析交由上层。
    pub async fn get_constraint_table_full(&self, project_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT bianliang FROM optimization WHERE optimization_project_name='{}'",
            project_name
        );
        let df = query_df(self.connector.as_ref(), &sql).await?;
        let constraints_str = df
            .column("bianliang")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string())
            .unwrap_or_default();
        parse_constraints_full(&constraints_str)
    }

    /// TrainInfo 表检查/创建。
    pub async fn check_train_info_table(&self) -> Result<()> {
        let sql = r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'TrainInfo')
            BEGIN
                CREATE TABLE TrainInfo (
                    DateTime datetime,
                    project_name NVARCHAR(255),
                    epoch int,
                    train_loss float,
                    vali_loss float,
                    model_type NVARCHAR(255),
                    uuid NVARCHAR(255)
                );
            END
        "#;
        execute_modify(self.connector.as_ref(), sql).await?;
        Ok(())
    }

    /// 插入训练信息。
    pub async fn insert_train_info(
        &self,
        project_name: &str,
        epoch: i32,
        train_loss: f64,
        vali_loss: f64,
        model_type: &str,
        uuid: &str,
    ) -> Result<()> {
        let sql = format!(
            "INSERT INTO TrainInfo(DateTime, project_name, epoch, train_loss, vali_loss, model_type, uuid) \
             VALUES (GETDATE(), '{project}', {epoch}, {train_loss}, {vali_loss}, '{model_type}', '{uuid}')",
            project = project_name,
            epoch = epoch,
            train_loss = train_loss,
            vali_loss = vali_loss,
            model_type = model_type,
            uuid = uuid
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 确保 operation_status 表存在（兼容旧接口）。
    pub async fn ensure_table_exist_operation_status(&self) -> Result<()> {
        let sql = r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'operation_status')
            BEGIN
                CREATE TABLE operation_status (
                    DateTime datetime,
                    project_name NVARCHAR(255) PRIMARY KEY,
                    project_type NVARCHAR(255),
                    operation_status int
                );
            END
        "#;
        execute_modify(self.connector.as_ref(), sql).await?;
        Ok(())
    }

    /// 初始化 operation_status 表。
    pub async fn init_operation_status(&self) -> Result<()> {
        execute_modify(self.connector.as_ref(), "DELETE FROM operation_status").await?;
        Ok(())
    }

    /// 更新运行状态（已废弃兼容）。
    pub async fn update_operation_status(
        &self,
        project_name: &str,
        project_type: &str,
        operation_status: i32,
    ) -> Result<()> {
        let sql = format!(
            "INSERT INTO operation_status (DateTime, project_name, project_type, operation_status) VALUES (GETDATE(), '{name}', '{ptype}', {status})",
            name = project_name,
            ptype = project_type,
            status = operation_status
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 线程状态表检查/创建。
    pub async fn init_thread_status_table(&self) -> Result<()> {
        let sql = r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'thread_status')
            BEGIN
                CREATE TABLE thread_status (
                    DateTime datetime,
                    project_name NVARCHAR(255),
                    project_type NVARCHAR(255),
                    train_status int,
                    predict_status int,
                    PRIMARY KEY (project_name, project_type)
                );
            END
        "#;
        execute_modify(self.connector.as_ref(), sql).await?;
        Ok(())
    }

    /// 刷新 thread_status 表。
    pub async fn fresh_thread_status_table(&self) -> Result<()> {
        execute_modify(self.connector.as_ref(), "DELETE FROM thread_status").await?;
        Ok(())
    }

    /// 更新线程状态。
    pub async fn update_thread_status(
        &self,
        project_name: &str,
        project_type: &str,
        train_status: i32,
        predict_status: i32,
    ) -> Result<()> {
        let sql = format!(
            "MERGE thread_status AS t \
             USING (SELECT '{project_name}' AS project_name, '{project_type}' AS project_type) AS s \
             ON (t.project_name = s.project_name AND t.project_type = s.project_type) \
             WHEN MATCHED THEN UPDATE SET DateTime = GETDATE(), train_status = {train_status}, predict_status = {predict_status} \
             WHEN NOT MATCHED THEN INSERT (DateTime, project_name, project_type, train_status, predict_status) \
             VALUES (GETDATE(), '{project_name}', '{project_type}', {train_status}, {predict_status});",
            project_name = project_name,
            project_type = project_type,
            train_status = train_status,
            predict_status = predict_status,
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 在线训练表检查/创建。
    pub async fn check_online_trainer_table(&self) -> Result<()> {
        let sql = r#"
            IF NOT EXISTS (SELECT * FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_NAME = 'OnlineTrainer')
            BEGIN
                CREATE TABLE OnlineTrainer (
                    DateTime datetime,
                    project_name NVARCHAR(255),
                    epoch int,
                    train_loss float,
                    test_loss float
                );
            END
        "#;
        execute_modify(self.connector.as_ref(), sql).await?;
        Ok(())
    }

    /// 插入在线训练信息。
    pub async fn insert_online_trainer_info(
        &self,
        project_name: &str,
        epoch: i32,
        train_loss: f64,
        test_loss: f64,
    ) -> Result<()> {
        let sql = format!(
            "INSERT INTO OnlineTrainer(DateTime, project_name, epoch, train_loss, test_loss) \
             VALUES (GETDATE(), '{project_name}', {epoch}, {train_loss}, {test_loss})",
            project_name = project_name,
            epoch = epoch,
            train_loss = train_loss,
            test_loss = test_loss,
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 获取在线训练信息。
    pub async fn get_online_trainer_info(&self, project_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT TOP 60 * FROM OnlineTrainer WHERE project_name='{}' ORDER BY DateTime DESC",
            project_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 写入设备状态到 TagDatabase。
    pub async fn write_device_status(&self, project_name: &str, device_status: i32) -> Result<()> {
        let sql = format!(
            "MERGE TagDatabase AS t USING (SELECT '{name}' AS TagName) AS s ON (t.TagName = s.TagName) WHEN MATCHED THEN UPDATE SET TagVal = {status} WHEN NOT MATCHED THEN INSERT (TagName, TagVal) VALUES ('{name}', {status});",
            name = project_name,
            status = device_status
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 记录设备状态历史。
    pub async fn insert_device_status(&self, project_name: &str, device_status: i32) -> Result<()> {
        let sql = format!(
            "INSERT INTO device_status(DateTime, device_name, device_status) VALUES (GETDATE(), '{name}', {status})",
            name = project_name,
            status = device_status
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }
}

/// Online trainer client is currently an alias of WebClient with restricted methods.
pub struct OnlineTrainerClient {
    connector: Arc<MssqlConnector>,
}

impl OnlineTrainerClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    pub async fn check_online_trainer_table(&self) -> Result<()> {
        WebClient::new(self.connector.clone())
            .check_online_trainer_table()
            .await
    }

    pub async fn insert_online_trainer_info(
        &self,
        project_name: &str,
        epoch: i32,
        train_loss: f64,
        test_loss: f64,
    ) -> Result<()> {
        WebClient::new(self.connector.clone())
            .insert_online_trainer_info(project_name, epoch, train_loss, test_loss)
            .await
    }

    pub async fn get_online_trainer_info(&self, project_name: &str) -> Result<DataFrame> {
        WebClient::new(self.connector.clone())
            .get_online_trainer_info(project_name)
            .await
    }
}

fn serialize_prediction_parameter(param_str: &str) -> String {
    if param_str.trim().is_empty() {
        return "{}".to_string();
    }
    let mut map = Map::new();
    for kv in param_str.split(',') {
        if let Some((k, v)) = kv.split_once(':') {
            let key = k.trim().trim_matches('"');
            let raw = v.trim().trim_matches('"');
            if let Ok(i) = raw.parse::<i64>() {
                map.insert(key.to_string(), json!(i));
            } else if let Ok(f) = raw.parse::<f64>() {
                map.insert(key.to_string(), json!(f));
            } else {
                map.insert(key.to_string(), json!(raw));
            }
        }
    }
    Value::Object(map).to_string()
}

fn parse_constraints(constraints_str: &str) -> Result<DataFrame> {
    parse_constraints_impl(constraints_str, false)
}

fn parse_constraints_full(constraints_str: &str) -> Result<DataFrame> {
    parse_constraints_impl(constraints_str, true)
}

fn parse_constraints_impl(constraints_str: &str, with_extra: bool) -> Result<DataFrame> {
    if constraints_str.trim().is_empty() {
        let cols = if with_extra {
            vec![
                Series::new("variable".into(), Vec::<String>::new()).into_column(),
                Series::new("min".into(), Vec::<f64>::new()).into_column(),
                Series::new("max".into(), Vec::<f64>::new()).into_column(),
                Series::new("magnitude".into(), Vec::<Option<f64>>::new()).into_column(),
                Series::new("history_ref".into(), Vec::<Option<String>>::new()).into_column(),
            ]
        } else {
            vec![
                Series::new("variable".into(), Vec::<String>::new()).into_column(),
                Series::new("min".into(), Vec::<f64>::new()).into_column(),
                Series::new("max".into(), Vec::<f64>::new()).into_column(),
            ]
        };
        return Ok(DataFrame::new(cols)?);
    }

    let pattern = Regex::new(
        r"([^,]+?):\s*min:\s*([-+]?\d*\.?\d+)\s*max:\s*([-+]?\d*\.?\d+)(?:\s*magnitude:\s*([-+]?\d*\.?\d+))?(?:\s*history_ref:\s*([A-Za-z0-9]+))?",
    )
    .map_err(|e| IndustryDbError::InvalidParameter(e.to_string()))?;

    let mut variables = Vec::new();
    let mut mins = Vec::new();
    let mut maxs = Vec::new();
    let mut magnitudes: Vec<Option<f64>> = Vec::new();
    let mut history_refs: Vec<Option<String>> = Vec::new();

    for caps in pattern.captures_iter(constraints_str) {
        let name = caps
            .get(1)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        let min_v = caps
            .get(2)
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
        let max_v = caps
            .get(3)
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
        variables.push(name);
        mins.push(min_v);
        maxs.push(max_v);
        if with_extra {
            let mag = caps
                .get(4)
                .and_then(|m| m.as_str().parse::<f64>().ok());
            let hist = caps.get(5).map(|m| m.as_str().to_string());
            magnitudes.push(mag);
            history_refs.push(hist);
        }
    }

    if with_extra {
        Ok(DataFrame::new(vec![
            Series::new("variable".into(), variables).into_column(),
            Series::new("min".into(), mins).into_column(),
            Series::new("max".into(), maxs).into_column(),
            Series::new("magnitude".into(), magnitudes).into_column(),
            Series::new("history_ref".into(), history_refs).into_column(),
        ])?)
    } else {
        Ok(DataFrame::new(vec![
            Series::new("variable".into(), variables).into_column(),
            Series::new("min".into(), mins).into_column(),
            Series::new("max".into(), maxs).into_column(),
        ])?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_prediction_parameter() {
        let json = serialize_prediction_parameter("a:1,b:2.5,c:text");
        assert!(json.contains("\"a\":1"));
        assert!(json.contains("\"b\":2.5"));
        assert!(json.contains("\"c\":\"text\""));
    }

    #[test]
    fn test_parse_constraints_basic() {
        let df = parse_constraints("temp: min: 10 max: 20, press: min: -1.5 max: 5").unwrap();
        assert_eq!(df.height(), 2);
        assert!(df.column("variable").is_ok());
    }

    #[test]
    fn test_parse_constraints_full_with_extra() {
        let df = parse_constraints_full(
            "x: min: 0 max: 1 magnitude:0.1 history_ref:10min, y: min: 1 max: 2",
        )
        .unwrap();
        assert_eq!(df.height(), 2);
        assert!(df.column("magnitude").is_ok());
        assert!(df.column("history_ref").is_ok());
    }
}
