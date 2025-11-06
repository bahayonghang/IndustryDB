# Getting Started

This guide will help you install IndustryDB and write your first database queries.

## Prerequisites

Before installing IndustryDB, ensure you have:

- **Python 3.8 or higher**
- **pip** or **uv** (recommended)

For building from source, you'll also need:
- **Rust 1.70+** (install from [rustup.rs](https://rustup.rs/))
- **System dependencies** (see [Build Requirements](#build-requirements))

## Installation

### From PyPI (Recommended)

Once IndustryDB is published, install via pip:

```bash
pip install industrydb
```

Or using [uv](https://github.com/astral-sh/uv) (faster):

```bash
uv pip install industrydb
```

### From Source

For the latest development version:

```bash
# Clone the repository
git clone https://github.com/yourusername/industrydb.git
cd industrydb

# Create virtual environment
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install build dependencies
uv pip install maturin

# Build and install
uv run maturin develop
```

### Build Requirements

::: details Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    libkrb5-dev \
    libssl-dev \
    pkg-config
```
:::

::: details macOS
```bash
brew install krb5 openssl
```
:::

::: details Windows
Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ development tools.
:::

## Quick Start

### Your First Query

Create a file `quickstart.py`:

```python
import industrydb as idb

# Connect to SQLite (easiest to start with)
conn = idb.Connection.from_uri("sqlite://./test.db")

# Execute a query
df = conn.execute("""
    SELECT 'Hello' as message, 
           'IndustryDB' as name,
           42 as answer
""")

print(df)
conn.close()
```

Run it:

```bash
python quickstart.py
```

Expected output:
```
shape: (1, 3)
┌─────────┬────────────┬────────┐
│ message ┆ name       ┆ answer │
│ ---     ┆ ---        ┆ ---    │
│ str     ┆ str        ┆ i64    │
╞═════════╪════════════╪════════╡
│ Hello   ┆ IndustryDB ┆ 42     │
└─────────┴────────────┴────────┘
```

### Using Context Manager

Better practice with automatic cleanup:

```python
import industrydb as idb

# Context manager automatically closes connection
with idb.Connection.from_uri("sqlite://./test.db") as conn:
    df = conn.execute("SELECT * FROM users")
    print(df)
    
# Connection is automatically closed here
```

## Configuration File

For production use, store connection details in a TOML file.

### Create Configuration

Create `database.toml`:

```toml
[connections.dev_db]
type = "sqlite"
path = "./dev.db"

[connections.prod_db]
type = "postgres"
host = "localhost"
port = 5432
database = "myapp"
username = "dbuser"
password = "secret"

[connections.analytics]
type = "mssql"
host = "analytics.example.com"
port = 1433
database = "warehouse"
username = "analyst"
password = "secret"
```

### Load Configuration

```python
import industrydb as idb

# Load all configurations
configs = idb.load_config("database.toml")

# Connect to specific database
with configs["dev_db"].connect() as conn:
    df = conn.execute("SELECT * FROM users")
    print(df)

# Switch to another database easily
with configs["prod_db"].connect() as conn:
    df = conn.execute("SELECT * FROM orders")
    print(df)
```

## Basic CRUD Operations

### Insert Data

```python
import industrydb as idb

conn = idb.Connection.from_uri("sqlite://./test.db")

# Create table first
conn.execute("""
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        age INTEGER
    )
""")

# Insert single record
conn.insert("users", {
    "name": ["Alice"],
    "age": [25]
})

# Insert multiple records
conn.insert("users", {
    "name": ["Bob", "Charlie"],
    "age": [30, 35]
})

conn.close()
```

### Select Data

```python
# Select all
df = conn.select("users")

# Select with condition
df = conn.select("users", where="age > ?", params=[25])

# Select specific columns
df = conn.select("users", columns=["name", "age"])

# Combine options
df = conn.select(
    "users",
    columns=["name"],
    where="age >= ? AND age <= ?",
    params=[25, 35]
)
```

### Update Data

```python
# Update records
rows_affected = conn.update(
    "users",
    {"age": 26},
    where="name = ?",
    params=["Alice"]
)
print(f"Updated {rows_affected} rows")
```

### Delete Data

```python
# Delete records
rows_affected = conn.delete(
    "users",
    where="age < ?",
    params=[18]
)
print(f"Deleted {rows_affected} rows")
```

## Working with DataFrames

IndustryDB seamlessly integrates with Polars:

```python
import polars as pl
import industrydb as idb

# Create a Polars DataFrame
df = pl.DataFrame({
    "id": [1, 2, 3],
    "name": ["Alice", "Bob", "Charlie"],
    "age": [25, 30, 35],
    "city": ["NYC", "LA", "SF"]
})

# Insert DataFrame directly
conn = idb.Connection.from_uri("sqlite://./test.db")
conn.insert("users", df)

# Query returns Polars DataFrame
result = conn.select("users")

# Use Polars operations
filtered = result.filter(pl.col("age") > 25)
sorted_df = filtered.sort("age", descending=True)
print(sorted_df)

conn.close()
```

## Error Handling

IndustryDB provides typed exceptions:

```python
import industrydb as idb
from industrydb import (
    ConnectionError,
    QueryExecutionError,
    ConstraintViolationError
)

try:
    conn = idb.Connection.from_uri("postgresql://invalid")
except ConnectionError as e:
    print(f"Failed to connect: {e}")

try:
    conn.execute("INVALID SQL")
except QueryExecutionError as e:
    print(f"Query failed: {e}")

try:
    # Duplicate key insert
    conn.insert("users", {"id": [1], "name": ["Alice"]})
except ConstraintViolationError as e:
    print(f"Constraint violated: {e}")
```

## Database-Specific Examples

### PostgreSQL

```python
import industrydb as idb

conn = idb.Connection.from_uri(
    "postgresql://user:password@localhost:5432/mydb"
)

# PostgreSQL-specific features
df = conn.execute("""
    SELECT * FROM users
    WHERE created_at > NOW() - INTERVAL '7 days'
    ORDER BY created_at DESC
    LIMIT 100
""")

conn.close()
```

### SQLite

```python
import industrydb as idb

# In-memory database
conn = idb.Connection.from_uri("sqlite://:memory:")

# File-based database
conn = idb.Connection.from_uri("sqlite://./myapp.db")

# Relative path
conn = idb.Connection.from_uri("sqlite://./data/app.db")

conn.close()
```

### MSSQL

```python
import industrydb as idb

conn = idb.Connection.from_uri(
    "mssql://user:password@server:1433/database"
)

# MSSQL uses TOP instead of LIMIT
df = conn.execute("SELECT TOP 10 * FROM orders")

conn.close()
```

## Next Steps

Now that you've completed the basics:

- [Configuration Guide](/guide/configuration) - Advanced configuration options
- [CRUD Operations](/guide/crud-operations) - Detailed CRUD documentation
- [DataFrame Integration](/guide/dataframe) - Working with Polars
- [Error Handling](/guide/error-handling) - Comprehensive error handling
- [Examples](/examples/quick-start) - More practical examples

## Troubleshooting

### Import Error

If you see `ModuleNotFoundError: No module named 'industrydb'`:

```bash
# Verify installation
pip list | grep industrydb

# Reinstall if necessary
pip install --force-reinstall industrydb
```

### Connection Error

If connection fails:

1. **Check database is running**: Verify your database server is accessible
2. **Verify credentials**: Ensure username/password are correct
3. **Check network**: Ensure firewall allows connection
4. **Test with native client**: Try connecting with psql, sqlite3, or sqlcmd

### Build Errors

If building from source fails:

1. **Update Rust**: `rustup update`
2. **Install system dependencies**: See [Build Requirements](#build-requirements)
3. **Check issue tracker**: [GitHub Issues](https://github.com/yourusername/industrydb/issues)
