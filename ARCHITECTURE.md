# IndustryDB 架构文档

## 概览

IndustryDB 采用**多 crate 分层架构**，通过 trait 抽象实现数据库连接的统一接口。

## 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     Python Layer (用户层)                    │
│                                                               │
│  import industrydb as idb                                     │
│  conn = idb.Connection(config)                                │
│  df = conn.execute("SELECT * FROM table")                     │
└───────────────────────────┬─────────────────────────────────┘
                            │ PyO3 Bindings
┌───────────────────────────▼─────────────────────────────────┐
│              industrydb-py (Python 绑定层)                   │
│                                                               │
│  ┌──────────────────────────────────────────────┐            │
│  │  Connection (create_connector 工厂函数)     │            │
│  │                                               │            │
│  │  match config.db_type {                      │            │
│  │    Postgres => PostgresConnector::new()      │            │
│  │    Sqlite => SqliteConnector::new()          │            │
│  │    Mssql => MssqlConnector::new()            │            │
│  │  }                                            │            │
│  └──────────────────────────────────────────────┘            │
└───────────┬──────────────┬──────────────┬──────────────────┘
            │              │              │
            │              │              │
     ┌──────▼──────┐┌─────▼──────┐┌─────▼──────┐
     │ industrydb- ││ industrydb-││ industrydb-│
     │  postgres   ││   sqlite   ││    mssql   │
     │             ││            ││            │
     │ implements  ││ implements ││ implements │
     │   traits    ││   traits   ││   traits   │
     └──────┬──────┘└─────┬──────┘└─────┬──────┘
            │              │              │
            └──────────────┴──────────────┘
                           │
            ┌──────────────▼──────────────┐
            │      industrydb-core        │
            │      (抽象层 / 核心)        │
            │                              │
            │  ┌────────────────────────┐  │
            │  │ Traits (接口定义)      │  │
            │  │ - DatabaseConnector    │  │
            │  │ - CrudOperations       │  │
            │  └────────────────────────┘  │
            │                              │
            │  ┌────────────────────────┐  │
            │  │ Config (配置管理)      │  │
            │  │ - ConnectionConfig     │  │
            │  │ - DatabaseType         │  │
            │  └────────────────────────┘  │
            │                              │
            │  ┌────────────────────────┐  │
            │  │ Error (错误处理)       │  │
            │  │ - IndustryDbError      │  │
            │  │ - Result<T>            │  │
            │  └────────────────────────┘  │
            └──────────────────────────────┘
                           │
            ┌──────────────▼──────────────┐
            │   External Dependencies     │
            │                              │
            │  - Polars (DataFrame)        │
            │  - ConnectorX (DB Driver)    │
            │  - Arrow (Zero-Copy)         │
            └──────────────────────────────┘
```

## 层次说明

### 1. Python Layer（用户层）
- **职责**：提供 Pythonic API
- **组件**：
  - `Connection` 类：数据库连接
  - `DatabaseConfig` 类：配置管理
  - `load_config()` 函数：配置加载
- **特性**：
  - 上下文管理器支持
  - 完整类型提示（.pyi）
  - 零拷贝 DataFrame 传递

### 2. industrydb-py（绑定层）
- **职责**：Rust ↔ Python 桥接
- **技术**：PyO3
- **功能**：
  - 工厂模式创建连接器
  - 异常映射（Rust Error → Python Exception）
  - DataFrame 转换（Arrow 格式）
  - 参数验证

### 3. 数据库实现层
三个独立的 crate，各自负责一种数据库：

#### industrydb-postgres
- **驱动**：ConnectorX (postgres)
- **特性**：
  - 完整 SQL 支持
  - 布尔值：`TRUE/FALSE`
  - 分页：`LIMIT N`

#### industrydb-sqlite
- **驱动**：ConnectorX (rusqlite)
- **特性**：
  - 嵌入式数据库
  - 布尔值：`1/0`
  - 分页：`LIMIT N`

#### industrydb-mssql
- **驱动**：ConnectorX (tiberius)
- **特性**：
  - SQL Server 支持
  - 布尔值：`1/0`
  - 分页：`SELECT TOP N`（不是 LIMIT）

### 4. industrydb-core（抽象层）
- **职责**：定义统一接口和核心类型
- **组件**：
  - **Traits**：接口定义
  - **Config**：配置类型
  - **Error**：错误类型
  - **Factory**：工厂框架（可选）

### 5. 外部依赖层
- **Polars**：DataFrame 操作
- **ConnectorX**：高性能数据库驱动
- **Arrow**：零拷贝数据格式

## 数据流

### 查询执行流程

```
1. Python 代码
   conn.execute("SELECT * FROM users")
   │
2. PyO3 绑定
   │ PyConnection::execute()
   │
3. 工厂模式
   │ create_connector() 根据类型选择
   │
4. 具体连接器
   │ PostgresConnector::execute()
   │ └─> ConnectorX::get_arrow2()
   │
5. 数据转换
   │ Arrow → Polars DataFrame
   │
6. 返回 Python
   └─> PyDataFrame (零拷贝)
```

### 配置加载流程

```
1. TOML 文件
   database.toml
   │
2. Python 配置加载
   │ load_config("database.toml")
   │ └─> rtoml.load()
   │
3. 验证
   │ validate_config()
   │
4. 创建 Config
   │ DatabaseConfig.from_dict()
   │
5. 传递到 Rust
   │ PyO3 转换
   │
6. 创建连接器
   └─> create_connector(config)
