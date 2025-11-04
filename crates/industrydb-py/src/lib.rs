//! IndustryDB Python Bindings
//!
//! High-performance database middleware for Python, powered by Rust and Polars.

use pyo3::prelude::*;

mod config;
mod connection;
mod errors;

use config::PyDatabaseConfig;
use connection::PyConnection;

/// IndustryDB - High-performance database middleware
#[pymodule]
fn industrydb(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", "IndustryDB Contributors")?;

    // Classes
    m.add_class::<PyDatabaseConfig>()?;
    m.add_class::<PyConnection>()?;

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

    Ok(())
}
