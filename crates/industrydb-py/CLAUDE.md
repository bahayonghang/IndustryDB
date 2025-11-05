# industrydb-py - Python Bindings

[Root](../../CLAUDE.md) > [crates](../) > **industrydb-py**

## Change Log

### 2025-11-04 18:11:48 - Initial Documentation
- Documented PyO3 bindings architecture
- Added exception mapping details
- Documented Python API surface

---

## Module Responsibilities

The `industrydb-py` crate provides Python bindings for IndustryDB using PyO3. This is the bridge between Rust implementations and Python users. Key features:

- **PyO3 bindings**: Expose Rust functionality to Python
- **Factory pattern**: Create appropriate connector based on database type
- **Exception mapping**: Convert Rust errors to Python exceptions
- **DataFrame conversion**: Zero-copy transfer via Arrow format
- **Type stubs**: Comprehensive .pyi files for IDE support

## Entry and Startup

**Main entry**: `src/lib.rs`
```rust
#[pymodule]
fn industrydb(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyDatabaseConfig>()?;
    m.add_class::<PyConnection>()?;
    // ... exception classes
    Ok(())
}
```

**Module structure**:
```
src/
├── lib.rs          # PyO3 module definition
├── config.rs       # PyDatabaseConfig wrapper
├── connection.rs   # PyConnection wrapper
└── errors.rs       # Python exception types
```

**Build artifact**: `libindustrydb.so` (Linux), `industrydb.pyd` (Windows), `industrydb.dylib` (macOS)

## External Interfaces

### Python Classes

#### PyDatabaseConfig
```python
class DatabaseConfig:
    def __init__(self, db_type: str, **kwargs) -> None: ...

    @staticmethod
    def from_dict(config: Dict[str, Any]) -> DatabaseConfig: ...

    @staticmethod
    def from_uri(uri: str) -> DatabaseConfig: ...

    def to_dict(self) -> Dict[str, Any]: ...
    def to_uri(self) -> str: ...
```

Python wrapper around Rust `ConnectionConfig`.

#### PyConnection
```python
class Connection:
    def __init__(self, config: DatabaseConfig) -> None: ...

    @staticmethod
    def connect(config: DatabaseConfig) -> Connection: ...

    @staticmethod
    def from_uri(uri: str) -> Connection: ...

    def execute(self, sql: str, params: Optional[List[Any]] = None) -> pl.DataFrame: ...
    def insert(self, table: str, data: Union[pl.DataFrame, Dict]) -> int: ...
    def select(self, table: str, ...) -> pl.DataFrame: ...
    def update(self, table: str, ...) -> int: ...
    def delete(self, table: str, ...) -> int: ...

    def close(self) -> None: ...
    def is_closed(self) -> bool: ...

    def __enter__(self) -> Connection: ...
    def __exit__(self, ...) -> None: ...
```

Python wrapper managing underlying Rust connector via factory pattern.

### Exception Hierarchy

```python
class IndustryDbError(Exception): ...
class DatabaseConnectionError(IndustryDbError): ...
class QueryExecutionError(IndustryDbError): ...
class ConfigurationError(IndustryDbError): ...
class ConnectionClosedError(IndustryDbError): ...
class ConstraintViolationError(IndustryDbError): ...
```

## Key Dependencies and Configuration

### Dependencies

**Cargo.toml**:
```toml
[lib]
name = "industrydb"
crate-type = ["cdylib"]

[dependencies]
industrydb-core = { path = "../industrydb-core" }
industrydb-postgres = { path = "../industrydb-postgres" }
industrydb-sqlite = { path = "../industrydb-sqlite" }
industrydb-mssql = { path = "../industrydb-mssql" }
pyo3 = { workspace = true }
polars = { workspace = true }
pythonize = "0.21"
tokio = { workspace = true }
serde_json = "1.0"
```

- **pyo3**: Python bindings framework (v0.21)
- **pythonize**: Serde to Python conversion
- **All connector crates**: Unified access to all databases

### Build Configuration

**pyproject.toml** (root):
```toml
[tool.maturin]
manifest-path = "crates/industrydb-py/Cargo.toml"
python-source = "python"
module-name = "industrydb.industrydb"
```

Maturin builds this crate as a Python extension module.

## Data Flow

### Query Execution Flow

```
Python: conn.execute("SELECT * FROM users")
  ↓
PyO3: PyConnection::execute()
  ↓
Factory: Match db_type → PostgresConnector/SqliteConnector/MssqlConnector
  ↓
Trait: DatabaseConnector::execute()
  ↓
Driver: sqlx/tiberius query execution
  ↓
DataFrame: Construct Polars DataFrame
  ↓
Arrow: Share memory via Arrow arrays
  ↓
PyO3: Convert to Python PyDataFrame
  ↓
Python: Return pl.DataFrame (zero-copy)
```

