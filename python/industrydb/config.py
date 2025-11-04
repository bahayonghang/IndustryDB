"""Configuration loading utilities for IndustryDB."""

from pathlib import Path
from typing import Dict, Any, Union

try:
    import rtoml as toml
except ImportError:
    import tomli as toml  # Fallback for Python < 3.11

from .industrydb import DatabaseConfig, ConfigurationError


def load_config(config_path: Union[str, Path]) -> Dict[str, DatabaseConfig]:
    """
    Load database configurations from a TOML file.

    Args:
        config_path: Path to the TOML configuration file

    Returns:
        Dictionary mapping connection names to DatabaseConfig objects

    Raises:
        ConfigurationError: If the configuration file is invalid or missing required fields
        FileNotFoundError: If the configuration file does not exist

    Example:
        >>> configs = load_config("database.toml")
        >>> conn = configs["my_postgres"].connect()
    """
    config_path = Path(config_path)
    
    if not config_path.exists():
        raise FileNotFoundError(f"Configuration file not found: {config_path}")
    
    try:
        with open(config_path, "rb") as f:
            data = toml.load(f)
    except Exception as e:
        raise ConfigurationError(f"Failed to parse TOML file: {e}") from e
    
    if "connections" not in data:
        raise ConfigurationError("Configuration file must contain 'connections' section")
    
    configs = {}
    for name, conn_config in data["connections"].items():
        try:
            # Validate required fields based on database type
            db_type = conn_config.get("type")
            if not db_type:
                raise ConfigurationError(f"Connection '{name}' missing 'type' field")
            
            configs[name] = DatabaseConfig.from_dict(conn_config)
        except Exception as e:
            raise ConfigurationError(
                f"Invalid configuration for connection '{name}': {e}"
            ) from e
    
    return configs


def validate_config(config: Dict[str, Any]) -> None:
    """
    Validate a configuration dictionary.

    Args:
        config: Configuration dictionary to validate

    Raises:
        ConfigurationError: If the configuration is invalid
    """
    required_fields = {"type"}
    
    if not isinstance(config, dict):
        raise ConfigurationError("Configuration must be a dictionary")
    
    db_type = config.get("type", "").lower()
    
    # Type-specific required fields
    type_requirements = {
        "postgres": {"host", "database", "username"},
        "postgresql": {"host", "database", "username"},
        "sqlite": {"path"},
        "mssql": {"server", "database"},
        "sqlserver": {"server", "database"},
    }
    
    if db_type not in type_requirements:
        raise ConfigurationError(
            f"Unsupported database type: {db_type}. "
            f"Supported types: {', '.join(type_requirements.keys())}"
        )
    
    required = required_fields | type_requirements[db_type]
    missing = required - set(config.keys())
    
    if missing:
        raise ConfigurationError(
            f"Missing required fields for {db_type}: {', '.join(missing)}"
        )
