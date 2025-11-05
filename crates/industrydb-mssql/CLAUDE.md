# industrydb-mssql - Microsoft SQL Server Connector

[Root](../../CLAUDE.md) > [crates](../) > **industrydb-mssql**

## Change Log

### 2025-11-04 18:11:48 - Initial Documentation
- Documented MSSQL connector implementation
- Added Tiberius driver integration details
- Documented SQL Server specific features

---

## Module Responsibilities

The `industrydb-mssql` crate provides a Microsoft SQL Server database connector implementing the traits defined in `industrydb-core`. Key features:

- **SQL Server connectivity**: Via Tiberius native driver
- **Connection pooling**: Using bb8 connection pool
- **CRUD operations**: High-level insert, select, update, delete operations
- **Type mapping**: SQL Server types to Polars DataFrame columns
- **TDS protocol**: Native TDS 7.3+ support
- **Authentication**: SQL Server and Windows authentication

## Entry and Startup

**Main entry**: `src/lib.rs`
```rust
pub use connector::MssqlConnector;
pub use industrydb_core::traits::{CrudOperations, DatabaseConnector};
```

**Module structure**:
```
src/
├── lib.rs          # Public API exports
├── connector.rs    # MssqlConnector struct and DatabaseConnector impl
└── operations.rs   # CrudOperations implementation
```

## External Interfaces

### MssqlConnector

**Constructor**:
```rust
impl MssqlConnector {
    pub async fn new(config: ConnectionConfig) -> Result<Self>
}
```

Creates a new MSSQL connector with connection pooling.

**Trait implementations**:
- `DatabaseConnector` - Basic query execution
- `CrudOperations` - INSERT, SELECT, UPDATE, DELETE

### DatabaseConnector Implementation

```rust
async fn execute(&self, sql: &str) -> Result<DataFrame>
```
Executes T-SQL query and returns results as Polars DataFrame.

```rust
async fn is_alive(&self) -> bool
```
Checks connection health using `SELECT 1` query.

```rust
async fn close(&mut self) -> Result<()>
```
Closes connection pool.

### CrudOperations Implementation

**Important**: MSSQL uses different SQL syntax:
- Pagination: `SELECT TOP N` instead of `LIMIT N`
- Boolean literals: `1`/`0` instead of `TRUE`/`FALSE`

## Key Dependencies and Configuration

### Dependencies

**Cargo.toml**:
```toml
[dependencies]
industrydb-core = { path = "../industrydb-core" }
polars = { workspace = true }
tiberius = { version = "0.12", features = ["chrono", "tds73"] }
tokio = { workspace = true }
tokio-util = { version = "0.7", features = ["compat"] }
thiserror = { workspace = true }
async-trait = "0.1"
bb8 = "0.8"
bb8-tiberius = "0.15"
```

- **tiberius**: Pure Rust TDS client
- **bb8**: Async connection pool
- **bb8-tiberius**: Tiberius pool adapter
- **tokio-util**: Compatibility layer

### Configuration

**Required fields** in `ConnectionConfig`:
- `db_type`: Must be `DatabaseType::Mssql`
- `server` or `host`: SQL Server hostname/IP
- `database`: Database name
- `username`: SQL Server username (unless using Windows auth)
- `password`: SQL Server password (unless using Windows auth)

**Optional fields**:
- `trusted_connection`: Use Windows authentication
- `port`: Port number (default: 1433)
- `timeout`: Connection timeout in seconds

**Example**:
```rust
// SQL Server authentication
let config = ConnectionConfig::mssql(
    "localhost".to_string(),
    "mydb".to_string(),
    "sa".to_string(),
    "Password123".to_string(),
);

// Windows authentication
let mut config = ConnectionConfig {
    db_type: DatabaseType::Mssql,
    server: Some("localhost".to_string()),
    database: Some("mydb".to_string()),
    trusted_connection: Some(true),
    ..Default::default()
};
```

**Connection string format**:
```
Server=hostname;Database=dbname;User Id=user;Password=pass
```

## Data Models

### Type Mapping

SQL Server to Polars type conversion:

| SQL Server Type | Polars Type |
|----------------|-------------|
| INT, BIGINT | Int64 |
| FLOAT, REAL | Float64 |
| VARCHAR, NVARCHAR, TEXT | Utf8 |
| BIT | Boolean (via 0/1) |
| DATE | Date |
| DATETIME, DATETIME2 | Datetime |
| UNIQUEIDENTIFIER | Utf8 (UUID string) |
| DECIMAL, NUMERIC | Float64 |

