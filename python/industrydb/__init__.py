"""
IndustryDB - High-performance database middleware

A Rust-powered database connector supporting MSSQL, PostgreSQL, and SQLite
with Polars DataFrame integration.
"""

# Re-export main classes for convenience
from .config import load_config
from .industrydb import (
    ConfigurationError,
    DatabaseConnectionError,
    IndustryDbError,
    QueryExecutionError,
    __author__,
    __version__,
)
from .industrydb import PyConnection as Connection
from .industrydb import PyDatabaseConfig as DatabaseConfig

__all__ = [
    "__version__",
    "__author__",
    # Config
    "DatabaseConfig",
    "load_config",
    # Connection
    "Connection",
    # Exceptions
    "IndustryDbError",
    "DatabaseConnectionError",
    "QueryExecutionError",
    "ConfigurationError",
]
