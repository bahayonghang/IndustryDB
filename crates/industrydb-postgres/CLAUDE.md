# industrydb-postgres - PostgreSQL Connector

[Root](../../CLAUDE.md) > [crates](../) > **industrydb-postgres**

## Change Log

### 2025-11-04 18:11:48 - Initial Documentation
- Documented PostgreSQL connector implementation
- Added sqlx integration details
- Documented CRUD operations

---

## Module Responsibilities

The `industrydb-postgres` crate provides a PostgreSQL database connector implementing the traits defined in `industrydb-core`. Key features:

- **Connection management**: Async connection pooling via sqlx
- **Query execution**: Execute SQL and return Polars DataFrames
- **CRUD operations**: High-level insert, select, update, delete operations
- **Type mapping**: PostgreSQL types to Polars DataFrame columns
- **Error handling**: Convert sqlx errors to IndustryDbError

## Entry and Startup

**Main entry**: `src/lib.rs`
```rust
pub use connector::PostgresConnector;
pub use industrydb_core::traits::{CrudOperations, DatabaseConnector};
```

**Module structure**:
```
src/
├── lib.rs          # Public API exports
├── connector.rs    # PostgresConnector struct and DatabaseConnector impl
└── operations.rs   # CrudOperations implementation
```

## External Interfaces

### PostgresConnector

**Constructor**:
```rust
impl PostgresConnector {
    pub async fn new(config: ConnectionConfig) -> Result<Self>
}
```

Creates a new PostgreSQL connector from configuration.

**Trait implementations**:
- `DatabaseConnector` - Basic query execution
- `CrudOperations` - INSERT, SELECT, UPDATE, DELETE

### DatabaseConnector Implementation

```rust
async fn execute(&self, sql: &str) -> Result<DataFrame>
```
Executes raw SQL query and returns results as Polars DataFrame via sqlx.

```rust
async fn is_alive(&self) -> bool
```
Checks connection health using `SELECT 1` query.

```rust
async fn close(&mut self) -> Result<()>
```
Closes the connection pool.

### CrudOperations Implementation

```rust
async fn insert(&self, table: &str, data: DataFrame) -> Result<usize>
```
Generates and executes INSERT statements from DataFrame.

```rust
async fn select(&self, table: &str, columns: Option<&[String]>,
                where_clause: Option<&str>, limit: Option<usize>) -> Result<DataFrame>
```
Builds and executes SELECT query with optional filtering.

## Key Dependencies and Configuration

### Dependencies

**Cargo.toml**:
```toml
[dependencies]
industrydb-core = { path = "../industrydb-core" }
polars = { workspace = true }
sqlx = { workspace = true, features = ["postgres"] }
tokio = { workspace = true }
thiserror = { workspace = true }
async-trait = "0.1"
```

- **sqlx**: PostgreSQL driver with async support
- **industrydb-core**: Core trait definitions
- **polars**: DataFrame operations

### Configuration

**Required fields** in `ConnectionConfig`:
- `db_type`: Must be `DatabaseType::Postgres`
- `host`: PostgreSQL server hostname
- `port`: Port number (default: 5432)
- `database`: Database name
- `username`: Authentication username
- `password`: Authentication password

**Example**:
```rust
let config = ConnectionConfig::postgres(
    "localhost".to_string(),
    5432,
    "mydb".to_string(),
    "user".to_string(),
    "password".to_string(),
);
```

**Connection URI format**:
```
postgresql://username:password@host:port/database
```

## Data Models

### Type Mapping

PostgreSQL to Polars type conversion:

| PostgreSQL Type | Polars Type |
|----------------|-------------|
| INTEGER, BIGINT | Int64 |
| REAL, DOUBLE PRECISION | Float64 |
| TEXT, VARCHAR | Utf8 |
| BOOLEAN | Boolean |
| DATE | Date |
| TIMESTAMP | Datetime |
| JSON, JSONB | Utf8 (JSON string) |

### Query Patterns

**Boolean literals**: `TRUE` / `FALSE`
**Pagination**: `LIMIT N`
**NULL handling**: Polars nullable types

## Testing and Quality

### Testing Strategy

**Unit tests**: Test configuration and error handling
**Integration tests**: Require running PostgreSQL instance

**Mock testing**:
```rust
#[cfg(test)]
mod tests {
    // Tests with mock responses
}
```

### Running Tests

```bash
# Unit tests
cargo test --package industrydb-postgres

# With PostgreSQL running
docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=test postgres:14
cargo test --package industrydb-postgres -- --ignored
```

## Frequently Asked Questions

### Q: How does sqlx connection pooling work?
A: sqlx automatically creates a connection pool. The pool size and behavior can be configured via connection string parameters.

### Q: Are prepared statements used?
A: Yes, sqlx uses prepared statements internally for query parameters, providing SQL injection protection.

### Q: What happens on connection failure?
A: `new()` returns `Err(IndustryDbError::ConnectionError)` with details from sqlx.

### Q: How are transactions handled?
A: Current implementation doesn't expose transactions. This is planned for Phase 2.

### Q: Can I use PostgreSQL-specific features?
A: Yes, via `execute()` method you can use any PostgreSQL SQL including extensions, window functions, CTEs, etc.

## Related File List

**Source files**:
- `src/lib.rs` - Public API (10 lines)
- `src/connector.rs` - Connector implementation
- `src/operations.rs` - CRUD operations

**Build files**:
- `Cargo.toml` - Crate manifest

## Design Patterns

### Async/Await
All database operations are async, utilizing tokio runtime.

### Error Conversion
```rust
impl From<sqlx::Error> for IndustryDbError {
    fn from(err: sqlx::Error) -> Self {
        IndustryDbError::QueryError(err.to_string())
    }
}
```

### Connection Pooling
sqlx automatically manages connection pool lifecycle.

## Integration Points

**Depends on**:
- `industrydb-core` - Trait definitions
- `sqlx` - PostgreSQL driver
- `polars` - DataFrame type

**Used by**:
- `industrydb-py` - Exposed to Python via factory pattern

## Performance Notes

- **Connection pooling**: Reduces connection overhead
- **Prepared statements**: Used by sqlx automatically
- **Async execution**: Non-blocking I/O via tokio
- **Zero-copy**: DataFrame construction minimizes allocations
- **Batch operations**: INSERT supports batch inserts from DataFrame

## PostgreSQL-Specific Notes

### Supported Versions
- PostgreSQL 10+
- Tested with PostgreSQL 14

### Authentication
- Password authentication
- SCRAM-SHA-256 supported
- SSL/TLS connections via rustls

### Extensions
Can use any installed PostgreSQL extensions via `execute()`:
- PostGIS for geospatial
- pg_trgm for fuzzy search
- hstore for key-value
- JSON operators and functions
