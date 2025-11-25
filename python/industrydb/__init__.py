"""
IndustryDB - High-performance database middleware

A Rust-powered database connector supporting MSSQL, PostgreSQL, and SQLite
with Polars DataFrame integration.

# Factory helpers: create_*_client for MSSQL domain clients (web/timeseries/quality/etc.)
"""

# Re-export main classes for convenience
from .config import load_config
from .industrydb import (
    ConfigurationError,
    DatabaseConnectionError,
    IndustryDbError,
    PyDecisionHistoryClient,
    PyDecisionMakingClient,
    PyModelStatusClient,
    PyOperationClient,
    PyQualityClient,
    PyRealtimePredictAlterClient,
    PyRealtimePredictClient,
    PyRealtimePredictSequenceClient,
    PyTimeSeriesClient,
    PyWebClient,
    QueryExecutionError,
    __author__,
    __version__,
    create_decision_history_client,
    create_decision_making_client,
    create_model_status_client,
    create_operation_client,
    create_quality_client,
    create_realtime_predict_alter_client,
    create_realtime_predict_client,
    create_realtime_predict_sequence_client,
    create_timeseries_client,
    create_web_client,
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
    # Domain clients
    "PyWebClient",
    "PyQualityClient",
    "PyTimeSeriesClient",
    "PyRealtimePredictClient",
    "PyRealtimePredictAlterClient",
    "PyRealtimePredictSequenceClient",
    "PyDecisionHistoryClient",
    "PyDecisionMakingClient",
    "PyOperationClient",
    "PyModelStatusClient",
    # Factory helpers
    "create_web_client",
    "create_quality_client",
    "create_timeseries_client",
    "create_realtime_predict_client",
    "create_realtime_predict_alter_client",
    "create_realtime_predict_sequence_client",
    "create_decision_history_client",
    "create_decision_making_client",
    "create_operation_client",
    "create_model_status_client",
]
