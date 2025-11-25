#![allow(unsafe_op_in_unsafe_fn)]
//! IndustryDB Python Bindings
//!
//! High-performance database middleware for Python, powered by Rust and Polars.

use pyo3::{prelude::*, wrap_pyfunction};

mod config;
mod connection;
mod domain;
mod errors;

use config::PyDatabaseConfig;
use connection::PyConnection;
use domain::*;

/// IndustryDB - High-performance database middleware
#[pymodule]
fn industrydb(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "IndustryDB Contributors")?;

    // Classes
    m.add_class::<PyDatabaseConfig>()?;
    m.add_class::<PyConnection>()?;
    m.add_class::<PyWebClient>()?;
    m.add_class::<PyQualityClient>()?;
    m.add_class::<PyTimeSeriesClient>()?;
    m.add_class::<PyRealtimePredictClient>()?;
    m.add_class::<PyRealtimePredictAlterClient>()?;
    m.add_class::<PyRealtimePredictSequenceClient>()?;
    m.add_class::<PyDecisionHistoryClient>()?;
    m.add_class::<PyDecisionMakingClient>()?;
    m.add_class::<PyOperationClient>()?;
    m.add_class::<PyModelStatusClient>()?;

    // Exceptions
    m.add(
        "IndustryDbError",
        py.get_type_bound::<errors::IndustryDbError>(),
    )?;
    m.add(
        "DatabaseConnectionError",
        py.get_type_bound::<errors::DatabaseConnectionError>(),
    )?;
    m.add(
        "QueryExecutionError",
        py.get_type_bound::<errors::QueryExecutionError>(),
    )?;
    m.add(
        "ConfigurationError",
        py.get_type_bound::<errors::ConfigurationError>(),
    )?;
    m.add(
        "ConnectionClosedError",
        py.get_type_bound::<errors::ConnectionClosedError>(),
    )?;
    m.add(
        "ConstraintViolationError",
        py.get_type_bound::<errors::ConstraintViolationError>(),
    )?;

    // Factory helpers
    m.add_function(wrap_pyfunction!(create_web_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_quality_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_timeseries_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_realtime_predict_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_realtime_predict_alter_client, m)?)?;
    m.add_function(wrap_pyfunction!(
        create_realtime_predict_sequence_client,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(create_decision_history_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_decision_making_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_operation_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_model_status_client, m)?)?;
    m.add_function(wrap_pyfunction!(create_online_trainer_client, m)?)?;

    Ok(())
}
