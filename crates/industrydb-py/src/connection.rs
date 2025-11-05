//! Python connection bindings

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::config::PyDatabaseConfig;
use crate::errors::to_py_err;
use industrydb_core::{
    config::{ConnectionConfig, DatabaseType},
    traits::CrudOperations,
};

/// Python-exposed database connection
#[pyclass(name = "PyConnection")]
pub struct PyConnection {
    inner: Option<Box<dyn CrudOperations>>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl PyConnection {
    /// Create a new connection
    #[new]
    fn new(config: &PyDatabaseConfig) -> PyResult<Self> {
        let runtime = Arc::new(Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create runtime: {}",
                e
            ))
        })?);

        let connector = runtime
            .block_on(create_connector(config.inner()))
            .map_err(to_py_err)?;

        Ok(PyConnection {
            inner: Some(connector),
            runtime,
        })
    }

    /// Connect to database
    #[staticmethod]
    fn connect(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new(config)
    }

    /// Connect from URI
    #[staticmethod]
    fn from_uri(uri: String) -> PyResult<Self> {
        let config = ConnectionConfig::from_uri(&uri).map_err(to_py_err)?;
        let runtime = Arc::new(Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create runtime: {}",
                e
            ))
        })?);

        let connector = runtime
            .block_on(create_connector(&config))
            .map_err(to_py_err)?;

        Ok(PyConnection {
            inner: Some(connector),
            runtime,
        })
    }

    /// Close the connection
    fn close(&mut self) -> PyResult<()> {
        if let Some(mut conn) = self.inner.take() {
            self.runtime.block_on(conn.close()).map_err(to_py_err)?;
        }
        Ok(())
    }

    /// Check if connection is closed
    fn is_closed(&self) -> bool {
        self.inner.as_ref().map(|c| c.is_closed()).unwrap_or(true)
    }

    /// Execute SQL query
    #[pyo3(signature = (sql, params=None))]
    fn execute(
        &self,
        py: Python,
        sql: String,
        params: Option<&Bound<'_, PyList>>,
    ) -> PyResult<Py<PyDict>> {
        let conn = self.inner.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Connection is closed")
        })?;

        // TODO: Implement parameter binding
        let _ = params;

        let df = self
            .runtime
            .block_on(conn.execute(&sql))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    /// Insert data into table
    #[pyo3(signature = (table, data, **_kwargs))]
    fn insert(
        &self,
        table: String,
        data: &Bound<'_, PyDict>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<usize> {
        let conn = self.inner.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Connection is closed")
        })?;

        let df = py_dict_to_dataframe(data)?;
        let rows = self
            .runtime
            .block_on(conn.insert(&table, df))
            .map_err(to_py_err)?;
        Ok(rows)
    }

    /// Select data from table
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (table, columns=None, where_clause=None, params=None, limit=None, **_kwargs))]
    fn select(
        &self,
        py: Python,
        table: String,
        columns: Option<Vec<String>>,
        where_clause: Option<String>,
        params: Option<&Bound<'_, PyList>>,
        limit: Option<usize>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyDict>> {
        let conn = self.inner.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Connection is closed")
        })?;

        let _ = params;

        let df = self
            .runtime
            .block_on(conn.select(&table, columns.as_deref(), where_clause.as_deref(), limit))
            .map_err(to_py_err)?;

        dataframe_to_py_dict(py, &df)
    }

    /// Update rows in table
    #[pyo3(signature = (table, values, where_clause=None, params=None, **_kwargs))]
    fn update(
        &self,
        table: String,
        values: &Bound<'_, PyDict>,
        where_clause: Option<String>,
        params: Option<&Bound<'_, PyList>>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<usize> {
        let conn = self.inner.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Connection is closed")
        })?;

        let mut values_map = HashMap::new();
        for (key, value) in values.iter() {
            let key_str: String = key.extract()?;
            let value_str: String = value.str()?.extract()?;
            values_map.insert(key_str, value_str);
        }

        let _ = params;

        let rows = self
            .runtime
            .block_on(conn.update(&table, &values_map, where_clause.as_deref()))
            .map_err(to_py_err)?;

        Ok(rows)
    }

    /// Delete rows from table
    #[pyo3(signature = (table, where_clause=None, params=None, **_kwargs))]
    fn delete(
        &self,
        table: String,
        where_clause: Option<String>,
        params: Option<&Bound<'_, PyList>>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<usize> {
        let conn = self.inner.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Connection is closed")
        })?;

        let _ = params;

        let rows = self
            .runtime
            .block_on(conn.delete(&table, where_clause.as_deref()))
            .map_err(to_py_err)?;

        Ok(rows)
    }

    /// Context manager entry
    fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Context manager exit
    fn __exit__(
        &mut self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)
    }

    fn __repr__(&self) -> String {
        if self.is_closed() {
            "Connection(closed)".to_string()
        } else {
            "Connection(active)".to_string()
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl Drop for PyConnection {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Factory function to create the appropriate connector
async fn create_connector(
    config: &ConnectionConfig,
) -> Result<Box<dyn CrudOperations>, industrydb_core::error::IndustryDbError> {
    match config.db_type {
        DatabaseType::Postgres => {
            let connector = industrydb_postgres::PostgresConnector::new(config).await?;
            Ok(Box::new(connector))
        }
        DatabaseType::Sqlite => {
            let connector = industrydb_sqlite::SqliteConnector::new(config).await?;
            Ok(Box::new(connector))
        }
        DatabaseType::Mssql => {
            let connector = industrydb_mssql::MssqlConnector::new(config).await?;
            Ok(Box::new(connector))
        }
    }
}

/// Convert Polars DataFrame to Python dict
fn dataframe_to_py_dict(py: Python, df: &polars::prelude::DataFrame) -> PyResult<Py<PyDict>> {
    use polars::prelude::*;

    let dict = PyDict::new_bound(py);

    for col in df.get_columns() {
        let col_name = col.name().as_str();
        let values = PyList::empty_bound(py);

        // Convert column to PyList based on dtype
        for i in 0..col.len() {
            if col.is_null().get(i).unwrap_or(false) {
                values.append(py.None())?;
            } else {
                match col.dtype() {
                    DataType::Int32 => {
                        let val = col
                            .i32()
                            .map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                            })?
                            .get(i);
                        values.append(val)?;
                    }
                    DataType::Int64 => {
                        let val = col
                            .i64()
                            .map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                            })?
                            .get(i);
                        values.append(val)?;
                    }
                    DataType::Float64 => {
                        let val = col
                            .f64()
                            .map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                            })?
                            .get(i);
                        values.append(val)?;
                    }
                    DataType::String => {
                        let val = col
                            .str()
                            .map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                            })?
                            .get(i)
                            .unwrap_or("");
                        values.append(val)?;
                    }
                    DataType::Boolean => {
                        let val = col
                            .bool()
                            .map_err(|e| {
                                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                            })?
                            .get(i);
                        values.append(val)?;
                    }
                    _ => {
                        // Fallback to string representation
                        values.append(format!(
                            "{:?}",
                            col.get(i)
                                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                                    e.to_string()
                                ))?
                        ))?;
                    }
                }
            }
        }

        dict.set_item(col_name, values)?;
    }

    Ok(dict.unbind())
}

