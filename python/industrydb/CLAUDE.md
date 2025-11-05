# industrydb - Python Package

[Root](../../CLAUDE.md) > [python](../) > **industrydb**

## Change Log

### 2025-11-04 18:11:48 - Initial Documentation
- Documented Python package structure
- Added configuration loading utilities
- Documented public API and type stubs

---

## Module Responsibilities

The `python/industrydb` package provides pure Python utilities and the public API for IndustryDB. Key features:

- **Configuration loading**: Load TOML configs and validate
- **Type stubs**: Comprehensive type hints for IDE support
- **Re-exports**: Clean public API from Rust extension module
- **Utilities**: Helper functions for common tasks

This package wraps the Rust extension module (`industrydb.industrydb`) with a Pythonic interface.

## Entry and Startup

**Main entry**: `__init__.py`
```python
from .industrydb import (
    __version__,
    __author__,
    IndustryDbError,
    DatabaseConnectionError,
    QueryExecutionError,
    ConfigurationError,
)

from .config import load_config
from .industrydb import PyDatabaseConfig as DatabaseConfig
from .industrydb import PyConnection as Connection
```

**Package structure**:
```
python/industrydb/
├── __init__.py         # Public API exports
├── config.py           # Configuration loading utilities
├── industrydb.pyi      # Type stubs for Rust module
└── py.typed            # PEP 561 marker
```

## External Interfaces

### Public Functions

#### load_config
```python
def load_config(config_path: Union[str, Path]) -> Dict[str, DatabaseConfig]:
    """Load database configurations from TOML file."""
```

Loads and validates TOML configuration file, returns dict of named connections.

**Example**:
```python
import industrydb as idb

configs = idb.load_config("database.toml")
conn = idb.Connection(configs["my_postgres"])
```

#### validate_config
```python
def validate_config(config: Dict[str, Any]) -> None:
    """Validate configuration dictionary."""
```

Validates required fields for database type.

### Re-exported Classes

#### DatabaseConfig
Alias for `PyDatabaseConfig` from Rust module.

**Usage**:
```python
config = idb.DatabaseConfig(
    db_type="postgres",
    host="localhost",
    port=5432,
    database="mydb",
    username="user",
    password="pass"
)
```

#### Connection
Alias for `PyConnection` from Rust module.

**Usage**:
```python
with idb.Connection(config) as conn:
    df = conn.execute("SELECT * FROM users")
```

### Exception Classes

All exceptions imported from Rust module:
- `IndustryDbError` - Base exception
- `DatabaseConnectionError` - Connection failures
- `QueryExecutionError` - Query execution failures
- `ConfigurationError` - Configuration errors

## Key Dependencies and Configuration

### Dependencies

**pyproject.toml** (root):
```toml
[project]
dependencies = [
    "polars>=0.19.0",
]

[project.optional-dependencies]
config = [
    "rtoml>=0.9.0",
    "tomli>=2.0; python_version<'3.11'",
]
```

- **polars**: DataFrame library (required)
- **rtoml**: Fast TOML parser (optional, for config loading)
- **tomli**: Python 3.11 backport (optional fallback)

### Package Configuration

**Type checking** (PEP 561):
- `py.typed` marker file indicates typed package
- `.pyi` stub files provide type information

**Import paths**:
```python
import industrydb as idb           # Main package
from industrydb import Connection  # Public class
from industrydb.config import load_config  # Utility
```

## Data Models

### Configuration File Format

**TOML structure**:
```toml
[connections.postgres_main]
type = "postgres"
host = "localhost"
port = 5432
database = "production"
username = "app_user"
password = "secret"

[connections.sqlite_local]
type = "sqlite"
path = "./local.db"

[connections.mssql_analytics]
type = "mssql"
server = "mssql.example.com"
database = "analytics"
username = "reader"
password = "secret"
trusted_connection = false
```

### Type Requirements by Database

**PostgreSQL**:
```python
{
    "type": "postgres",
    "host": str,
    "database": str,
    "username": str,
    "password": str (optional if using cert auth),
    "port": int (optional, default 5432)
}
```

**SQLite**:
```python
{
    "type": "sqlite",
    "path": str  # File path or ":memory:"
}
```

**MSSQL**:
```python
{
    "type": "mssql",
    "server": str,
    "database": str,
    "username": str (if not using Windows auth),
    "password": str (if not using Windows auth),
    "trusted_connection": bool (optional)
}
```

## Testing and Quality

### Testing Strategy

**Location**: `tests/test_basic.py` (root level)

**Test categories**:
- Module import tests
- Configuration creation
- URI generation and parsing
- SQLite integration tests
- Context manager tests

### Running Tests

```bash
# All tests
pytest tests/ -v

# Specific test file
pytest tests/test_basic.py::test_sqlite_connection -v

# With coverage
pytest tests/ --cov=python/industrydb --cov-report=html
```

### Type Checking

```bash
# Check type hints
mypy python/industrydb

# Verify type stubs
stubtest industrydb
```

### Code Quality

```bash
# Lint
ruff check python/

# Format
ruff format python/

# Check imports
ruff check --select I python/
```

## Frequently Asked Questions

### Q: Why split Python code from Rust extension?
A: Separation of concerns - Rust provides performance, Python provides convenience and configuration.

