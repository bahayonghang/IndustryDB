"""
IndustryDB - High-performance database middleware

A Rust-powered database connector supporting MSSQL, PostgreSQL, and SQLite
with Polars DataFrame integration.
"""

from .industrydb import (
    __version__,
    __author__,
    IndustryDbError,
    DatabaseConnectionError,
    QueryExecutionError,
    ConfigurationError,
)

# Re-export main classes for convenience
from .config import load_config
from .industrydb import PyDatabaseConfig as DatabaseConfig
from .industrydb import PyConnection as Connection

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