### Exception Flow

```
Rust Error: IndustryDbError::ConnectionError
  ↓
PyO3 conversion: errors::convert_error()
  ↓
Python Exception: DatabaseConnectionError
  ↓
User code: try/except handling
```

## Testing and Quality

### Testing Strategy

**Unit tests**: Test PyO3 bindings, error conversion
**Integration tests**: Python-based tests in `tests/test_basic.py`

### Running Tests

```bash
# Build extension module
maturin develop

# Run Python tests
pytest tests/ -v

# Type checking
mypy python/industrydb
```

## Frequently Asked Questions

### Q: Why use cdylib instead of regular lib?
A: `cdylib` produces a C-compatible dynamic library required for Python extension modules.

### Q: How is the async runtime managed?
A: PyO3 blocks on async operations using `tokio::runtime::Runtime::new().block_on()`.

### Q: Can I use multiple databases in one Python process?
A: Yes, create separate `Connection` objects with different configs.

### Q: What happens to Python GIL?
A: PyO3 releases GIL during blocking Rust operations, allowing parallelism.

### Q: How are DataFrames transferred?
A: Via Apache Arrow format with shared memory (zero-copy).

### Q: Can I extend this with custom connectors?
A: Not from Python side. Add new Rust crate implementing traits, then update factory.

## Related File List

**Source files**:
- `src/lib.rs` - PyO3 module definition (53 lines)
- `src/config.rs` - PyDatabaseConfig wrapper
- `src/connection.rs` - PyConnection wrapper
- `src/errors.rs` - Exception types

**Build files**:
- `Cargo.toml` - Crate manifest (cdylib)

**Type stubs**:
- `python/industrydb/industrydb.pyi` - Type hints (261 lines)

## Design Patterns

### Factory Pattern
```rust
fn create_connector(config: ConnectionConfig) -> Result<Box<dyn DatabaseConnector>> {
    match config.db_type {
        DatabaseType::Postgres => Ok(Box::new(PostgresConnector::new(config)?)),
        DatabaseType::Sqlite => Ok(Box::new(SqliteConnector::new(config)?)),
        DatabaseType::Mssql => Ok(Box::new(MssqlConnector::new(config)?)),
    }
}
```

### Wrapper Pattern
Python classes wrap Rust types, exposing only safe, Pythonic API.

### Error Conversion
```rust
impl From<IndustryDbError> for PyErr {
    fn from(err: IndustryDbError) -> PyErr {
        match err {
            IndustryDbError::ConnectionError(msg) =>
                DatabaseConnectionError::new_err(msg),
            // ... other variants
        }
    }
}
```

## Integration Points

**Depends on**:
- `industrydb-core` - Core types
- `industrydb-postgres` - PostgreSQL connector
- `industrydb-sqlite` - SQLite connector
- `industrydb-mssql` - MSSQL connector
- `pyo3` - Python bindings
- `polars` - DataFrame type

**Produces**:
- Python extension module: `industrydb.industrydb`

**Used by**:
- `python/industrydb` - Pure Python wrapper and utilities

## Performance Notes

- **Zero-copy**: DataFrame transfer via Arrow shared memory
- **GIL release**: Long operations release Python GIL
- **Tokio runtime**: Efficient async I/O handling
- **Compiled code**: Release builds are fully optimized
- **No serialization**: Direct memory sharing for DataFrames

## PyO3 Integration Details

### Version Compatibility
- PyO3 version: 0.21
- Python ABI: abi3-py38 (supports Python 3.8+)
- pyo3-polars: 0.15

### Memory Management
- Python objects managed by reference counting
- Rust objects owned by PyO3 wrapper structs
- DataFrames use Arrow shared memory

### Threading Model
- Python: GIL-based threading
- Rust: Tokio async runtime
- Bridge: PyO3 releases GIL during Rust operations

### Type Conversion

| Rust Type | Python Type |
|-----------|-------------|
| `String` | `str` |
| `i64` | `int` |
| `f64` | `float` |
| `bool` | `bool` |
| `Vec<T>` | `List[T]` |
| `HashMap<K,V>` | `Dict[K,V]` |
| `DataFrame` | `polars.DataFrame` |
| `Result<T>` | T or Exception |

### Building with Maturin

```bash
# Development build
maturin develop

# Release build
maturin build --release

# Build wheel for distribution
maturin build --release --strip
```

### Debugging

**Enable debug symbols**:
```bash
maturin develop --release --strip=false
```

**Python-side debugging**:
```python
import logging
logging.basicConfig(level=logging.DEBUG)
```

**Rust-side logging**:
```rust
env_logger::init();
log::debug!("Debug message");
```
