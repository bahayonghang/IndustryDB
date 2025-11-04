//! Configuration types and parsing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{IndustryDbError, Result};

/// Database type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    /// PostgreSQL database
    #[serde(alias = "postgresql")]
    Postgres,
    /// SQLite database
    Sqlite,
    /// Microsoft SQL Server
    #[serde(alias = "sqlserver")]
    Mssql,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::Postgres => write!(f, "postgres"),
            DatabaseType::Sqlite => write!(f, "sqlite"),
            DatabaseType::Mssql => write!(f, "mssql"),
        }
    }
}

impl std::str::FromStr for DatabaseType {
    type Err = IndustryDbError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "postgres" | "postgresql" => Ok(DatabaseType::Postgres),
            "sqlite" => Ok(DatabaseType::Sqlite),
            "mssql" | "sqlserver" => Ok(DatabaseType::Mssql),
            _ => Err(IndustryDbError::UnsupportedDatabase(s.to_string())),
        }
    }
}

/// Connection configuration for a single database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Database type
    #[serde(rename = "type")]
    pub db_type: DatabaseType,

    /// Host address (for Postgres/MSSQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Port number (for Postgres/MSSQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Database name (for Postgres/MSSQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,

    /// Username (for Postgres/MSSQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password (for Postgres/MSSQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Server address (alternative for MSSQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,

    /// File path (for SQLite)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Use Windows authentication (for MSSQL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trusted_connection: Option<bool>,

    /// Connection timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,

    /// Additional connection options
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl ConnectionConfig {
    /// Create a new PostgreSQL configuration
    pub fn postgres(
        host: String,
        port: u16,
        database: String,
        username: String,
        password: String,
    ) -> Self {
        Self {
            db_type: DatabaseType::Postgres,
            host: Some(host),
            port: Some(port),
            database: Some(database),
            username: Some(username),
            password: Some(password),
            server: None,
            path: None,
            trusted_connection: None,
            timeout: None,
            extra: HashMap::new(),
        }
    }

    /// Create a new SQLite configuration
    pub fn sqlite<P: AsRef<Path>>(path: P) -> Self {
        Self {
            db_type: DatabaseType::Sqlite,
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            server: None,
            path: Some(path.as_ref().to_string_lossy().to_string()),
            trusted_connection: None,
            timeout: None,
            extra: HashMap::new(),
        }
    }

    /// Create a new MSSQL configuration
    pub fn mssql(server: String, database: String, username: String, password: String) -> Self {
        Self {
            db_type: DatabaseType::Mssql,
            host: None,
            port: None,
            database: Some(database),
            username: Some(username),
            password: Some(password),
            server: Some(server),
            path: None,
            trusted_connection: None,
            timeout: None,
            extra: HashMap::new(),
        }
    }

    /// Build a connection URI string
    pub fn to_uri(&self) -> Result<String> {
        match self.db_type {
            DatabaseType::Postgres => {
                let host = self
                    .host
                    .as_ref()
                    .ok_or_else(|| IndustryDbError::config_error("Missing host for Postgres"))?;
                let port = self.port.unwrap_or(5432);
                let database = self.database.as_ref().ok_or_else(|| {
                    IndustryDbError::config_error("Missing database for Postgres")
                })?;
                let username = self.username.as_ref().ok_or_else(|| {
                    IndustryDbError::config_error("Missing username for Postgres")
                })?;
                let password = self.password.as_ref().ok_or_else(|| {
                    IndustryDbError::config_error("Missing password for Postgres")
                })?;

                Ok(format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    username, password, host, port, database
                ))
            }
            DatabaseType::Sqlite => {
                let path = self
                    .path
                    .as_ref()
                    .ok_or_else(|| IndustryDbError::config_error("Missing path for SQLite"))?;
                Ok(format!("sqlite://{}", path))
            }
            DatabaseType::Mssql => {
                let server = self
                    .server
                    .as_ref()
                    .or(self.host.as_ref())
                    .ok_or_else(|| IndustryDbError::config_error("Missing server for MSSQL"))?;
                let database = self
                    .database
                    .as_ref()
                    .ok_or_else(|| IndustryDbError::config_error("Missing database for MSSQL"))?;

                if self.trusted_connection.unwrap_or(false) {
                    Ok(format!(
                        "mssql://{}/?database={}&trusted_connection=true",
                        server, database
                    ))
                } else {
                    let username = self.username.as_ref().ok_or_else(|| {
                        IndustryDbError::config_error("Missing username for MSSQL")
                    })?;
                    let password = self.password.as_ref().ok_or_else(|| {
                        IndustryDbError::config_error("Missing password for MSSQL")
                    })?;
                    Ok(format!(
                        "mssql://{}:{}@{}/?database={}",
                        username, password, server, database
                    ))
                }
            }
        }
    }

    /// Parse a connection URI string
    pub fn from_uri(uri: &str) -> Result<Self> {
        // Basic URI parsing - in production use a proper URI parser
        if uri.starts_with("postgresql://") || uri.starts_with("postgres://") {
            // Parse postgres URI
            let uri = uri
                .trim_start_matches("postgresql://")
                .trim_start_matches("postgres://");
            let parts: Vec<&str> = uri.split('@').collect();
            if parts.len() != 2 {
                return Err(IndustryDbError::config_error(
                    "Invalid PostgreSQL URI format",
                ));
            }

            let auth_parts: Vec<&str> = parts[0].split(':').collect();
            let server_parts: Vec<&str> = parts[1].split('/').collect();

            if auth_parts.len() != 2 || server_parts.len() != 2 {
                return Err(IndustryDbError::config_error(
                    "Invalid PostgreSQL URI format",
                ));
            }

            let host_port: Vec<&str> = server_parts[0].split(':').collect();
            let host = host_port[0].to_string();
            let port = host_port
                .get(1)
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432);

            Ok(Self::postgres(
                host,
                port,
                server_parts[1].to_string(),
                auth_parts[0].to_string(),
                auth_parts[1].to_string(),
            ))
        } else if uri.starts_with("sqlite://") {
            let path = uri.trim_start_matches("sqlite://");
            Ok(Self::sqlite(path))
        } else if uri.starts_with("mssql://") {
            // Simplified MSSQL URI parsing
            Err(IndustryDbError::config_error(
                "MSSQL URI parsing not fully implemented yet",
            ))
        } else {
            Err(IndustryDbError::config_error(format!(
                "Unsupported URI scheme: {}",
                uri
            )))
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        match self.db_type {
            DatabaseType::Postgres => {
                if self.host.is_none() {
                    return Err(IndustryDbError::config_error("Missing host for Postgres"));
                }
                if self.database.is_none() {
                    return Err(IndustryDbError::config_error(
                        "Missing database for Postgres",
                    ));
                }
                if self.username.is_none() {
                    return Err(IndustryDbError::config_error(
                        "Missing username for Postgres",
                    ));
                }
            }
            DatabaseType::Sqlite => {
                if self.path.is_none() {
                    return Err(IndustryDbError::config_error("Missing path for SQLite"));
                }
            }
            DatabaseType::Mssql => {
                if self.server.is_none() && self.host.is_none() {
                    return Err(IndustryDbError::config_error("Missing server for MSSQL"));
                }
                if self.database.is_none() {
                    return Err(IndustryDbError::config_error("Missing database for MSSQL"));
                }
            }
        }
        Ok(())
    }
}

