"""Type stubs for industrydb Rust module."""

from typing import Any

import polars as pl

__version__: str
__author__: str

class IndustryDbError(Exception):
    """Base exception for IndustryDB errors."""

    ...

class DatabaseConnectionError(IndustryDbError):
    """Raised when database connection fails."""

    ...

class QueryExecutionError(IndustryDbError):
    """Raised when query execution fails."""

    ...

class ConfigurationError(IndustryDbError):
    """Raised when configuration is invalid."""

    ...

class PyDatabaseConfig:
    """Database configuration."""

    def __init__(
        self,
        db_type: str,
        host: str | None = None,
        port: int | None = None,
        database: str | None = None,
        username: str | None = None,
        password: str | None = None,
        path: str | None = None,
        **kwargs: Any,
    ) -> None:
        """
        Create a new database configuration.

        Args:
            db_type: Database type ('postgres', 'sqlite', 'mssql')
            host: Database host (for postgres/mssql)
            port: Database port (for postgres/mssql)
            database: Database name (for postgres/mssql)
            username: Username (for postgres/mssql)
            password: Password (for postgres/mssql)
            path: Database file path (for sqlite)
            **kwargs: Additional database-specific options
        """
        ...

    @staticmethod
    def from_dict(config: dict[str, Any]) -> PyDatabaseConfig:
        """
        Create configuration from dictionary.

        Args:
            config: Configuration dictionary

        Returns:
            Database configuration object
        """
        ...

    @staticmethod
    def from_uri(uri: str) -> PyDatabaseConfig:
        """
        Create configuration from URI connection string.

        Args:
            uri: Database URI (e.g., 'postgresql://user:pass@host:port/db')

        Returns:
            Database configuration object
        """
        ...

    def to_dict(self) -> dict[str, Any]:
        """Convert configuration to dictionary."""
        ...

    def to_uri(self) -> str:
        """Convert configuration to URI connection string."""
        ...

class PyConnection:
    """Database connection."""

    def __init__(self, config: PyDatabaseConfig) -> None:
        """
        Create a new database connection.

        Args:
            config: Database configuration
        """
        ...

    @staticmethod
    def connect(config: PyDatabaseConfig) -> PyConnection:
        """
        Establish database connection.

        Args:
            config: Database configuration

        Returns:
            Active database connection

        Raises:
            DatabaseConnectionError: If connection fails
        """
        ...

    @staticmethod
    def from_uri(uri: str) -> PyConnection:
        """
        Connect using URI connection string.

        Args:
            uri: Database URI

        Returns:
            Active database connection
        """
        ...

    def close(self) -> None:
        """Close the database connection."""
        ...

    def is_closed(self) -> bool:
        """Check if connection is closed."""
        ...

    def execute(self, sql: str, params: list[Any] | None = None) -> pl.DataFrame:
        """
        Execute SQL query and return results as DataFrame.

        Args:
            sql: SQL query string
            params: Optional query parameters

        Returns:
            Query results as Polars DataFrame

        Raises:
            QueryExecutionError: If query execution fails
        """
        ...

    def execute_many(self, sql: str, params_list: list[list[Any]]) -> int:
        """
        Execute SQL query with multiple parameter sets.

        Args:
            sql: SQL query string
            params_list: List of parameter sets

        Returns:
            Total number of affected rows
        """
        ...

    def insert(self, table: str, data: pl.DataFrame | dict[str, list[Any]], **kwargs: Any) -> int:
        """
        Insert data into table.

        Args:
            table: Table name
            data: Data to insert (DataFrame or dict)
            **kwargs: Additional options

        Returns:
            Number of rows inserted
        """
        ...

    def select(
        self,
        table: str,
        columns: list[str] | None = None,
        where: str | None = None,
        params: list[Any] | None = None,
        limit: int | None = None,
        **kwargs: Any,
    ) -> pl.DataFrame:
        """
        Select data from table.

        Args:
            table: Table name
            columns: Columns to select (None for all)
            where: WHERE clause
            params: Query parameters
            limit: Maximum rows to return
            **kwargs: Additional options

        Returns:
            Query results as DataFrame
        """
        ...

    def update(
        self,
        table: str,
        values: dict[str, Any],
        where: str | None = None,
        params: list[Any] | None = None,
        **kwargs: Any,
    ) -> int:
        """
        Update rows in table.

        Args:
            table: Table name
            values: Column values to update
            where: WHERE clause
            params: Query parameters
            **kwargs: Additional options

        Returns:
            Number of rows updated
        """
        ...

    def delete(
        self,
        table: str,
        where: str | None = None,
        params: list[Any] | None = None,
        **kwargs: Any,
    ) -> int:
        """
        Delete rows from table.

        Args:
            table: Table name
            where: WHERE clause
            params: Query parameters
            **kwargs: Additional options

        Returns:
            Number of rows deleted
        """
        ...

    def __enter__(self) -> PyConnection:
        """Context manager entry."""
        ...

    def __exit__(self, exc_type: Any, exc_val: Any, exc_tb: Any) -> None:
        """Context manager exit."""
        ...

