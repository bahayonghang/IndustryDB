//! Python exception types

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

use industrydb_core::error::IndustryDbError as CoreError;

// Create custom exception types
create_exception!(industrydb, IndustryDbError, PyException);
create_exception!(industrydb, DatabaseConnectionError, IndustryDbError);
create_exception!(industrydb, QueryExecutionError, IndustryDbError);
create_exception!(industrydb, ConfigurationError, IndustryDbError);
create_exception!(industrydb, ConnectionClosedError, IndustryDbError);
create_exception!(industrydb, ConstraintViolationError, IndustryDbError);

/// Convert core errors to Python exceptions
pub fn to_py_err(err: CoreError) -> PyErr {
    match err {
        CoreError::ConnectionError(msg) => PyErr::new::<DatabaseConnectionError, _>(msg),
        CoreError::QueryError(msg) => PyErr::new::<QueryExecutionError, _>(msg),
        CoreError::ConfigError(msg) => PyErr::new::<ConfigurationError, _>(msg),
        CoreError::ConnectionClosed => {
            PyErr::new::<ConnectionClosedError, _>("Connection is closed")
        }
        CoreError::ConstraintViolation(msg) => PyErr::new::<ConstraintViolationError, _>(msg),
        CoreError::InvalidParameter(msg) => {
            PyErr::new::<IndustryDbError, _>(format!("Invalid parameter: {}", msg))
        }
        _ => PyErr::new::<IndustryDbError, _>(err.to_string()),
    }
}

/// Result type for Python operations
pub type PyResult<T> = Result<T, PyErr>;

/// Convert core Result to PyResult
pub fn to_py_result<T>(result: Result<T, CoreError>) -> PyResult<T> {
    result.map_err(to_py_err)
}
