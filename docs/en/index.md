---
layout: home

hero:
  name: "IndustryDB"
  text: "High-Performance Database Middleware"
  tagline: Powered by Rust and Polars for blazing-fast data operations
  image:
    src: /logo.svg
    alt: IndustryDB
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/yourusername/industrydb

features:
  - icon: ⚡
    title: Blazing Fast
    details: Built with Rust for maximum performance. Zero-copy data transfer via Apache Arrow format ensures minimal overhead.
    
  - icon: 🔌
    title: Multi-Database Support
    details: Seamless support for PostgreSQL, SQLite, and MSSQL with a unified interface.
    
  - icon: 📊
    title: Polars Integration
    details: Native DataFrame support with first-class Polars integration for efficient data processing.
    
  - icon: 🛡️
    title: Type Safe
    details: Comprehensive type hints and stubs for excellent IDE support and type checking.
    
  - icon: 🔧
    title: Easy to Use
    details: Simple, Pythonic API that feels natural. Context manager support for resource management.
    
  - icon: 🚀
    title: Production Ready
    details: Modular architecture with comprehensive error handling and extensive test coverage.
---

## Quick Example

::: code-group
```python [Quick Start]
import industrydb as idb

# Connect using URI
conn = idb.Connection.from_uri(
    "postgresql://user:pass@localhost/mydb"
)

# Execute query and get Polars DataFrame
df = conn.execute("SELECT * FROM users WHERE age > ?", [18])
print(df)

conn.close()
```

```python [With Config File]
import industrydb as idb

# Load from TOML config
configs = idb.load_config("database.toml")

# Use context manager
with configs["my_postgres"].connect() as conn:
    # CRUD operations
    conn.insert("users", {"name": ["Alice"], "age": [25]})
    df = conn.select("users", where="name = ?", params=["Alice"])
    print(df)
```

```python [DataFrame Operations]
import polars as pl
import industrydb as idb

# Create DataFrame
df = pl.DataFrame({
    "name": ["Alice", "Bob", "Charlie"],
    "age": [25, 30, 35]
})

# Insert DataFrame directly
conn = idb.Connection.from_uri("sqlite://./test.db")
conn.insert("users", df)

# Query back as DataFrame
result = conn.select("users", where="age >= ?", params=[30])
print(result)
```
:::

## Why IndustryDB?

IndustryDB bridges the gap between high-performance Rust code and Python's ease of use. Whether you're building data pipelines, analytics tools, or need efficient database access in Python, IndustryDB provides:

- **Performance**: Rust-compiled code with zero-copy data transfer
- **Simplicity**: Pythonic API that feels natural and intuitive  
- **Flexibility**: Support for multiple databases with a unified interface
- **Safety**: Type-safe operations with comprehensive error handling

## Installation

```bash
pip install industrydb
```

Or from source:

```bash
git clone https://github.com/yourusername/industrydb.git
cd industrydb
uv pip install maturin
uv run maturin develop
```

## Community

- **GitHub**: [yourusername/industrydb](https://github.com/yourusername/industrydb)
- **Issues**: [Report bugs or request features](https://github.com/yourusername/industrydb/issues)
- **License**: [MIT License](https://github.com/yourusername/industrydb/blob/main/LICENSE)

---

<div style="text-align: center; margin-top: 2rem; padding: 1rem;">
  <p style="color: #666;">Built with ❤️ using Rust, Polars, ConnectorX, and PyO3</p>
</div>
