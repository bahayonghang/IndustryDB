# industrydb-core - Core Abstractions

[Root](../../CLAUDE.md) > [crates](../) > **industrydb-core**

## Change Log

### 2025-11-04 18:11:48 - Initial Documentation
- Documented core traits and abstractions
- Added configuration system details
- Documented error handling hierarchy

---

## Module Responsibilities

The `industrydb-core` crate provides the foundational abstractions and types for the entire IndustryDB system. It defines:

- **Traits**: `DatabaseConnector` and `CrudOperations` that all database implementations must follow
- **Configuration**: Type-safe configuration system with validation
- **Error Handling**: Comprehensive error types with proper conversion chains
- **Factory**: Connection factory framework (optional)

This crate has **no database-specific dependencies** and serves as the contract between the Python bindings layer and the concrete database implementations.

## Entry and Startup

**Main entry**: `src/lib.rs`
- Re-exports all public types
- Defines library version constant

**Module structure**:
```
src/
├── lib.rs          # Public API exports
├── traits.rs       # DatabaseConnector and CrudOperations traits
├── config.rs       # Configuration types and parsing
├── error.rs        # Error types and Result alias
└── factory.rs      # Connection factory (optional)
```

## External Interfaces

### Public Traits

#### DatabaseConnector
```rust
pub trait DatabaseConnector: Send + Sync {
    async fn execute(&self, sql: &str) -> Result<DataFrame>;
    async fn is_alive(&self) -> bool;
    async fn close(&mut self) -> Result<()>;
    fn is_closed(&self) -> bool;
    fn db_type(&self) -> &str;
}
```

All database connectors must implement this trait for basic query execution and connection management.

#### CrudOperations
```rust
pub trait CrudOperations: DatabaseConnector {
    async fn insert(&self, table: &str, data: DataFrame) -> Result<usize>;
    async fn select(&self, table: &str, columns: Option<&[String]>,
                    where_clause: Option<&str>, limit: Option<usize>) -> Result<DataFrame>;
    async fn update(&self, table: &str, values: &HashMap<String, String>,
                    where_clause: Option<&str>) -> Result<usize>;
    async fn delete(&self, table: &str, where_clause: Option<&str>) -> Result<usize>;
}
```

Extends `DatabaseConnector` with high-level CRUD operations.

### Configuration Types

#### DatabaseType
```rust
pub enum DatabaseType {
    Postgres,
    Sqlite,
    Mssql,
}
```

Enum representing supported database types with serde serialization support.

#### ConnectionConfig
```rust
pub struct ConnectionConfig {
    pub db_type: DatabaseType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub path: Option<String>,
    pub server: Option<String>,
    pub trusted_connection: Option<bool>,
    pub timeout: Option<u32>,
    pub extra: HashMap<String, serde_json::Value>,
}
```

Type-safe configuration with builder methods:
- `ConnectionConfig::postgres(...)` - PostgreSQL configuration
- `ConnectionConfig::sqlite(...)` - SQLite configuration
- `ConnectionConfig::mssql(...)` - MSSQL configuration
- `validate()` - Validates required fields for database type
- `to_uri()` / `from_uri()` - URI string conversion

## Key Dependencies and Configuration

### Dependencies

**Cargo.toml**:
```toml
[dependencies]
polars = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
thiserror = { workspace = true }
async-trait = "0.1"
tokio = { workspace = true }
anyhow = { workspace = true }
```

- **polars**: DataFrame type used in trait signatures
- **serde**: Serialization for config types
- **thiserror**: Declarative error types
- **async-trait**: Async trait support
- **tokio**: Async runtime

### Configuration

No runtime configuration needed. This is a pure library crate.

## Data Models

### Error Hierarchy

```rust
pub enum IndustryDbError {
    ConnectionError(String),
    QueryError(String),
    ConfigError(String),
    TomlError(toml::de::Error),
    SerializationError(serde_json::Error),
    IoError(std::io::Error),
    ConnectorXError(String),
    PolarsError(String),
    UnsupportedDatabase(String),
    ConnectionClosed,
    Timeout(String),
    ConstraintViolation(String),
    InvalidParameter(String),
    NotImplemented(String),
}
```

Each variant includes:
- Descriptive error message
- Automatic conversion from common error types
- Helper constructors like `connection_error()`, `query_error()`

### OperationResult

```rust
pub struct OperationResult {
    pub rows_affected: usize,
    pub success: bool,
    pub message: Option<String>,
}
```

Represents the result of CRUD operations.

## Testing and Quality

### Unit Tests

**Location**: `src/config.rs`, `src/error.rs`

**Test coverage**:
- Config creation and validation
- URI generation and parsing
- Error type conversions
- Database type string parsing

**Run tests**:
```bash
cargo test --package industrydb-core
```

### Quality Tools

- **Clippy**: `cargo clippy --package industrydb-core`
- **Format**: `cargo fmt --package industrydb-core`
- **Documentation**: `cargo doc --package industrydb-core --open`

## Frequently Asked Questions

### Q: Why are traits async?
A: All database operations are inherently I/O-bound, so async traits enable non-blocking execution and better concurrency.

### Q: Why separate DatabaseConnector and CrudOperations?
A: Interface segregation. Some use cases may only need basic query execution without CRUD helpers.

### Q: How do I add a new database type?
A:
1. Add variant to `DatabaseType` enum
2. Update `FromStr` and `Display` implementations
3. Add validation logic in `ConnectionConfig::validate()`
4. Create new crate implementing the traits

### Q: Can I use this crate standalone?
A: Yes, but you'll need to provide your own trait implementations. The database-specific crates depend on this.

## Related File List

**Core files**:
- `src/lib.rs` - Public API
- `src/traits.rs` - Trait definitions (85 lines)
- `src/config.rs` - Configuration types (372 lines)
- `src/error.rs` - Error types (128 lines)
- `src/factory.rs` - Factory pattern implementation

**Build files**:
- `Cargo.toml` - Crate manifest

**Tests**:
- Inline unit tests in each module

## Design Patterns

### Trait-Based Architecture
Uses traits to define contracts, enabling polymorphism and testability.

### Builder Pattern
`ConnectionConfig` provides type-specific constructors for ergonomic configuration.

### Error Conversion Chain
Automatic `From` implementations for common error types enable `?` operator usage.

## Integration Points

**Used by**:
- `industrydb-postgres` - Implements traits for PostgreSQL
- `industrydb-sqlite` - Implements traits for SQLite
- `industrydb-mssql` - Implements traits for MSSQL
- `industrydb-py` - Exposes types to Python via PyO3

**Dependencies**:
- `polars` - DataFrame type
- `tokio` - Async runtime

## Performance Notes

- Configuration validation is performed once at creation time
- Error types use `String` for messages (allocates on error path only)
- Traits use zero-cost abstractions (monomorphization at compile time)
- Async overhead is minimal with tokio runtime
