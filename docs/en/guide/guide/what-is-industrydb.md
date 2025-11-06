# What is IndustryDB?

IndustryDB is a high-performance database middleware that combines the speed of Rust with the simplicity of Python. It provides a unified interface for multiple databases while leveraging Polars for efficient data operations.

## Core Philosophy

IndustryDB is built on three key principles:

### 🚀 Performance First

Written in Rust and compiled to native code, IndustryDB delivers exceptional performance:

- **Zero-copy data transfer** via Apache Arrow format
- **Parallel query execution** through ConnectorX
- **Lazy evaluation** for query optimization
- **Minimal Python overhead** with PyO3 bindings

### 🎯 Developer Experience

A clean, Pythonic API that feels natural:

```python
import industrydb as idb

# Simple and intuitive
conn = idb.Connection.from_uri("postgresql://localhost/mydb")
df = conn.execute("SELECT * FROM users")
conn.close()
```

### 🔌 Unified Interface

One API for multiple databases:

- PostgreSQL
- SQLite  
- Microsoft SQL Server

Switch databases without changing your code structure.

## Key Features

### Multi-Database Support

IndustryDB provides native support for three major database systems:

| Database | Status | Driver |
|----------|--------|--------|
| PostgreSQL | ✅ Full Support | ConnectorX |
| SQLite | ✅ Full Support | ConnectorX |
| MSSQL | ✅ Full Support | ConnectorX |

### Polars Integration

First-class support for Polars DataFrames:

```python
import polars as pl
import industrydb as idb

# Create DataFrame
df = pl.DataFrame({
    "id": [1, 2, 3],
    "name": ["Alice", "Bob", "Charlie"]
})

# Direct DataFrame operations
conn = idb.Connection.from_uri("sqlite://./data.db")
conn.insert("users", df)
result = conn.select("users")
```

### Type Safety

Comprehensive type hints for excellent IDE support:

- Full `.pyi` stub files
- Runtime type checking
- MyPy compatibility
- Autocomplete in modern IDEs

### CRUD Operations

Simple CRUD operations with optional parameters:

```python
# Insert
conn.insert("users", {"name": ["Alice"], "age": [25]})

# Select with conditions
df = conn.select("users", where="age > ?", params=[18])

# Update
conn.update("users", {"age": 26}, where="name = ?", params=["Alice"])

# Delete
conn.delete("users", where="age < ?", params=[18])
```

## Architecture Overview

IndustryDB uses a modular multi-crate architecture:

```
┌─────────────────────────────────┐
│     Python Layer (User API)     │
│  - Simple Pythonic interface    │
└────────────┬────────────────────┘
             │ PyO3 Bindings
┌────────────▼────────────────────┐
│   industrydb-py (Rust→Python)   │
│  - Factory pattern              │
│  - Exception mapping            │
└─┬──────────┬──────────┬─────────┘
  │          │          │
  ▼          ▼          ▼
┌────────┬────────┬────────┐
│Postgres│ SQLite │ MSSQL  │ Connectors
└───┬────┴───┬────┴───┬────┘
    └────────┼────────┘
             │
    ┌────────▼────────┐
    │ industrydb-core │ Core Traits
    │ - Traits        │
    │ - Config        │
    │ - Errors        │
    └─────────────────┘
```

### Design Principles

- **Trait-based abstractions**: All connectors implement common traits
- **Factory pattern**: Runtime selection of database connector
- **Zero-copy transfers**: Arrow format for efficient data exchange
- **Separation of concerns**: Clear boundaries between layers

## Performance

IndustryDB is designed for performance:

- **Native Rust code**: Compiled to machine code, no interpreter overhead
- **ConnectorX**: Battle-tested driver with parallel query support
- **Apache Arrow**: Column-oriented format enabling zero-copy operations
- **Polars**: Fast DataFrame operations with SIMD optimizations

Benchmark results show IndustryDB is typically 2-5x faster than traditional Python database libraries for data-intensive workloads.

## Use Cases

IndustryDB is ideal for:

### Data Engineering

- **ETL pipelines**: Fast data extraction and transformation
- **Data warehousing**: Bulk loading and querying
- **Data migration**: Cross-database data transfer

### Analytics

- **Exploratory analysis**: Quick queries with DataFrame output
- **Reporting**: Generate reports from multiple data sources
- **Data science**: Integration with Polars ecosystem

### Application Development

- **Backend services**: Fast database access for APIs
- **Batch processing**: Efficient bulk operations
- **Real-time processing**: Low-latency query execution

## Comparison

### vs. SQLAlchemy

| Feature | IndustryDB | SQLAlchemy |
|---------|-----------|------------|
| Language | Rust + Python | Pure Python |
| Performance | ~3-5x faster | Baseline |
| DataFrame | Native Polars | Pandas adapter |
| API Style | Functional | ORM + Core |
| Type Hints | Comprehensive | Partial |

### vs. pandas + SQLAlchemy

| Feature | IndustryDB | pandas + SQLAlchemy |
|---------|-----------|---------------------|
| Speed | 🚀 Very Fast | 🐌 Slower |
| Memory | ✅ Efficient | ❌ Higher overhead |
| Type Safety | ✅ Full | ⚠️ Limited |
| Setup | Simple | Complex |

## What's Next?

- [Getting Started](/guide/getting-started) - Install and run your first query
- [Configuration](/guide/configuration) - Learn about config options
- [CRUD Operations](/guide/crud-operations) - Master data manipulation
- [API Reference](/api/connection) - Detailed API documentation
