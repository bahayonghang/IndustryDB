# industrydb-sqlite - SQLite Connector

[Root](../../CLAUDE.md) > [crates](../) > **industrydb-sqlite**

## Change Log

### 2025-11-04 18:11:48 - Initial Documentation
- Documented SQLite connector implementation
- Added sqlx integration details
- Documented embedded database features

---

## Module Responsibilities

The `industrydb-sqlite` crate provides a SQLite database connector implementing the traits defined in `industrydb-core`. Key features:

- **Embedded database**: File-based or in-memory SQLite databases
- **Async operations**: Async wrapper around synchronous SQLite
- **CRUD operations**: High-level insert, select, update, delete operations
- **Type mapping**: SQLite types to Polars DataFrame columns
- **Transaction safety**: ACID guarantees via SQLite

## Entry and Startup

**Main entry**: `src/lib.rs`
```rust
pub use connector::SqliteConnector;
pub use industrydb_core::traits::{CrudOperations, DatabaseConnector};
```

**Module structure**:
```
src/
├── lib.rs          # Public API exports
├── connector.rs    # SqliteConnector struct and DatabaseConnector impl
└── operations.rs   # CrudOperations implementation
```

## External Interfaces

### SqliteConnector

**Constructor**:
```rust
impl SqliteConnector {
    pub async fn new(config: ConnectionConfig) -> Result<Self>
}
```

Creates a new SQLite connector from configuration.

**Supported paths**:
- File path: `./data.db`, `/absolute/path/to/db.sqlite`
- In-memory: `:memory:`

**Trait implementations**:
- `DatabaseConnector` - Basic query execution
- `CrudOperations` - INSERT, SELECT, UPDATE, DELETE

### DatabaseConnector Implementation

```rust
async fn execute(&self, sql: &str) -> Result<DataFrame>
```
Executes SQL query and returns results as Polars DataFrame.

```rust
async fn is_alive(&self) -> bool
```
Checks connection health using `SELECT 1` query.

```rust
async fn close(&mut self) -> Result<()>
```
Closes the database connection.

### CrudOperations Implementation

Similar to PostgreSQL connector but with SQLite-specific SQL generation.

## Key Dependencies and Configuration

### Dependencies

**Cargo.toml**:
```toml
[dependencies]
industrydb-core = { path = "../industrydb-core" }
polars = { workspace = true }
sqlx = { workspace = true, features = ["sqlite"] }
tokio = { workspace = true }
thiserror = { workspace = true }
async-trait = "0.1"
```

- **sqlx**: SQLite driver with async support
- **industrydb-core**: Core trait definitions
- **polars**: DataFrame operations

### Configuration

**Required fields** in `ConnectionConfig`:
- `db_type`: Must be `DatabaseType::Sqlite`
- `path`: Database file path or `:memory:`

**Example**:
```rust
// File-based database
let config = ConnectionConfig::sqlite("./mydata.db");

// In-memory database
let config = ConnectionConfig::sqlite(":memory:");
```

**Connection URI format**:
```
sqlite://path/to/database.db
sqlite://:memory:
```

## Data Models

### Type Mapping

SQLite to Polars type conversion:

| SQLite Type | Storage Class | Polars Type |
|-------------|--------------|-------------|
| INTEGER | INTEGER | Int64 |
| REAL | REAL | Float64 |
| TEXT | TEXT | Utf8 |
| BLOB | BLOB | Binary |
| NULL | NULL | Null |

**Note**: SQLite uses dynamic typing, so actual types are inferred from values.

### SQLite-Specific Features

**Boolean representation**:
- SQLite has no native BOOLEAN type
- Uses INTEGER: `0` for false, `1` for true

**Date/Time**:
- Stored as TEXT (ISO 8601) or INTEGER (Unix timestamp)
- Polars parses TEXT dates automatically

**Pagination**:
- Uses `LIMIT N` syntax (same as PostgreSQL)

## Testing and Quality

### Testing Strategy

**Unit tests**: File-based and in-memory database tests
**Integration tests**: Can run without external dependencies

**Test example**:
```rust
#[tokio::test]
async fn test_inmemory_db() {
    let config = ConnectionConfig::sqlite(":memory:");
    let connector = SqliteConnector::new(config).await.unwrap();
    // ... test operations
}
```

### Running Tests

```bash
# All tests (no external dependencies needed)
cargo test --package industrydb-sqlite

# With temp directory cleanup
cargo test --package industrydb-sqlite -- --test-threads=1
```

## Frequently Asked Questions

### Q: Can I use SQLite in production?
A: Yes, for read-heavy workloads or embedded use cases. Not recommended for high-concurrency writes.

### Q: How are concurrent writes handled?
A: SQLite uses file locking. Writes are serialized. Consider Write-Ahead Logging (WAL) mode for better concurrency.

### Q: What's the maximum database size?
A: Theoretical limit is 281 TB, practical limit depends on filesystem.

### Q: Are foreign keys enforced?
A: Only if explicitly enabled: `PRAGMA foreign_keys = ON;`

### Q: Can I use in-memory database across connections?
A: In-memory databases are per-connection. Use shared cache mode for multi-connection access.

## Related File List

**Source files**:
- `src/lib.rs` - Public API (10 lines)
- `src/connector.rs` - Connector implementation
- `src/operations.rs` - CRUD operations

**Build files**:
- `Cargo.toml` - Crate manifest

## Design Patterns

### Async Wrapper
SQLite is synchronous but wrapped in async interface for consistency.

### File-Based Storage
Database persisted to single file, making backup and deployment simple.

### Connection Pooling
sqlx manages connection pool even for SQLite (useful for concurrency).

## Integration Points

**Depends on**:
- `industrydb-core` - Trait definitions
- `sqlx` - SQLite driver (uses rusqlite internally)
- `polars` - DataFrame type

**Used by**:
- `industrydb-py` - Exposed to Python via factory pattern

## Performance Notes

- **In-memory mode**: Fastest, data lost on close
- **File mode**: Persistent but slower than in-memory
- **WAL mode**: Improves concurrent read performance
- **Page size**: Default 4KB, can be tuned
- **Cache size**: Configurable via PRAGMA
- **No network overhead**: Embedded database

## SQLite-Specific Notes

### Supported Versions
- SQLite 3.35+
- Bundled or system SQLite via sqlx features

### Performance Tuning

**Enable WAL mode**:
```sql
PRAGMA journal_mode = WAL;
```

**Increase cache**:
```sql
PRAGMA cache_size = -64000;  -- 64MB
```

**Disable synchronous for speed** (less safe):
```sql
PRAGMA synchronous = NORMAL;
```

### Limitations

- No RIGHT JOIN or FULL OUTER JOIN
- No stored procedures
- Limited ALTER TABLE support
- No user authentication (file permissions only)
- Single writer at a time (without WAL)

### Best Practices

1. **Use WAL mode** for better concurrency
2. **Enable foreign keys** if using relationships
3. **Vacuum regularly** to reclaim space
4. **Use transactions** for bulk inserts
5. **Close connections** properly to avoid locks
