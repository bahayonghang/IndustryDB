# IndustryDB

High-performance database middleware powered by Rust and Polars.

## Features

- 🚀 **Blazing Fast**: Built with Rust for maximum performance
- 🔌 **Multi-Database**: Support for PostgreSQL, SQLite, and MSSQL
- 📊 **Polars Integration**: Native DataFrame support with zero-copy data transfer
- 🛡️ **Type Safe**: Comprehensive type hints for Python
- 🔧 **Easy to Use**: Simple, Pythonic API

## Supported Databases

- **PostgreSQL** - Full support via ConnectorX
- **SQLite** - Full support via ConnectorX
- **MSSQL** - Full support via ConnectorX

## Installation

### From PyPI (Coming Soon)

```bash
pip install industrydb
```

### From Source

Requirements:
- Rust 1.70+ (install from https://rustup.rs/)
- Python 3.8+
- uv (recommended) or pip

```bash
# Clone the repository
git clone https://github.com/yourusername/industrydb.git
cd industrydb

# Install with uv (recommended)
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
uv pip install maturin
uv run maturin develop

# Or install with pip
pip install maturin
maturin develop
```

## Quick Start

### Using TOML Configuration

Create a `database.toml` file:

```toml
[connections.my_postgres]
type = "postgres"
host = "localhost"
port = 5432
database = "mydb"
username = "user"
password = "password"

[connections.my_sqlite]
type = "sqlite"
path = "./data.db"
```

Python code:

```python
import industrydb as idb

# Load configurations
configs = idb.load_config("database.toml")

# Connect to PostgreSQL
with configs["my_postgres"].connect() as conn:
    # Execute SQL and get Polars DataFrame
    df = conn.execute("SELECT * FROM users WHERE age > ?", [18])
    print(df)
    
    # CRUD operations
    conn.insert("users", {"name": ["Alice"], "age": [25]})
    df = conn.select("users", where="name = ?", params=["Alice"])
    conn.update("users", {"age": 26}, where="name = ?", params=["Alice"])
    conn.delete("users", where="age < ?", params=[18])
```

### Using URI Connection Strings

```python
import industrydb as idb

# Connect using URI
conn = idb.Connection.from_uri("postgresql://user:pass@localhost:5432/mydb")

# Execute query
df = conn.execute("SELECT * FROM products LIMIT 10")
print(df)

conn.close()
```

### Using DataFrame Operations

```python
import polars as pl
import industrydb as idb

# Create a DataFrame
df = pl.DataFrame({
    "id": [1, 2, 3],
    "name": ["Alice", "Bob", "Charlie"],
    "age": [25, 30, 35]
})

# Connect and insert
conn = idb.Connection.from_uri("sqlite://./test.db")
conn.insert("users", df)

# Query back
result = conn.select("users", where="age >= ?", params=[30])
print(result)
```

## Development

### Using just (Recommended)

IndustryDB uses [just](https://github.com/casey/just) for common development tasks:

```bash
# Install just
cargo install just

# Setup environment
just setup
source .venv/bin/activate

# Sync dependencies
just sync

# Build and develop
just develop

# Run tests
just test

# Complete dev workflow
just dev

# See all commands
just --list
```

See [JUSTFILE.md](JUSTFILE.md) for complete documentation.

### Manual Setup

```bash
# Install development dependencies
uv pip install -e ".[dev]"

# Or with pip
pip install -e ".[dev]"
```

### Build and Test

```bash
# Build the project
maturin develop

# Run Rust tests
cargo test

# Run Python tests
pytest

# Type checking
mypy python/industrydb

# Linting
ruff check python/
```

### Build Wheel

```bash
# Build release wheel
maturin build --release

# Build wheel for specific Python version
maturin build --release --interpreter python3.11
```

## Architecture

IndustryDB follows a modular multi-crate architecture with each database having its own implementation:

```
industrydb/
├── crates/
│   ├── industrydb-core/      # Core abstractions (traits, config, errors)
│   ├── industrydb-postgres/  # PostgreSQL implementation
│   ├── industrydb-sqlite/    # SQLite implementation
│   ├── industrydb-mssql/     # MSSQL implementation
│   └── industrydb-py/        # Python bindings (PyO3)
├── python/
│   └── industrydb/           # Python package
│       ├── __init__.py
│       ├── config.py         # Configuration utilities
│       ├── industrydb.pyi    # Type stubs
│       └── py.typed          # PEP 561 marker
├── Cargo.toml                # Rust workspace
└── pyproject.toml            # Python package config
```

### Key Design Decisions

- **Multi-Crate Architecture**: Each database has its own crate for better modularity
- **Factory Pattern**: Python bindings use factory pattern to create appropriate connectors
- **Trait-Based Design**: Core defines `DatabaseConnector` and `CrudOperations` traits
- **ConnectorX**: Used for database connectivity with native Arrow support
- **Zero-Copy**: Data transfer via Arrow format between Rust and Python
- **Separation of Concerns**: Core abstractions, database implementations, and bindings are separate
- **Type Safety**: Comprehensive type stubs for excellent IDE support

## Performance

IndustryDB leverages Rust's performance and Polars' efficient data structures:

- Zero-copy data transfer via Apache Arrow
- Parallel query execution via ConnectorX
- Lazy evaluation for query optimization
- Compiled Rust code for minimal overhead

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Credits

Built with:
- [Rust](https://www.rust-lang.org/)
- [Polars](https://www.pola.rs/)
- [ConnectorX](https://github.com/sfu-db/connector-x)
- [PyO3](https://pyo3.rs/)
- [Maturin](https://www.maturin.rs/)