/// Convert Python dict to Polars DataFrame
fn py_dict_to_dataframe(data: &Bound<'_, PyDict>) -> PyResult<polars::prelude::DataFrame> {
    use polars::prelude::*;

    let mut series_vec: Vec<Series> = Vec::new();

    for (key, value) in data.iter() {
        let col_name: String = key.extract()?;
        let list: &Bound<'_, PyList> = value.downcast()?;

        // Try to infer type from first non-null value
        let mut values_i64: Vec<Option<i64>> = Vec::new();
        let mut values_f64: Vec<Option<f64>> = Vec::new();
        let mut values_str: Vec<Option<String>> = Vec::new();
        let mut is_int = true;
        let mut is_float = true;

        for item in list.iter() {
            if item.is_none() {
                values_i64.push(None);
                values_f64.push(None);
                values_str.push(None);
            } else if let Ok(val) = item.extract::<i64>() {
                values_i64.push(Some(val));
                values_f64.push(Some(val as f64));
                values_str.push(Some(val.to_string()));
            } else if let Ok(val) = item.extract::<f64>() {
                is_int = false;
                values_f64.push(Some(val));
                values_str.push(Some(val.to_string()));
            } else if let Ok(val) = item.extract::<String>() {
                is_int = false;
                is_float = false;
                values_str.push(Some(val));
            } else {
                is_int = false;
                is_float = false;
                values_str.push(Some(item.str()?.extract()?));
            }
        }

        let series = if is_int {
            Series::new(col_name.as_str().into(), values_i64)
        } else if is_float {
            Series::new(col_name.as_str().into(), values_f64)
        } else {
            Series::new(col_name.as_str().into(), values_str)
        };

        series_vec.push(series);
    }

    DataFrame::new(series_vec.into_iter().map(|s| s.into_column()).collect())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}
