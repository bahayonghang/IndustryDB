"""Type stubs for industrydb Rust module."""

from typing import Any, Dict, List, Optional, Union
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
        host: Optional[str] = None,
        port: Optional[int] = None,
        database: Optional[str] = None,
        username: Optional[str] = None,
        password: Optional[str] = None,
        path: Optional[str] = None,
        **kwargs: Any
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
    def from_dict(config: Dict[str, Any]) -> PyDatabaseConfig:
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
    
    def to_dict(self) -> Dict[str, Any]:
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
    
    def execute(self, sql: str, params: Optional[List[Any]] = None) -> pl.DataFrame:
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
    
    def execute_many(self, sql: str, params_list: List[List[Any]]) -> int:
        """
        Execute SQL query with multiple parameter sets.
        
        Args:
            sql: SQL query string
            params_list: List of parameter sets
            
        Returns:
            Total number of affected rows
        """
        ...
    
    def insert(
        self,
        table: str,
        data: Union[pl.DataFrame, Dict[str, List[Any]]],
        **kwargs: Any
    ) -> int:
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
        columns: Optional[List[str]] = None,
        where: Optional[str] = None,
        params: Optional[List[Any]] = None,
        limit: Optional[int] = None,
        **kwargs: Any
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
        values: Dict[str, Any],
        where: Optional[str] = None,
        params: Optional[List[Any]] = None,
        **kwargs: Any
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
        where: Optional[str] = None,
        params: Optional[List[Any]] = None,
        **kwargs: Any
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