### Q: Can I use this without the config module?
A: Yes, create `DatabaseConfig` directly without `load_config()`.

### Q: How do I handle missing TOML library?
A: Install with extras: `pip install industrydb[config]`

### Q: Are type hints checked at runtime?
A: No, type hints are for static analysis only. Use mypy or similar tools.

### Q: Can I extend the configuration loader?
A: Yes, `config.py` is pure Python and can be modified or extended.

## Related File List

**Python files**:
- `__init__.py` - Public API (36 lines)
- `config.py` - Configuration utilities (102 lines)
- `industrydb.pyi` - Type stubs (261 lines)
- `py.typed` - PEP 561 marker (empty)

**Test files**:
- `../../tests/test_basic.py` - Integration tests (142 lines)

**Example files**:
- `../../examples/quickstart.py` - Usage example (96 lines)

## Design Patterns

### Facade Pattern
Package provides simplified interface to complex Rust extension module.

### Configuration Loader Pattern
`load_config()` centralizes configuration loading and validation.

### Type Aliasing
```python
DatabaseConfig = PyDatabaseConfig  # Shorter, more Pythonic name
Connection = PyConnection
```

## Integration Points

**Imports from**:
- `industrydb.industrydb` - Rust extension module (built by maturin)

**Used by**:
- User applications
- Integration tests
- Example scripts

**Extends**:
- Adds pure Python utilities on top of Rust module

## API Design

### Pythonic Conventions

**Context managers**:
```python
with idb.Connection(config) as conn:
    df = conn.execute("SELECT * FROM users")
# Auto-closes connection
```

**Type hints**:
```python
def load_config(config_path: Union[str, Path]) -> Dict[str, DatabaseConfig]:
    ...
```

**Exceptions**:
```python
try:
    conn = idb.Connection(config)
except idb.DatabaseConnectionError as e:
    print(f"Failed to connect: {e}")
```

**Keyword arguments**:
```python
df = conn.select(
    table="users",
    columns=["id", "name"],
    where_clause="age > 18",
    limit=100
)
```

### Best Practices

1. **Use context managers** for automatic cleanup
2. **Load config from files** rather than hardcoding
3. **Handle exceptions** appropriately
4. **Use type hints** for better IDE support
5. **Close connections** explicitly if not using context manager

### Common Patterns

**Configuration from file**:
```python
configs = idb.load_config("database.toml")
conn = idb.Connection(configs["production"])
```

**Direct configuration**:
```python
config = idb.DatabaseConfig(db_type="sqlite", path=":memory:")
conn = idb.Connection(config)
```

**URI connection**:
```python
conn = idb.Connection.from_uri("postgresql://user:pass@localhost/db")
```

**Query and process**:
```python
import polars as pl

with idb.Connection(config) as conn:
    df = conn.execute("SELECT * FROM sales WHERE year = 2024")
    summary = df.group_by("region").agg(pl.sum("revenue"))
    print(summary)
```

## Configuration Examples

### Multi-environment Setup

**database.toml**:
```toml
[connections.dev]
type = "sqlite"
path = "./dev.db"

[connections.staging]
type = "postgres"
host = "staging-db.internal"
port = 5432
database = "staging"
username = "app"
password = "${STAGING_DB_PASSWORD}"  # Environment variable

[connections.production]
type = "postgres"
host = "prod-db.internal"
port = 5432
database = "production"
username = "app"
password = "${PROD_DB_PASSWORD}"
```

**Usage**:
```python
import os
import industrydb as idb

# Load configs
configs = idb.load_config("database.toml")

# Select based on environment
env = os.getenv("APP_ENV", "dev")
conn = idb.Connection(configs[env])
```

### Connection Pooling (Future)

Currently connection pooling is managed internally by Rust layer. Future versions may expose pool configuration:

```python
# Future API (not yet implemented)
config = idb.DatabaseConfig(
    db_type="postgres",
    host="localhost",
    database="mydb",
    username="user",
    password="pass",
    pool_size=10,  # Max connections
    pool_timeout=30  # Seconds
)
```

## Error Handling Guide

### Exception Hierarchy

```
IndustryDbError (base)
├── DatabaseConnectionError
│   └── Failed to connect to database
├── QueryExecutionError
│   └── SQL syntax error, permission denied, etc.
├── ConfigurationError
│   └── Invalid config, missing fields
└── ConnectionClosedError
    └── Operation on closed connection
```

### Handling Examples

```python
import industrydb as idb

# Connection errors
try:
    conn = idb.Connection(config)
except idb.DatabaseConnectionError as e:
    logger.error(f"Cannot connect: {e}")
    # Fallback to cache or retry

# Query errors
try:
    df = conn.execute("SELECT * FROM invalid_table")
except idb.QueryExecutionError as e:
    logger.error(f"Query failed: {e}")
    # Return empty DataFrame or default

# Configuration errors
try:
    configs = idb.load_config("missing.toml")
except FileNotFoundError:
    # Use default config
    config = idb.DatabaseConfig(db_type="sqlite", path=":memory:")
except idb.ConfigurationError as e:
    logger.error(f"Invalid config: {e}")
    sys.exit(1)
```