/// Top-level database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Named connections
    pub connections: HashMap<String, ConnectionConfig>,
}

impl DatabaseConfig {
    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: DatabaseConfig = toml::from_str(&content)?;

        // Validate all connections
        for (name, conn) in &config.connections {
            conn.validate().map_err(|e| {
                IndustryDbError::config_error(format!("Invalid connection '{}': {}", name, e))
            })?;
        }

        Ok(config)
    }

    /// Get a connection by name
    pub fn get(&self, name: &str) -> Option<&ConnectionConfig> {
        self.connections.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_config() {
        let config = ConnectionConfig::postgres(
            "localhost".to_string(),
            5432,
            "testdb".to_string(),
            "user".to_string(),
            "pass".to_string(),
        );
        assert_eq!(config.db_type, DatabaseType::Postgres);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_sqlite_config() {
        let config = ConnectionConfig::sqlite("./test.db");
        assert_eq!(config.db_type, DatabaseType::Sqlite);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_uri_generation() {
        let config = ConnectionConfig::postgres(
            "localhost".to_string(),
            5432,
            "mydb".to_string(),
            "user".to_string(),
            "secret".to_string(),
        );
        let uri = config.to_uri().unwrap();
        assert!(uri.starts_with("postgresql://"));
        assert!(uri.contains("user:secret@localhost:5432/mydb"));
    }
}