```

## Trait 设计

### DatabaseConnector（基础连接器）

```rust
pub trait DatabaseConnector: Send + Sync {
    /// 获取数据库类型
    fn db_type(&self) -> &str;
    
    /// 执行 SQL 查询
    fn execute(&self, sql: &str) -> Result<DataFrame>;
    
    /// 检查连接是否存活
    fn is_alive(&self) -> bool;
    
    /// 关闭连接
    fn close(&mut self) -> Result<()>;
    
    /// 检查连接是否已关闭
    fn is_closed(&self) -> bool;
}
```

### CrudOperations（CRUD 操作）

```rust
pub trait CrudOperations: DatabaseConnector {
    /// 插入数据
    fn insert(&self, table: &str, data: DataFrame) -> Result<usize>;
    
    /// 查询数据
    fn select(&self, table: &str, ...) -> Result<DataFrame>;
    
    /// 更新数据
    fn update(&self, table: &str, ...) -> Result<usize>;
    
    /// 删除数据
    fn delete(&self, table: &str, ...) -> Result<usize>;
}
```

## 扩展性

### 添加新数据库（如 MySQL）

1. **创建新 crate**
   ```bash
   mkdir -p crates/industrydb-mysql/src
   ```

2. **实现 traits**
   ```rust
   impl DatabaseConnector for MysqlConnector { ... }
   impl CrudOperations for MysqlConnector { ... }
   ```

3. **注册到工厂**
   ```rust
   DatabaseType::Mysql => Ok(Box::new(MysqlConnector::new(config)?))
   ```

4. **更新配置**
   ```rust
   pub enum DatabaseType {
       Postgres,
       Sqlite,
       Mssql,
       Mysql,  // 新增
   }
   ```

### 添加新功能（如事务支持）

1. **定义 trait**
   ```rust
   pub trait TransactionOperations: DatabaseConnector {
       fn begin_transaction(&self) -> Result<Transaction>;
   }
   ```

2. **各数据库实现**
   ```rust
   impl TransactionOperations for PostgresConnector { ... }
   impl TransactionOperations for SqliteConnector { ... }
   impl TransactionOperations for MssqlConnector { ... }
   ```

3. **Python 绑定**
   ```rust
   #[pymethods]
   impl PyConnection {
       fn transaction(&self) -> PyResult<PyTransaction> { ... }
   }
   ```

## 设计原则

### 1. 单一职责原则 (SRP)
- 每个 crate 只负责一件事
- `core`：定义抽象
- `postgres/sqlite/mssql`：具体实现
- `py`：Python 绑定

### 2. 开闭原则 (OCP)
- 对扩展开放：添加新数据库无需修改现有代码
- 对修改封闭：核心接口稳定

### 3. 里氏替换原则 (LSP)
- 所有实现可互相替换
- 通过 trait object 实现多态

### 4. 接口隔离原则 (ISP)
- `DatabaseConnector` 和 `CrudOperations` 分离
- 可以只实现基础连接器

### 5. 依赖倒置原则 (DIP)
- 高层模块（Python 绑定）依赖抽象（traits）
- 低层模块（数据库实现）实现抽象

## 性能优化

### 零拷贝传输
```
Rust DataFrame → Arrow Array → Python DataFrame
         ↑                              ↑
    内存共享                        无序列化
```

### 批量操作
- 批量 INSERT 减少往返次数
- ConnectorX 并行查询分区

### 类型保持
- Arrow 格式保持原始类型
- 无类型转换开销

## 错误处理

### 错误类型层次

```
IndustryDbError (Rust)
├─ ConnectionError → DatabaseConnectionError (Python)
├─ QueryError → QueryExecutionError (Python)
├─ ConfigError → ConfigurationError (Python)
├─ ConnectionClosed → ConnectionClosedError (Python)
└─ ConstraintViolation → ConstraintViolationError (Python)
```

### 错误传播

```
数据库驱动错误
    ↓
IndustryDbError::ConnectorXError
    ↓
PyO3 转换
    ↓
Python Exception
```

## 测试策略

### 单元测试（各 crate）
- `industrydb-core`：配置解析、错误处理
- `industrydb-postgres`：连接器、CRUD
- `industrydb-sqlite`：连接器、CRUD
- `industrydb-mssql`：连接器、CRUD

### 集成测试（Python 层）
- 端到端查询
- CRUD 操作
- 错误处理
- 配置加载

### 性能测试
- DataFrame 转换开销
- 批量操作吞吐
- 内存使用

## 部署考虑

### 系统依赖
- **Linux**: `libkrb5-dev`, `libssl-dev`
- **macOS**: `brew install krb5 openssl`
- **Windows**: Visual Studio Build Tools

### 跨平台编译
- `maturin build --release`
- 为每个平台构建独立 wheel

### Docker 部署
```dockerfile
FROM rust:1.75
RUN apt-get install -y libkrb5-dev libssl-dev
COPY . /app
RUN maturin build --release
```

## 未来规划

### Phase 1（当前）
- ✅ 多 crate 架构
- ✅ 基础 CRUD 操作
- ✅ 三种数据库支持

### Phase 2
- ⏳ 完整参数绑定
- ⏳ 事务支持
- ⏳ 连接池

### Phase 3
- ⏳ 异步 API
- ⏳ 更多数据库（MySQL, Oracle）
- ⏳ 查询构建器

## 参考资料

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [PyO3 User Guide](https://pyo3.rs/)
- [Polars Book](https://pola-rs.github.io/polars-book/)
- [ConnectorX Documentation](https://github.com/sfu-db/connector-x)
