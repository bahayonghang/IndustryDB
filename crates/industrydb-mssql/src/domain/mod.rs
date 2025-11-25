//! Domain-specific MSSQL clients and factory routing.
//!
//! These clients wrap `MssqlConnector` to provide higher-level operations
//! equivalent to the reference Python interfaces. Implementations are kept
//! lean and reuse shared helpers for query/execute paths.

use std::sync::Arc;

use industrydb_core::{
    config::ConnectionConfig,
    error::{IndustryDbError, Result},
    traits::DatabaseConnector,
};
use polars::prelude::DataFrame;

use crate::connector::MssqlConnector;

mod decision_history;
mod decision_making;
mod helpers;
mod model_status;
mod operation;
mod quality;
mod rt_predict;
mod rt_predict_alter;
mod rt_predict_sequence;
mod timeseries;
mod web;

pub use decision_history::DecisionHistoryClient;
pub use decision_making::DecisionMakingClient;
pub use model_status::ModelStatusClient;
pub use operation::OperationClient;
pub use quality::QualityClient;
pub use rt_predict::RealtimePredictClient;
pub use rt_predict_alter::RealtimePredictAlterClient;
pub use rt_predict_sequence::RealtimePredictSequenceClient;
#[cfg(test)]
mod tests;
pub use timeseries::TimeSeriesClient;
pub use web::{OnlineTrainerClient, WebClient};

/// Supported MSSQL domain targets.
pub const SUPPORTED_TARGETS: &[&str] = &[
    "base",
    "web",
    "timeseries",
    "quality",
    "online_trainer",
    "realtime_predict",
    "realtime_predict_alter",
    "realtime_predict_sequence",
    "decision_history",
    "decision_making",
    "operation",
    "model_status",
];

/// Domain clients enumeration for factory return.
pub enum DomainClient {
    Web(WebClient),
    TimeSeries(TimeSeriesClient),
    Quality(QualityClient),
    OnlineTrainer(OnlineTrainerClient),
    RealtimePredict(RealtimePredictClient),
    RealtimePredictAlter(RealtimePredictAlterClient),
    RealtimePredictSequence(RealtimePredictSequenceClient),
    DecisionHistory(DecisionHistoryClient),
    DecisionMaking(DecisionMakingClient),
    Operation(OperationClient),
    ModelStatus(ModelStatusClient),
    Base(Arc<MssqlConnector>),
}

impl DomainClient {
    /// Return the underlying connector for base operations when target is `base`.
    pub fn as_base(&self) -> Option<&Arc<MssqlConnector>> {
        if let DomainClient::Base(conn) = self {
            Some(conn)
        } else {
            None
        }
    }
}

/// Factory to build domain-specific clients backed by a shared connector.
pub struct DomainFactory;

impl DomainFactory {
    /// Create a shared MSSQL connector.
    pub async fn create_connector(config: &ConnectionConfig) -> Result<Arc<MssqlConnector>> {
        let connector = MssqlConnector::new(config).await?;
        Ok(Arc::new(connector))
    }

    /// Create a domain client by target.
    pub async fn create(config: &ConnectionConfig, target: &str) -> Result<DomainClient> {
        if !SUPPORTED_TARGETS.contains(&target) {
            return Err(IndustryDbError::config_error(format!(
                "Unsupported MSSQL target: {}. Allowed: {}",
                target,
                SUPPORTED_TARGETS.join(", ")
            )));
        }

        let connector = Self::create_connector(config).await?;

        match target {
            "web" => Ok(DomainClient::Web(WebClient::new(connector.clone()))),
            "timeseries" => Ok(DomainClient::TimeSeries(TimeSeriesClient::new(
                connector.clone(),
            ))),
            "quality" => Ok(DomainClient::Quality(QualityClient::new(connector.clone()))),
            "online_trainer" => Ok(DomainClient::OnlineTrainer(OnlineTrainerClient::new(
                connector.clone(),
            ))),
            "realtime_predict" => Ok(DomainClient::RealtimePredict(RealtimePredictClient::new(
                connector.clone(),
            ))),
            "realtime_predict_alter" => Ok(DomainClient::RealtimePredictAlter(
                RealtimePredictAlterClient::new(connector.clone()),
            )),
            "realtime_predict_sequence" => Ok(DomainClient::RealtimePredictSequence(
                RealtimePredictSequenceClient::new(connector.clone()),
            )),
            "decision_history" => Ok(DomainClient::DecisionHistory(DecisionHistoryClient::new(
                connector.clone(),
            ))),
            "decision_making" => Ok(DomainClient::DecisionMaking(DecisionMakingClient::new(
                connector.clone(),
            ))),
            "operation" => Ok(DomainClient::Operation(OperationClient::new(
                connector.clone(),
            ))),
            "model_status" => Ok(DomainClient::ModelStatus(ModelStatusClient::new(
                connector.clone(),
            ))),
            "base" => Ok(DomainClient::Base(connector)),
            _ => Err(IndustryDbError::config_error(format!(
                "Unsupported MSSQL target: {}",
                target
            ))),
        }
    }
}

/// Convenience helpers for select/update/execute operations.
pub(crate) async fn query_df(connector: &MssqlConnector, sql: &str) -> Result<DataFrame> {
    connector.execute(sql).await
}

pub(crate) async fn execute_modify(connector: &MssqlConnector, sql: &str) -> Result<u64> {
    helpers::execute_rows_affected(connector, sql).await
}
