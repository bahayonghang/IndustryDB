use std::sync::Arc;

use chrono::{DateTime, Duration as ChronoDuration, NaiveDateTime, TimeZone, Utc};
use industrydb_core::error::{IndustryDbError, Result};
use polars::prelude::*;

use super::{execute_modify, query_df};
use crate::connector::MssqlConnector;

#[derive(Clone, Copy)]
enum TruncateUnit {
    None,
    Second,
    Minute,
    Hour,
}

impl TruncateUnit {
    fn sql_expr(&self) -> &'static str {
        match self {
            TruncateUnit::None => "DateTime",
            TruncateUnit::Second => "DATEADD(second, DATEDIFF(second, 0, DateTime), 0)",
            TruncateUnit::Minute => "DATEADD(minute, DATEDIFF(minute, 0, DateTime), 0)",
            TruncateUnit::Hour => "DATEADD(hour, DATEDIFF(hour, 0, DateTime), 0)",
        }
    }
}

fn sanitize_columns(df: &mut DataFrame) -> Result<()> {
    let names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|name| name.replace(' ', ""))
        .collect();
    df.set_column_names(&names)?;
    Ok(())
}

fn tail_df(df: DataFrame, n: usize) -> DataFrame {
    if n >= df.height() {
        df
    } else {
        df.tail(Some(n))
    }
}

fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
        .ok()
        .or_else(|| NaiveDateTime::parse_from_str(value, "%Y-%m-%d").ok())
        .or_else(|| {
            DateTime::parse_from_rfc3339(value)
                .ok()
                .map(|dt| dt.naive_utc())
        })
}

fn parse_chunk_size(chunk_size: &str) -> ChronoDuration {
    let lowered = chunk_size.to_ascii_lowercase();
    if lowered.contains("week") {
        let weeks = lowered
            .split_whitespace()
            .next()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(1);
        ChronoDuration::days(7 * weeks)
    } else if lowered.contains("month") {
        ChronoDuration::days(30)
    } else if lowered.contains("day") {
        let days = lowered
            .split_whitespace()
            .next()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(7);
        ChronoDuration::days(days)
    } else {
        ChronoDuration::days(7)
    }
}

async fn query_tag_val(connector: &MssqlConnector, tag_name: &str) -> Result<Option<f64>> {
    let sql = format!(
        "SELECT TOP 1 TagVal FROM TagDatabase WHERE TagName = '{name}'",
        name = tag_name
    );
    let df = query_df(connector, &sql).await?;
    if df.is_empty() {
        return Ok(None);
    }
    Ok(df
        .column("TagVal")
        .ok()
        .and_then(|c| c.get(0).ok())
        .and_then(|v| v.to_string().parse::<f64>().ok()))
}

async fn query_first_tag_name(connector: &MssqlConnector, sql: &str) -> Result<Option<String>> {
    let df = query_df(connector, sql).await?;
    if df.is_empty() {
        return Ok(None);
    }
    Ok(df
        .column("tagName")
        .or_else(|_| df.column("TagName"))
        .ok()
        .and_then(|c| c.get(0).ok())
        .map(|v| v.to_string()))
}

/// TimeSeries database client.
pub struct TimeSeriesClient {
    connector: Arc<MssqlConnector>,
}

impl TimeSeriesClient {
    pub fn new(connector: Arc<MssqlConnector>) -> Self {
        Self { connector }
    }