### SQL Server-Specific Syntax

**Boolean literals**: `1` (true), `0` (false)
**Pagination**: `SELECT TOP N ...` or `OFFSET-FETCH`
**String concatenation**: `+` operator
**Schema qualification**: `[dbo].[TableName]`

## Testing and Quality

### Testing Strategy

**Unit tests**: Mock tests for query generation
**Integration tests**: Require running SQL Server instance

**Docker setup**:
```bash
docker run -d -p 1433:1433 \
  -e 'ACCEPT_EULA=Y' \
  -e 'SA_PASSWORD=YourStrong@Password' \
  mcr.microsoft.com/mssql/server:2022-latest
```

### Running Tests

```bash
# Unit tests
cargo test --package industrydb-mssql

# Integration tests (requires SQL Server)
export MSSQL_HOST=localhost
export MSSQL_PASSWORD='YourStrong@Password'
cargo test --package industrydb-mssql -- --ignored
```

## Frequently Asked Questions

### Q: What SQL Server versions are supported?
A: SQL Server 2012+ and Azure SQL Database (TDS 7.3+).

### Q: Can I use Windows Authentication?
A: Yes, set `trusted_connection: true` in configuration. Requires GSSAPI on Linux/macOS.

### Q: How is connection pooling implemented?
A: Using bb8 async connection pool with configurable min/max connections.

### Q: Are stored procedures supported?
A: Yes, use `execute()` with `EXEC` statement: `EXEC MyProc @param1 = 'value'`

### Q: What about transactions?
A: Not yet exposed in the API. Planned for Phase 2.

### Q: Can I use Azure SQL Database?
A: Yes, same configuration with Azure hostname.

## Related File List

**Source files**:
- `src/lib.rs` - Public API (10 lines)
- `src/connector.rs` - Connector implementation
- `src/operations.rs` - CRUD operations

**Build files**:
- `Cargo.toml` - Crate manifest

## Design Patterns

### Connection Pooling
bb8 manages pool of Tiberius connections with automatic reconnection.

### Async I/O
Tiberius uses tokio for async I/O with TDS protocol.

### Error Conversion
```rust
impl From<tiberius::error::Error> for IndustryDbError {
    fn from(err: tiberius::error::Error) -> Self {
        IndustryDbError::QueryError(err.to_string())
    }
}
```

## Integration Points

**Depends on**:
- `industrydb-core` - Trait definitions
- `tiberius` - SQL Server TDS driver
- `bb8` - Connection pooling
- `polars` - DataFrame type

**Used by**:
- `industrydb-py` - Exposed to Python via factory pattern

## Performance Notes

- **Connection pooling**: Reduces connection overhead significantly
- **Prepared statements**: Supported by Tiberius
- **Bulk operations**: Batch inserts via BULK INSERT possible
- **TDS protocol**: Efficient binary protocol
- **Network optimization**: Packet size configurable

## SQL Server-Specific Notes

### Authentication Methods

**SQL Server Authentication**:
- Standard username/password
- Works on all platforms

**Windows Authentication**:
- Uses GSSAPI/Kerberos
- Requires system configuration on Linux/macOS:
  ```bash
  # Ubuntu/Debian
  sudo apt-get install libkrb5-dev
  ```

### System Requirements

**Linux**:
- GSSAPI libraries for Windows auth
- OpenSSL for TLS connections

**macOS**:
```bash
brew install krb5 openssl
export PKG_CONFIG_PATH="/usr/local/opt/krb5/lib/pkgconfig:$PKG_CONFIG_PATH"
```

**Windows**:
- Native support via SSPI

### Performance Tuning

**Connection pool settings**:
```rust
// In future versions, configure pool size
bb8::Pool::builder()
    .max_size(15)
    .build(manager)
```

**Query optimization**:
- Use schema-qualified names: `dbo.TableName`
- Create appropriate indexes
- Use `SET NOCOUNT ON` to reduce network traffic
- Consider query hints for complex queries

### Known Limitations

- **Transaction isolation**: Not yet exposed in API
- **CLR types**: Not supported
- **XML type**: Returned as string
- **Spatial types**: Not yet supported
- **Multiple result sets**: Returns first result set only

### Best Practices

1. **Use parameterized queries** to prevent SQL injection
2. **Schema qualify** table names for performance
3. **Set appropriate timeouts** for long-running queries
4. **Monitor connection pool** usage
5. **Use TOP** instead of LIMIT for pagination
6. **Consider columnstore indexes** for analytics workloads