# MSSQL domain clients (Python-facing)

class PyWebClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def get_prediction_info(self, project_name: str) -> tuple[str, str]: ...
    def get_rt_info(self, project_name: str) -> dict[str, list[Any]]: ...
    def get_model_parameter(self, project_name: str) -> tuple[str, str]: ...
    def get_sample_table_information(self, sample_name: str) -> dict[str, list[Any]]: ...
    def get_optimization_info(
        self, project_name: str
    ) -> tuple[
        dict[str, list[Any]], dict[str, list[Any]], list[str], list[str], list[list[str]]
    ]: ...
    def get_constraint_table(self, project_name: str) -> dict[str, list[Any]]: ...
    def get_constraint_table_full(self, project_name: str) -> dict[str, list[Any]]: ...
    def check_train_info_table(self) -> None: ...
    def insert_train_info(
        self,
        project_name: str,
        epoch: int,
        train_loss: float,
        vali_loss: float,
        model_type: str,
        uuid: str,
    ) -> None: ...
    def init_thread_status_table(self) -> None: ...
    def fresh_thread_status_table(self) -> None: ...
    def update_thread_status(
        self,
        project_name: str,
        project_type: str,
        train_status: int,
        predict_status: int,
    ) -> None: ...
    def check_online_trainer_table(self) -> None: ...
    def insert_online_trainer_info(
        self,
        project_name: str,
        epoch: int,
        train_loss: float,
        test_loss: float,
    ) -> None: ...
    def get_online_trainer_info(self, project_name: str) -> dict[str, list[Any]]: ...
    def ensure_table_exist_operation_status(self) -> None: ...
    def init_operation_status(self) -> None: ...
    def update_operation_status(
        self, project_name: str, project_type: str, operation_status: int
    ) -> None: ...
    def write_device_status(self, project_name: str, device_status: int) -> None: ...
    def insert_device_status(self, project_name: str, device_status: int) -> None: ...

class PyQualityClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def get_output_data_train(self, table: str, begin: str, end: str) -> dict[str, list[Any]]: ...
    def get_output_data_history(self, table: str, begin: str, end: str) -> dict[str, list[Any]]: ...
    def get_latest_true_value(self, table: str) -> dict[str, list[Any]]: ...

class PyTimeSeriesClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def get_latest_input_data(self, table: str) -> dict[str, list[Any]]: ...
    def get_latest_input_data_view(
        self, table: str, time_length: int, unit: str
    ) -> dict[str, list[Any]]: ...
    def get_latest_true_value_by_column(self, column_name: str) -> dict[str, list[Any]]: ...
    def get_latest_true_value(self, table: str) -> dict[str, list[Any]]: ...
    def get_input_data_online(self, name_list: list[str]) -> dict[str, list[Any]]: ...
    def get_output_data_online(self, name_list: list[str]) -> dict[str, list[Any]]: ...
    def get_input_data_train(self, table: str, begin: str, end: str) -> dict[str, list[Any]]: ...
    def get_input_data_history(
        self,
        name_list: list[str],
        begin: str,
        end: str,
        chunk_size: str | None = ...,
        use_temp_file: bool = ...,
    ) -> dict[str, list[Any]]: ...
    def get_output_data_history(
        self, name_list: list[str], begin: str, end: str, chunk_size: str | None = ...
    ) -> dict[str, list[Any]]: ...
    def get_latest_input_data_by_column(self, name_list: list[str]) -> dict[str, list[Any]]: ...
    def get_latest_input_data_by_column_time(
        self, name_list: list[str], hours: int
    ) -> dict[str, list[Any]]: ...
    def get_latest_input_data_by_column_time_hour(
        self, name_list: list[str], hours: int
    ) -> dict[str, list[Any]]: ...
    def get_latest_input_data_by_column_time_min(
        self, name_list: list[str], minutes: int
    ) -> dict[str, list[Any]]: ...
    def get_history_data_by_name_list(
        self, name_list: list[str], begin: str, end: str
    ) -> dict[str, list[Any]]: ...
    def check_tagname_exist(self, tag_name: str) -> bool: ...
    def update_tagval(self, tag_name: str, tag_val: float) -> None: ...
    def get_feed_amount_column_name(self, table: str) -> list[str]: ...
    def check_device_running_status(self, var_name: str) -> bool: ...
    def get_latest_input_data_model_parameter(
        self, name_list: list[str], filter_length: str, time_length: int
    ) -> dict[str, list[Any]]: ...
    def get_optimization_mode(self, optimization_procedure: str) -> str: ...
    def get_cement_mill_target(self, optimization_procedure: str) -> tuple[int, float]: ...
    def get_cement_mill_a_target(self) -> tuple[int, float]: ...
    def get_cement_mill_b_target(self) -> tuple[int, float]: ...
    def query_decision_history_data(self, table: str, minutes: int) -> dict[str, list[Any]]: ...
    def query_decision_history_data_by_name(
        self, name_list: list[str], time_query: int
    ) -> dict[str, list[Any]]: ...
    def write_decision_result(
        self,
        optimization_project_name: str,
        optimization_name_list: list[str],
        optimization_solution_results: list[float],
        optimization_type: str,
    ) -> None: ...
    def query_history_data_latest(self) -> dict[str, list[Any]]: ...
    def query_decision_history_data_latest_by_name_list(
        self, name_list: list[str]
    ) -> dict[str, list[Any]]: ...
    def query_tagdatabase_latest(self) -> dict[str, list[Any]]: ...
    def query_tagdatabase_latest_by_name_list(
        self, name_list: list[str]
    ) -> dict[str, list[Any]]: ...
    def check_decision_table(self) -> None: ...
    def query_decision_solution_rt_by_column(
        self, column_name_list: list[str], optimization_type: str
    ) -> dict[str, list[Any]]: ...

class PyRealtimePredictClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def check_table(self, table: str) -> None: ...
    def update_realvalue(self, table: str, realvalue: float, time_real: str) -> None: ...
    def update_predictvalue(self, table: str, predictvalue: float, time_predict: str) -> None: ...
    def query_latest_predict_value(self, table: str) -> dict[str, list[Any]]: ...
    def add_ai_predict_column(self, table: str) -> None: ...
    def update_ai_predict_value(self, table: str, data: dict[str, list[Any]]) -> None: ...
    def update_ai_predict_index(self, table: str, ai_index: float, time_predict: str) -> None: ...

class PyRealtimePredictAlterClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def check_table(self, table: str) -> None: ...
    def update_predictvalue(self, table: str, predictvalue: float, time_predict: str) -> None: ...
    def get_predict_data_by_table_name(self, table: str) -> dict[str, list[Any]]: ...

class PyRealtimePredictSequenceClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def create_sequence_table(self, table: str, name_list: list[str]) -> None: ...
    def create_sequence_predict_table(self, table: str, name_list: list[str]) -> None: ...
    def update_realvalue(self, table: str, data: dict[str, list[Any]]) -> None: ...
    def get_predict_data_by_table_name(self, table: str) -> dict[str, list[Any]]: ...

class PyDecisionHistoryClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def check_table_decision_history(self) -> None: ...
    def insert_decision_history(
        self,
        datetime: str,
        variable_name: str,
        project_name: str,
        opt_type: str,
        decision_value: float,
    ) -> None: ...
    def query_decision_history_recent_1h(self) -> dict[str, list[Any]]: ...

class PyDecisionMakingClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def check_decision_table(self) -> None: ...

class PyOperationClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def check_operation_table(self, project_name: str) -> None: ...
    def insert_operation_data(
        self,
        project_name: str,
        epoch: int,
        train_loss: float,
        vali_loss: float,
        sample_size: int,
        start_time: str,
    ) -> None: ...
    def get_operation_data(self, project_name: str) -> dict[str, list[Any]]: ...

class PyModelStatusClient:
    def __init__(self, config: PyDatabaseConfig) -> None: ...
    def init_model_status_table(self, project_name: str) -> None: ...
    def insert_model_status(
        self,
        project_name: str,
        algorithm_name: str,
        model_type: str,
        model_parameter_json: str,
        start_time: str,
        end_time: str,
        sample_size: int,
        mae: float,
        rmse: float,
        mape: float,
    ) -> None: ...
    def get_model_status(self, project_name: str) -> dict[str, list[Any]]: ...

def create_web_client(config: PyDatabaseConfig) -> PyWebClient: ...
def create_quality_client(config: PyDatabaseConfig) -> PyQualityClient: ...
def create_timeseries_client(config: PyDatabaseConfig) -> PyTimeSeriesClient: ...
def create_realtime_predict_client(config: PyDatabaseConfig) -> PyRealtimePredictClient: ...
def create_realtime_predict_alter_client(
    config: PyDatabaseConfig,
) -> PyRealtimePredictAlterClient: ...
def create_realtime_predict_sequence_client(
    config: PyDatabaseConfig,
) -> PyRealtimePredictSequenceClient: ...
def create_decision_history_client(config: PyDatabaseConfig) -> PyDecisionHistoryClient: ...
def create_decision_making_client(config: PyDatabaseConfig) -> PyDecisionMakingClient: ...
def create_operation_client(config: PyDatabaseConfig) -> PyOperationClient: ...
def create_model_status_client(config: PyDatabaseConfig) -> PyModelStatusClient: ...