    /// 获取最近 1 小时实时数据（与参考 get_latest_input_data 相似）。
    pub async fn get_latest_input_data(&self, table_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT TOP 360 * FROM {table} \
             WHERE (DateTime BETWEEN SUBSTRING(CONVERT(varchar, DATEADD(hour, -2, GETDATE()), 120), 1, 16) \
             AND SUBSTRING(CONVERT(varchar, DATEADD(hour, +1, GETDATE()), 120), 1, 16)) \
             ORDER BY DateTime DESC",
            table = table_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 获取最近时间长度的输入数据（视图，按小时/分钟聚合，简化为窗口查询）。
    pub async fn get_latest_input_data_view(
        &self,
        table_name: &str,
        time_length: i64,
        unit: &str,
    ) -> Result<DataFrame> {
        let sql = if unit.eq_ignore_ascii_case("hour") {
            let hours_before = -(time_length * 2 + 2);
            format!(
                "SELECT TOP {top} * FROM {table} \
                 WHERE DateTime BETWEEN SUBSTRING(CONVERT(varchar, DATEADD(hour, {hours_before}, GETDATE()), 120), 1, 16) \
                 AND SUBSTRING(CONVERT(varchar, DATEADD(hour, 1, GETDATE()), 120), 1, 16) \
                 ORDER BY DateTime DESC",
                top = time_length * 2,
                table = table_name,
                hours_before = hours_before
            )
        } else {
            let minutes_before = -(time_length * 2 + 60);
            format!(
                "SELECT TOP {top} * FROM {table} \
                 WHERE DateTime BETWEEN SUBSTRING(CONVERT(varchar, DATEADD(minute, {minutes_before}, GETDATE()), 120), 1, 16) \
                 AND SUBSTRING(CONVERT(varchar, DATEADD(minute, 1, GETDATE()), 120), 1, 16) \
                 ORDER BY DateTime DESC",
                top = time_length * 2,
                table = table_name,
                minutes_before = minutes_before
            )
        };
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 获取指定列的最新真实值。
    pub async fn get_latest_true_value_by_column(&self, column_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT TOP (1) TagVal FROM 历史表 WHERE DateTime BETWEEN DATEADD(HOUR, -1, GETDATE()) AND GETDATE() AND TagName = '{col}' ORDER BY DateTime DESC",
            col = column_name
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 获取最新真实值（默认 TagTime / DateTime 降序）。
    pub async fn get_latest_true_value(&self, table_name: &str) -> Result<DataFrame> {
        let sql = format!(
            "SELECT TOP (1) * FROM {table} \
             WHERE (DateTime BETWEEN SUBSTRING(CONVERT(varchar, DATEADD(hour, -2, GETDATE()), 120), 1, 16) \
                AND SUBSTRING(CONVERT(varchar, DATEADD(hour, +1, GETDATE()), 120), 1, 16)) \
             ORDER BY DateTime DESC",
            table = table_name,
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 在线输入数据获取（按列名列表）。
    pub async fn get_input_data_online(&self, name_list: &[&str]) -> Result<DataFrame> {
        if name_list.is_empty() {
            return Ok(DataFrame::default());
        }
        let mut df = pivot_history_between_truncated(
            self.connector.as_ref(),
            name_list,
            "DATEADD(MONTH, -12, GETDATE())",
            "GETDATE()",
            TruncateUnit::Second,
            None,
        )
        .await?;
        sanitize_columns(&mut df)?;
        if df.height() < 1000 {
            return Err(IndustryDbError::InvalidParameter(format!(
                "在线训练数据不足: {} 行",
                df.height()
            )));
        }
        Ok(df)
    }

    /// 在线输出数据获取（按列名列表）。
    pub async fn get_output_data_online(&self, name_list: &[&str]) -> Result<DataFrame> {
        if name_list.is_empty() {
            return Ok(DataFrame::default());
        }
        let mut df = pivot_history_between_truncated(
            self.connector.as_ref(),
            name_list,
            "DATEADD(MONTH, -1, GETDATE())",
            "GETDATE()",
            TruncateUnit::Second,
            None,
        )
        .await?;
        sanitize_columns(&mut df)?;
        Ok(df)
    }

    /// 输入数据时间窗口查询（按列，基于小时）。
    pub async fn get_latest_input_data_by_column_time(
        &self,
        input_name_list: &[&str],
        time_length_hours: i64,
    ) -> Result<DataFrame> {
        let df = pivot_history_between_truncated(
            self.connector.as_ref(),
            input_name_list,
            "DATEADD(HOUR, -6, GETDATE())",
            "GETDATE()",
            TruncateUnit::Minute,
            None,
        )
        .await?;
        Ok(tail_df(df, time_length_hours as usize))
    }

    /// 输入数据时间窗口查询（按列，基于分钟）。
    pub async fn get_latest_input_data_by_column_time_min(
        &self,
        input_name_list: &[&str],
        time_length_minutes: i64,
    ) -> Result<DataFrame> {
        let df = pivot_history_between_truncated(
            self.connector.as_ref(),
            input_name_list,
            &format!("DATEADD(MINUTE, -{}, GETDATE())", time_length_minutes),
            "GETDATE()",
            TruncateUnit::Minute,
            None,
        )
        .await?;
        Ok(tail_df(df, time_length_minutes as usize))
    }

    /// 输入数据时间窗口查询（按列，基于小时，聚焦决策/优化查询）。
    pub async fn get_latest_input_data_by_column_time_hour(
        &self,
        input_name_list: &[&str],
        time_length_hours: i64,
    ) -> Result<DataFrame> {
        let df = pivot_history_between_truncated(
            self.connector.as_ref(),
            input_name_list,
            "DATEADD(HOUR, -36, GETDATE())",
            "GETDATE()",
            TruncateUnit::Hour,
            None,
        )
        .await?;
        Ok(tail_df(df, time_length_hours as usize))
    }

    /// 历史数据（按列列表，时间范围）。
    pub async fn get_history_data_by_name_list(
        &self,
        input_name_list: &[&str],
        begin_time: &str,
        end_time: &str,
    ) -> Result<DataFrame> {
        if input_name_list.is_empty() {
            return Ok(DataFrame::default());
        }
        let mut df = pivot_history_between_truncated(
            self.connector.as_ref(),
            input_name_list,
            begin_time,
            end_time,
            TruncateUnit::Second,
            None,
        )
        .await?;
        sanitize_columns(&mut df)?;
        Ok(df)
    }

    /// 输入训练数据窗口。
    pub async fn get_input_data_train(
        &self,
        input_table_name: &str,
        begin_time: &str,
        end_time: &str,
    ) -> Result<DataFrame> {
        let sql = format!(
            "SELECT * FROM {table} WHERE DateTime BETWEEN '{begin}' AND '{end}' ORDER BY DateTime",
            table = input_table_name,
            begin = begin_time,
            end = end_time
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 输入历史数据窗口。
    pub async fn get_input_data_history(
        &self,
        name_list: &[&str],
        begin_time: &str,
        end_time: &str,
        chunk_size: Option<&str>,
        _use_temp_file: bool,
    ) -> Result<DataFrame> {
        if name_list.is_empty() {
            return Ok(DataFrame::default());
        }

        let start = parse_datetime(begin_time)
            .ok_or_else(|| IndustryDbError::InvalidParameter("Invalid begin_time".to_string()))?;
        let end = parse_datetime(end_time)
            .ok_or_else(|| IndustryDbError::InvalidParameter("Invalid end_time".to_string()))?;
        let step = parse_chunk_size(chunk_size.unwrap_or("7 days"));

        let mut chunks: Vec<DataFrame> = Vec::new();
        let mut current = start;
        while current < end {
            let next = std::cmp::min(current + step, end);
            let sql_start = current.format("%Y-%m-%d %H:%M:%S").to_string();
            let sql_end = next.format("%Y-%m-%d %H:%M:%S").to_string();
            let mut df = pivot_history_between_truncated(
                self.connector.as_ref(),
                name_list,
                &format!("'{}'", sql_start),
                &format!("'{}'", sql_end),
                TruncateUnit::Second,
                None,
            )
            .await?;
            if !df.is_empty() {
                sanitize_columns(&mut df)?;
                chunks.push(df);
            }
            if next == current {
                break;
            }
            current = next;
        }

        if chunks.is_empty() {
            return Ok(DataFrame::default());
        }

        let mut iter = chunks.into_iter();
        let mut df = iter.next().unwrap_or_default();
        for chunk in iter {
            df.vstack_mut(&chunk)?;
        }
        Ok(df)
    }

    /// 最新输入数据（按列列表）。
    pub async fn get_latest_input_data_by_column(
        &self,
        input_name_list: &[&str],
    ) -> Result<DataFrame> {
        let df = pivot_history_between_truncated(
            self.connector.as_ref(),
            input_name_list,
            "DATEADD(HOUR, -6, GETDATE())",
            "GETDATE()",
            TruncateUnit::Second,
            None,
        )
        .await?;
        Ok(tail_df(df, 360))
    }

    /// 输出训练数据窗口。
    pub async fn get_output_data_train(
        &self,
        output_table_name: &str,
        begin_time: &str,
        end_time: &str,
    ) -> Result<DataFrame> {
        let sql = format!(
            "SELECT * FROM {table} WHERE DateTime BETWEEN '{begin}' AND '{end}' ORDER BY DateTime",
            table = output_table_name,
            begin = begin_time,
            end = end_time
        );
        query_df(self.connector.as_ref(), &sql).await
    }

    /// 输出历史数据窗口。
    pub async fn get_output_data_history(
        &self,
        name_list: &[&str],
        begin_time: &str,
        end_time: &str,
        chunk_size: Option<&str>,
    ) -> Result<DataFrame> {
        if name_list.is_empty() {
            return Ok(DataFrame::default());
        }
        let start = parse_datetime(begin_time)
            .ok_or_else(|| IndustryDbError::InvalidParameter("Invalid begin_time".to_string()))?;
        let end = parse_datetime(end_time)
            .ok_or_else(|| IndustryDbError::InvalidParameter("Invalid end_time".to_string()))?;
        let step = parse_chunk_size(chunk_size.unwrap_or("30 days"));
        let mut chunks: Vec<DataFrame> = Vec::new();
        let mut current = start;
        while current < end {
            let next = std::cmp::min(current + step, end);
            let sql_start = current.format("%Y-%m-%d %H:%M:%S").to_string();
            let sql_end = next.format("%Y-%m-%d %H:%M:%S").to_string();
            let mut df = pivot_history_between_truncated(
                self.connector.as_ref(),
                name_list,
                &format!("'{}'", sql_start),
                &format!("'{}'", sql_end),
                TruncateUnit::Second,
                None,
            )
            .await?;
            if !df.is_empty() {
                sanitize_columns(&mut df)?;
                chunks.push(df);
            }
            if next == current {
                break;
            }
            current = next;
        }
        if chunks.is_empty() {
            return Ok(DataFrame::default());
        }
        let mut iter = chunks.into_iter();
        let mut df = iter.next().unwrap_or_default();
        for chunk in iter {
            df.vstack_mut(&chunk)?;
        }
        Ok(df)
    }

    /// 标签存在检查。
    pub async fn check_tagname_exist(&self, tag_name: &str) -> Result<bool> {
        let sql = format!(
            "SELECT COUNT(*) AS cnt FROM TagDatabase WHERE TagName = '{}'",
            tag_name
        );
        let df = query_df(self.connector.as_ref(), &sql).await?;
        let cnt = df
            .column("cnt")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string().parse::<i64>().unwrap_or(0))
            .unwrap_or(0);
        Ok(cnt > 0)
    }

    /// 更新 TagDatabase 值。
    pub async fn update_tagval(&self, tag_name: &str, tag_val: f64) -> Result<()> {
        let sql = format!(
            "MERGE TagDatabase AS t USING (SELECT '{name}' AS TagName) AS s \
             ON (t.TagName = s.TagName) \
             WHEN MATCHED THEN UPDATE SET TagVal = {val} \
             WHEN NOT MATCHED THEN INSERT (TagName, TagVal) VALUES ('{name}', {val});",
            name = tag_name,
            val = tag_val
        );
        execute_modify(self.connector.as_ref(), &sql).await?;
        Ok(())
    }

    /// 根据列名关键词猜测喂料列名。
    pub async fn get_feed_amount_column_name(&self, table_name: &str) -> Result<Vec<String>> {
        let sql = format!(
            "SELECT TOP (1) * FROM {table} WHERE DateTime BETWEEN DATEADD(hour, -2, GETDATE()) AND DATEADD(hour, 1, GETDATE()) ORDER BY DateTime DESC",
            table = table_name
        );
        let df = query_df(self.connector.as_ref(), &sql).await?;
        let keywords = ["喂料", "给料机", "给煤机"];
        let names: Vec<String> = df
            .get_column_names()
            .iter()
            .filter(|name| keywords.iter().any(|k| name.contains(k)))
            .map(|name| name.to_string())
            .collect();
        Ok(names)
    }

    /// 从 TagDatabase 读取设备运行状态 (1 运行, 0 停止)。
    pub async fn check_device_running_status(&self, var_name: &str) -> Result<bool> {
        let sql = format!(
            "SELECT TOP (1) TagVal FROM TagDatabase WHERE TagName = '{}' ORDER BY DataTime DESC",
            var_name
        );
        let df = query_df(self.connector.as_ref(), &sql).await?;
        let val = df
            .column("TagVal")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string().parse::<i64>().unwrap_or(0))
            .unwrap_or(0);
        Ok(val == 1)
    }

    /// 根据模型参数自动选择时间窗口。
    pub async fn get_latest_input_data_model_parameter(
        &self,
        input_name_list: &[&str],
        filter_length: &str,
        time_length: i64,
    ) -> Result<DataFrame> {
        if filter_length.to_ascii_lowercase().contains("min") {
            self.get_latest_input_data_by_column_time_min(input_name_list, time_length)
                .await
        } else {
            self.get_latest_input_data_by_column_time_hour(input_name_list, time_length)
                .await
        }
    }

    /// 从 TagDatabase 读取优化模式，未知时返回 "均衡"。
    pub async fn get_optimization_mode(&self, optimization_procedure: &str) -> Result<String> {
        let mode_sql = match optimization_procedure {
            "水泥磨" => {
                "SELECT TOP 1 TagName FROM TagDatabase WHERE TagName LIKE N'%水泥%' and (TagName LIKE N'%优先%' or TagName LIKE N'%均衡模式%') AND TagVal = 1 ORDER BY DataTime DESC"
            }
            "水泥A磨" => {
                "SELECT TOP 1 TagName FROM TagDatabase WHERE TagName LIKE N'%水泥A%' and (TagName LIKE N'%优先%' or TagName LIKE N'%均衡模式%') AND TagVal = 1 ORDER BY DataTime DESC"
            }
            "水泥B磨" => {
                "SELECT TOP 1 TagName FROM TagDatabase WHERE TagName LIKE N'%水泥B%' and (TagName LIKE N'%优先%' or TagName LIKE N'%均衡模式%') AND TagVal = 1 ORDER BY DataTime DESC"
            }
            "原料A磨" => {
                "SELECT TOP 1 TagName FROM TagDatabase WHERE (TagName LIKE N'%原料A%' or TagName LIKE N'%生料A%') and (TagName LIKE N'%优先%' or TagName LIKE N'%均衡模式%') AND TagVal = 1 ORDER BY DataTime DESC"
            }
            "原料B磨" => {
                "SELECT TOP 1 TagName FROM TagDatabase WHERE (TagName LIKE N'%原料B%' or TagName LIKE N'%生料B%') and (TagName LIKE N'%优先%' or TagName LIKE N'%均衡模式%') AND TagVal = 1 ORDER BY DataTime DESC"
            }
            "煤磨" => {
                "SELECT TOP 1 TagName FROM TagDatabase WHERE TagName LIKE N'%煤磨%' and (TagName LIKE N'%优先%' or TagName LIKE N'%均衡模式%') AND TagVal = 1 ORDER BY DataTime DESC"
            }
            "窑" => {
                "SELECT TOP 1 TagName FROM TagDatabase WHERE TagName LIKE N'%窑%' and (TagName LIKE N'%优先%' or TagName LIKE N'%均衡模式%') AND TagVal = 1 ORDER BY DataTime DESC"
            }
            _ => return Ok("均衡".to_string()),
        };

        let df = query_df(self.connector.as_ref(), mode_sql).await?;
        if df.is_empty() {
            return Ok("均衡".to_string());
        }

        let tag = df
            .column("TagName")
            .ok()
            .and_then(|c| c.get(0).ok())
            .map(|v| v.to_string())
            .unwrap_or_default();

        let mode = if tag.contains("均衡") {
            "均衡"
        } else if tag.contains("产量") {
            "产量"
        } else if tag.contains("能耗") {
            "能耗"
        } else if tag.contains("质量") {
            "质量"
        } else {
            "均衡"
        };

        Ok(mode.to_string())
    }

    /// 读取水泥磨目标 (TagDatabase 驱动)。
    pub async fn get_cement_mill_target(&self, optimization_procedure: &str) -> Result<(i64, f64)> {
        let cement_variety_sql = match optimization_procedure {
            "水泥磨" | "水泥A磨" => Some(
                "SELECT tagName FROM [TagDataBase] WHERE tagName IN (
                    '水泥APO425管装水泥',
                    '水泥APO425D',
                    '水泥APO425普通水泥',
                    '水泥APO525',
                    '水泥A道路水泥',
                    '水泥A铁标水泥',
                    'SA_325',
                    'SA_PO425',
                    'SA_PC425'
                ) AND tagVal = 1",
            ),
            "水泥B磨" => Some(
                "SELECT tagName FROM [TagDataBase] WHERE tagName IN (
                    '水泥BPO425管装水泥',
                    '水泥BPO425D',
                    '水泥BPO425普通水泥',
                    '水泥BPO525',
                    '水泥B道路水泥',
                    '水泥B铁标水泥',
                    'SB_325',
                    'SB_PO425',
                    'SB_PC425'
                ) AND tagVal = 1",
            ),
            _ => None,
        };

        if cement_variety_sql.is_none() {
            return Ok((350, 5.25));
        }

        let variety =
            query_first_tag_name(self.connector.as_ref(), cement_variety_sql.unwrap()).await?;
        let Some(variety) = variety else {
            return Ok((350, 5.25));
        };

        let tags = match variety.as_str() {
            "水泥APO425管装水泥" => Some((
                "水泥A磨品种1低限",
                "水泥A磨品种1高限",
                "水泥A磨品种细度1低限",
                "水泥A磨品种细度1高限",
            )),
            "水泥APO425D" => Some((
                "水泥A磨品种2低限",
                "水泥A磨品种2高限",
                "水泥A磨品种细度2低限",
                "水泥A磨品种细度2高限",
            )),
            "水泥APO425普通水泥" => Some((
                "水泥A磨品种3低限",
                "水泥A磨品种3高限",
                "水泥A磨品种细度3低限",
                "水泥A磨品种细度3高限",
            )),
            "水泥APO525" => Some((
                "水泥A磨品种4低限",
                "水泥A磨品种4高限",
                "水泥A磨品种细度4低限",
                "水泥A磨品种细度4高限",
            )),
            "水泥A道路水泥" => Some((
                "水泥A磨品种5低限",
                "水泥A磨品种5高限",
                "水泥A磨品种细度5低限",
                "水泥A磨品种细度5高限",
            )),
            "水泥A铁标水泥" => Some((
                "水泥A磨品种6低限",
                "水泥A磨品种6高限",
                "水泥A磨品种细度6低限",
                "水泥A磨品种细度6高限",
            )),
            "水泥BPO425管装水泥" => Some((
                "水泥B磨品种1低限",
                "水泥B磨品种1高限",
                "水泥B磨品种细度1低限",
                "水泥B磨品种细度1高限",
            )),
            "水泥BPO425D" => Some((
                "水泥B磨品种2低限",
                "水泥B磨品种2高限",
                "水泥B磨品种细度2低限",
                "水泥B磨品种细度2高限",
            )),
            "水泥BPO425普通水泥" => Some((
                "水泥B磨品种3低限",
                "水泥B磨品种3高限",
                "水泥B磨品种细度3低限",
                "水泥B磨品种细度3高限",
            )),
            "水泥BPO525" => Some((
                "水泥B磨品种4低限",
                "水泥B磨品种4高限",
                "水泥B磨品种细度4低限",
                "水泥B磨品种细度4高限",
            )),
            "水泥B道路水泥" => Some((
                "水泥B磨品种5低限",
                "水泥B磨品种5高限",
                "水泥B磨品种细度5低限",
                "水泥B磨品种细度5高限",
            )),
            "水泥B铁标水泥" => Some((
                "水泥B磨品种6低限",
                "水泥B磨品种6高限",
                "水泥B磨品种细度6低限",
                "水泥B磨品种细度6高限",
            )),
            _ => None,
        };

        let Some((lower_tag, upper_tag, fineness_low_tag, fineness_high_tag)) = tags else {
            return Ok((350, 5.25));
        };

        let lower = query_tag_val(self.connector.as_ref(), lower_tag).await?;
        let upper = query_tag_val(self.connector.as_ref(), upper_tag).await?;
        let fineness_lower = query_tag_val(self.connector.as_ref(), fineness_low_tag).await?;
        let fineness_upper = query_tag_val(self.connector.as_ref(), fineness_high_tag).await?;

        let surface_area_target = match (lower, upper) {
            (Some(l), Some(u)) => ((l + u) / 2.0).round() as i64,
            _ => 350,
        };
        let mut fineness_target = match (fineness_lower, fineness_upper) {
            (Some(l), Some(u)) => (l + u) / 2.0,
            _ => 5.25,
        };
        if fineness_target == 0.0 {
            fineness_target = 5.25;
        }

        if optimization_procedure.contains('A') {
            let _ = execute_modify(
                self.connector.as_ref(),
                &format!(
                    "UPDATE TagDatabase SET TagVal = {} WHERE TagName = '水泥A比表面积目标寄存'",
                    surface_area_target
                ),
            )
            .await;
        }

        Ok((surface_area_target, fineness_target))
    }

    pub async fn get_cement_mill_a_target(&self) -> Result<(i64, f64)> {
        self.get_cement_mill_target("水泥A磨").await
    }

    pub async fn get_cement_mill_b_target(&self) -> Result<(i64, f64)> {
        self.get_cement_mill_target("水泥B磨").await
    }

    /// 查询决策历史表，返回原始 DataFrame。
    pub async fn query_decision_history_data(
        &self,
        table_name: &str,
        time_query_minutes: i64,
    ) -> Result<DataFrame> {
        let rows = time_query_minutes * 6;
        let sql = format!(
            "SELECT TOP {rows} * FROM {table} WHERE DateTime BETWEEN SUBSTRING(CONVERT(varchar, DATEADD(hour, -2, GETDATE()), 120), 1, 16) AND SUBSTRING(CONVERT(varchar, DATEADD(hour, 1, GETDATE()), 120), 1, 16) ORDER BY DateTime DESC",
            rows = rows,
            table = table_name
        );
        let df = query_df(self.connector.as_ref(), &sql).await?;
        Ok(tail_df(df, time_query_minutes as usize))
    }

    /// 根据 TagName 列表查询决策历史（宽表转换）。
    pub async fn query_decision_history_data_by_name(
        &self,
        input_name_list: &[&str],
        time_query: i64,
    ) -> Result<DataFrame> {
        if input_name_list.is_empty() {
            return Ok(DataFrame::default());
        }
        let df = pivot_history_between_truncated(
            self.connector.as_ref(),
            input_name_list,
            "DATEADD(HOUR, -6, GETDATE())",
            "GETDATE()",
            TruncateUnit::Minute,
            None,
        )
        .await?;
        Ok(tail_df(df, time_query as usize))
    }

    /// 将决策结果写入 decision 表，按变量 upsert。
    pub async fn write_decision_result(
        &self,
        optimization_project_name: &str,
        optimization_name_list: &[&str],
        optimization_solution_results: &[f64],
        optimization_type: &str,
    ) -> Result<()> {
        let pairs = optimization_name_list
            .iter()
            .zip(optimization_solution_results.iter());
        for (name, value) in pairs {
            let rounded = (value * 100.0).round() / 100.0;
            let sql = format!(
                "MERGE decision AS t USING (SELECT '{proj}' AS optimization_project_name, '{var}' AS variable_name, '{otype}' AS optimization_type) AS s ON (t.optimization_project_name = s.optimization_project_name AND t.variable_name = s.variable_name AND t.optimization_type = s.optimization_type) WHEN MATCHED THEN UPDATE SET DateTime = GETDATE(), decision_value = {val} WHEN NOT MATCHED THEN INSERT (DateTime, optimization_project_name, variable_name, decision_value, optimization_type) VALUES (GETDATE(), '{proj}', '{var}', {val}, '{otype}');",
                proj = optimization_project_name,
                var = name,
                val = rounded,
                otype = optimization_type
            );
            execute_modify(self.connector.as_ref(), &sql).await?;
        }
        Ok(())
    }

    /// 历史表最近 4 小时数据（长表转宽表）。
    pub async fn query_history_data_latest(&self) -> Result<DataFrame> {
        let sql = "SELECT DateTime, TagName, TagVal FROM 历史表 WITH (NOLOCK) WHERE DateTime >= DATEADD(HOUR, -4, GETDATE()) AND DateTime <= GETDATE() ORDER BY DateTime";
        query_df(self.connector.as_ref(), sql).await
    }

    /// 按名称列表查询最近历史数据并宽表化。
    pub async fn query_decision_history_data_latest_by_name_list(
        &self,
        input_name_list: &[&str],
    ) -> Result<DataFrame> {
        if input_name_list.is_empty() {
            return Ok(DataFrame::default());
        }
        pivot_history_between(
            self.connector.as_ref(),
            input_name_list,
            "DATEADD(HOUR, -6, GETDATE())",
            "GETDATE()",
            None,
        )
        .await
    }

    /// 查询 TagDatabase 全量最新数据。
    pub async fn query_tagdatabase_latest(&self) -> Result<DataFrame> {
        let sql = "SELECT TagName, TagVal FROM TagDatabase WITH (NOLOCK)";
        let df = query_df(self.connector.as_ref(), sql).await?;
        if df.is_empty() {
            return Ok(DataFrame::default());
        }
        let names = df
            .column("TagName")
            .ok()
            .and_then(|c| c.str().ok())
            .map(|s| s.into_iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let vals = df
            .column("TagVal")
            .ok()
            .and_then(|c| c.f64().ok())
            .map(|s| s.into_iter().collect::<Vec<_>>())
            .unwrap_or_default();

        let mut columns: Vec<Series> = Vec::new();
        let ts = Series::new("DateTime".into(), &[Utc::now().timestamp_millis()]);
        columns.push(
            ts.cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
                .map_err(IndustryDbError::from)?,
        );

        for (name_opt, val_opt) in names.iter().zip(vals.iter()) {
            if let Some(name) = name_opt {
                let cname: String = name.to_string();
                let series = match val_opt {
                    Some(v) => Series::new(cname.as_str().into(), &[Some(*v)]),
                    None => Series::new(cname.as_str().into(), &[Option::<f64>::None]),
                };
                columns.push(series);
            }
        }

        let cols: Vec<Column> = columns.into_iter().map(|s| s.into_column()).collect();
        DataFrame::new(cols).map_err(IndustryDbError::from)
    }

    /// 查询 TagDatabase 指定 TagName。
    pub async fn query_tagdatabase_latest_by_name_list(
        &self,
        input_name_list: &[&str],
    ) -> Result<DataFrame> {
        if input_name_list.is_empty() {
            return Ok(DataFrame::default());
        }
        pivot_tagdb(self.connector.as_ref(), input_name_list).await
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
                );
            END
        "#;
        execute_modify(self.connector.as_ref(), sql).await?;
        Ok(())
    }

    /// 查询决策实时解并转宽表。
    pub async fn query_decision_solution_rt_by_column(
        &self,
        column_name_list: &[&str],
        optimization_type: &str,
    ) -> Result<DataFrame> {
        if column_name_list.is_empty() {
            return Ok(DataFrame::default());
        }
        let sql = format!(
            "SELECT DateTime, variable_name, decision_value FROM decision WITH (NOLOCK) WHERE optimization_type = '{otype}' AND variable_name IN ('{vars}') AND DateTime BETWEEN DATEADD(HOUR, -6, GETDATE()) AND GETDATE() ORDER BY DateTime",
            otype = optimization_type,
            vars = column_name_list.join("','"),
        );
        let df = query_df(self.connector.as_ref(), &sql).await?;
        if df.is_empty() {
            return Ok(df);
        }
        let mut wide =
            pivot_decision(self.connector.as_ref(), column_name_list, optimization_type).await?;
        sanitize_columns(&mut wide)?;
        if wide.is_empty() {
            return Ok(wide);
        }
        let unique = wide.column("DateTime")?.unique()?.len();
        if unique != 1 {
            return Err(IndustryDbError::InvalidParameter(format!(
                "决策时间戳不一致: {unique} 个"
            )));
        }

        let dt_val = wide
            .column("DateTime")?
            .get(wide.height().saturating_sub(1))
            .unwrap_or(AnyValue::Null);
        let chrono_dt = match dt_val {
            AnyValue::Datetime(v, tu, _) => {
                let nanos = match tu {
                    TimeUnit::Nanoseconds => v,
                    TimeUnit::Microseconds => v * 1_000,
                    TimeUnit::Milliseconds => v * 1_000_000,
                };
                let secs = nanos / 1_000_000_000;
                let sub_nanos = (nanos % 1_000_000_000) as u32;
                Utc.timestamp_opt(secs, sub_nanos).single()
            }
            _ => None,
        };

        if let Some(dt) = chrono_dt {
            let now = Utc::now();
            if now - dt > ChronoDuration::minutes(10) {
                return Err(IndustryDbError::InvalidParameter(format!(
                    "决策时间 {dt} 超过 10 分钟未更新"
                )));
            }
        }

        Ok(wide)
    }
}

/// 简单的标识符转义，防止列名中包含右括号导致 SQL 失败。
fn escape_ident(name: &str) -> String {
    name.replace(']', "]]")
}

/// 构造对历史表(TagName/TagVal)的 PIVOT 查询并执行。
async fn pivot_history_between_truncated(
    connector: &MssqlConnector,
    name_list: &[&str],
    start_expr: &str,
    end_expr: &str,
    truncate: TruncateUnit,
    top: Option<i64>,
) -> Result<DataFrame> {
    if name_list.is_empty() {
        return Ok(DataFrame::default());
    }

    let cols: Vec<String> = name_list
        .iter()
        .map(|c| format!("[{}]", escape_ident(c)))
        .collect();
    let top_clause = top.map(|t| format!("TOP {} ", t)).unwrap_or_default();

    let sql = format!(
        "SELECT {top} DateTime, {cols} FROM (SELECT {date_expr} AS DateTime, TagName, TagVal FROM [历史表] WHERE DateTime BETWEEN {start} AND {end} AND TagName IN ('{names}')) AS src PIVOT (AVG(TagVal) FOR TagName IN ({cols})) AS pvt ORDER BY DateTime",
        top = top_clause,
        cols = cols.join(", "),
        names = name_list.join("','"),
        start = start_expr,
        end = end_expr,
        date_expr = truncate.sql_expr(),
    );

    query_df(connector, &sql).await
}

async fn pivot_history_between(
    connector: &MssqlConnector,
    name_list: &[&str],
    start_expr: &str,
    end_expr: &str,
    top: Option<i64>,
) -> Result<DataFrame> {
    pivot_history_between_truncated(
        connector,
        name_list,
        start_expr,
        end_expr,
        TruncateUnit::None,
        top,
    )
    .await
}

/// 对 TagDatabase 进行 PIVOT 查询。
async fn pivot_tagdb(connector: &MssqlConnector, name_list: &[&str]) -> Result<DataFrame> {
    if name_list.is_empty() {
        return Ok(DataFrame::default());
    }
    let cols: Vec<String> = name_list
        .iter()
        .map(|c| format!("[{}]", escape_ident(c)))
        .collect();
    let sql = format!(
        "SELECT DateTime, {cols} FROM (SELECT GETDATE() AS DateTime, TagName, TagVal FROM TagDatabase WITH (NOLOCK) WHERE TagName IN ('{names}')) AS src PIVOT (AVG(TagVal) FOR TagName IN ({cols})) AS pvt",
        cols = cols.join(", "),
        names = name_list.join("','"),
    );
    query_df(connector, &sql).await
}

/// 对 decision 表按 variable_name 进行 PIVOT 查询。
async fn pivot_decision(
    connector: &MssqlConnector,
    column_name_list: &[&str],
    optimization_type: &str,
) -> Result<DataFrame> {
    if column_name_list.is_empty() {
        return Ok(DataFrame::default());
    }
    let cols: Vec<String> = column_name_list
        .iter()
        .map(|c| format!("[{}]", escape_ident(c)))
        .collect();
    let sql = format!(
        "SELECT DateTime, {cols} FROM (SELECT DateTime, variable_name, decision_value FROM decision WITH (NOLOCK) WHERE optimization_type = '{otype}' AND variable_name IN ('{names}') AND DateTime BETWEEN DATEADD(HOUR, -6, GETDATE()) AND GETDATE()) AS src PIVOT (AVG(decision_value) FOR variable_name IN ({cols})) AS pvt ORDER BY DateTime",
        cols = cols.join(", "),
        names = column_name_list.join("','"),
        otype = optimization_type,
    );
    query_df(connector, &sql).await
}
