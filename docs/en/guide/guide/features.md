# Features

IndustryDB provides a comprehensive set of features for high-performance database operations.

## Core Features

### 🚀 Blazing Fast Performance

Built with Rust for maximum speed:

- **Native compiled code**: No Python interpreter overhead
- **Zero-copy data transfer**: Direct memory mapping via Apache Arrow
- **Parallel execution**: Multi-threaded query processing with ConnectorX
- **SIMD optimizations**: Vectorized operations in Polars

**Performance Benchmarks**:
- 2-5x faster than SQLAlchemy for typical workloads
- Near-native query execution speed
- Minimal memory overhead

### 🔌 Multi-Database Support

Unified interface across databases:

```python
import industrydb as idb

# Same API for all databases
pg_conn = idb.Connection.from_uri("postgresql://...")
sqlite_conn = idb.Connection.from_uri("sqlite://...")
mssql_conn = idb.Connection.from_uri("mssql://...")

# Identical operations
for conn in [pg_conn, sqlite_conn, mssql_conn]:
    df = conn.select("users", where="age > ?", params=[18])
```

Supported databases:
- **PostgreSQL** 9.5+
- **SQLite** 3.x
- **Microsoft SQL Server** 2012+

### 📊 Native Polars Integration

First-class DataFrame support:

```python
import polars as pl
import industrydb as idb

# Create DataFrame
df = pl.DataFrame({
    "name": ["Alice", "Bob"],
    "age": [25, 30]
})

# Insert directly - no conversion needed
conn.insert("users", df)

# Query returns Polars DataFrame
result = conn.select("users")

# Chain Polars operations
result.filter(pl.col("age") > 25).sort("name")
```

Benefits:
- **Zero conversion overhead**: Direct Arrow → Polars
- **Type preservation**: Column types maintained
- **Lazy evaluation**: Optimize query chains
- **Rich operations**: Full Polars API available

### 🛡️ Type Safety

Comprehensive type hints for excellent IDE support:

```python
import industrydb as idb
import polars as pl

# Full type checking
conn: idb.Connection = idb.Connection.from_uri("...")
df: pl.DataFrame = conn.execute("SELECT * FROM users")

# Auto-completion in IDE
conn.select(  # IDE shows all parameters
    table="users",
    columns=["name", "age"],
    where="age > ?",
    params=[18]
)
```

Features:
- Complete `.pyi` stub files
- MyPy compatibility
- PyRight support
- Runtime type validation

### 🔧 Simple API

Pythonic and intuitive:

```python
import industrydb as idb

# Context manager support
with idb.Connection.from_uri("sqlite://./db.db") as conn:
    # CRUD operations
    conn.insert("users", {"name": ["Alice"], "age": [25]})
    df = conn.select("users")
    conn.update("users", {"age": 26}, where="name = ?", params=["Alice"])
    conn.delete("users", where="age < ?", params=[18])
    
# Auto cleanup - connection closed
```

### ⚙️ Flexible Configuration

Multiple configuration methods:

**1. URI Strings**:
```python
conn = idb.Connection.from_uri("postgresql://user:pass@host/db")
```

**2. TOML Files**:
```toml
[connections.prod]
type = "postgres"
host = "db.example.com"
database = "myapp"
username = "admin"
password = "secret"
```

```python
configs = idb.load_config("database.toml")
conn = configs["prod"].connect()
```

**3. Environment Variables**:
```python
import os
uri = os.environ["DATABASE_URL"]
conn = idb.Connection.from_uri(uri)
```

## Advanced Features

### Parameterized Queries

Safe SQL with parameter binding:

```python
# Positional parameters
df = conn.execute(
    "SELECT * FROM users WHERE age > ? AND city = ?",
    [18, "NYC"]
)

# Named parameters (if supported by database)
df = conn.execute(
    "SELECT * FROM users WHERE age > :age",
    {"age": 18}
)
```

### Batch Operations

Efficient bulk inserts:

```python
import polars as pl

# Large DataFrame
df = pl.DataFrame({
    "id": range(10000),
    "value": range(10000)
})

# Efficient batch insert
conn.insert("data", df)  # Fast bulk operation
```

### Error Handling

Typed exceptions for precise error handling:

```python
from industrydb import (
    ConnectionError,
    QueryExecutionError,
    ConstraintViolationError,
    ConnectionClosedError
)

try:
    conn = idb.Connection.from_uri("postgresql://...")
except ConnectionError:
    # Handle connection failure
    pass

try:
    conn.execute("SELECT * FROM nonexistent")
except QueryExecutionError as e:
    # Handle query error
    print(f"Query failed: {e}")

try:
    conn.insert("users", {"id": [1]})  # Duplicate
except ConstraintViolationError:
    # Handle constraint violation
    pass
```

### Context Manager Support

Automatic resource cleanup:

```python
# Connection auto-closes
with idb.Connection.from_uri("...") as conn:
    df = conn.select("users")
    # Work with data

# Connection guaranteed closed, even if exception occurs
```

## Database-Specific Features

### PostgreSQL

- Full JSONB support
- Array types
- Custom types
- Window functions
- CTEs (Common Table Expressions)

```python
df = conn.execute("""
    WITH recent_users AS (
        SELECT * FROM users 
        WHERE created_at > NOW() - INTERVAL '7 days'
    )
    SELECT * FROM recent_users
    WHERE jsonb_column @> '{"key": "value"}'
""")
```

### SQLite

- In-memory databases
- File-based databases
- No server setup required
- ACID transactions

```python
# In-memory for testing
conn = idb.Connection.from_uri("sqlite://:memory:")

# File-based for persistence
conn = idb.Connection.from_uri("sqlite://./app.db")
```

### MSSQL

- Windows Authentication
- SQL Server specific types
- TOP clause support
- Stored procedures

```python
# Use TOP instead of LIMIT
df = conn.execute("SELECT TOP 10 * FROM users")
```

## Coming Soon

Features planned for future releases:

### Phase 2
- ⏳ Connection pooling
- ⏳ Transaction support
- ⏳ Async API
- ⏳ Prepared statements

### Phase 3
- ⏳ MySQL support
- ⏳ Oracle support
- ⏳ Query builder
- ⏳ Schema migration tools
- ⏳ ORM layer (optional)

## Comparison Matrix

| Feature | IndustryDB | SQLAlchemy | pandas |
|---------|-----------|------------|--------|
| Speed | ⚡⚡⚡⚡⚡ | ⚡⚡⚡ | ⚡⚡ |
| Type Hints | ✅ Full | ⚠️ Partial | ❌ Limited |
| DataFrame | ✅ Polars | ⚠️ Pandas | ✅ Pandas |
| Multi-DB | ✅ 3 DBs | ✅ 10+ DBs | ⚠️ Via SQL |
| ORM | ❌ No | ✅ Yes | ❌ No |
| Learning Curve | 🟢 Easy | 🟡 Medium | 🟢 Easy |
| Memory | ✅ Low | ⚠️ Medium | ❌ High |

## Learn More

- [Getting Started](/guide/getting-started) - Installation and first query
- [Configuration](/guide/configuration) - Config file setup
- [CRUD Operations](/guide/crud-operations) - Data manipulation
- [Examples](/examples/quick-start) - Practical examples
