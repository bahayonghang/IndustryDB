//! Python configuration bindings

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use std::collections::HashMap;

use crate::errors::{to_py_err, to_py_result};
use industrydb_core::config::{ConnectionConfig as CoreConnectionConfig, DatabaseType};

/// Python-exposed database configuration
#[pyclass(name = "PyDatabaseConfig")]
#[derive(Clone)]
pub struct PyDatabaseConfig {
    inner: CoreConnectionConfig,
}

#[pymethods]
impl PyDatabaseConfig {
    /// Create a new database configuration
    #[new]
    #[pyo3(signature = (db_type, host=None, port=None, database=None, username=None, password=None, path=None, server=None, **kwargs))]
    fn new(
        db_type: String,
        host: Option<String>,
        port: Option<u16>,
        database: Option<String>,
        username: Option<String>,
        password: Option<String>,
        path: Option<String>,
        server: Option<String>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let db_type_enum: DatabaseType = db_type.parse().map_err(to_py_err)?;

        let mut config = CoreConnectionConfig {
            db_type: db_type_enum,
            host,
            port,
            database,
            username,
            password,
            server,
            path,
            trusted_connection: None,
            timeout: None,
            extra: HashMap::new(),
        };

        // Process kwargs if any
        if let Some(dict) = kwargs {
            for (key, value) in dict.iter() {
                let key_str: String = key.extract()?;
                let value_json = pythonize::depythonize_bound(value).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Failed to convert kwarg '{}': {}",
                        key_str, e
                    ))
                })?;
                config.extra.insert(key_str, value_json);
            }
        }

        config.validate().map_err(to_py_err)?;

        Ok(PyDatabaseConfig { inner: config })
    }

    /// Create from dictionary
    #[staticmethod]
    fn from_dict(config: &Bound<'_, PyDict>) -> PyResult<Self> {
        let db_type: String = config
            .get_item("type")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing 'type' field"))?
            .extract()?;

        let host: Option<String> = config.get_item("host")?.and_then(|v| v.extract().ok());
        let port: Option<u16> = config.get_item("port")?.and_then(|v| v.extract().ok());
        let database: Option<String> = config.get_item("database")?.and_then(|v| v.extract().ok());
        let username: Option<String> = config.get_item("username")?.and_then(|v| v.extract().ok());
        let password: Option<String> = config.get_item("password")?.and_then(|v| v.extract().ok());
        let path: Option<String> = config.get_item("path")?.and_then(|v| v.extract().ok());
        let server: Option<String> = config.get_item("server")?.and_then(|v| v.extract().ok());

        Self::new(
            db_type,
            host,
            port,
            database,
            username,
            password,
            path,
            server,
            Some(config),
        )
    }

    /// Create from URI
    #[staticmethod]
    fn from_uri(uri: String) -> PyResult<Self> {
        let config = CoreConnectionConfig::from_uri(&uri).map_err(to_py_err)?;
        Ok(PyDatabaseConfig { inner: config })
    }

    /// Convert to dictionary
    fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new_bound(py);

        dict.set_item("type", self.inner.db_type.to_string())?;

        if let Some(ref host) = self.inner.host {
            dict.set_item("host", host)?;
        }
        if let Some(port) = self.inner.port {
            dict.set_item("port", port)?;
        }
        if let Some(ref database) = self.inner.database {
            dict.set_item("database", database)?;
        }
        if let Some(ref username) = self.inner.username {
            dict.set_item("username", username)?;
        }
        if let Some(ref password) = self.inner.password {
            dict.set_item("password", password)?;
        }
        if let Some(ref path) = self.inner.path {
            dict.set_item("path", path)?;
        }
        if let Some(ref server) = self.inner.server {
            dict.set_item("server", server)?;
        }

        Ok(dict.into())
    }

    /// Convert to URI string
    fn to_uri(&self) -> PyResult<String> {
        to_py_result(self.inner.to_uri())
    }

    /// Get database type
    #[getter]
    fn db_type(&self) -> String {
        self.inner.db_type.to_string()
    }

    fn __repr__(&self) -> String {
        format!("DatabaseConfig(type='{}', ...)", self.inner.db_type)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl PyDatabaseConfig {
    /// Get reference to inner config
    pub fn inner(&self) -> &CoreConnectionConfig {
        &self.inner
    }

    /// Take inner config
    pub fn into_inner(self) -> CoreConnectionConfig {
        self.inner
    }
}
