# 特性

IndustryDB 提供全面的功能集，用于高性能数据库操作。

## 核心特性

### 🚀 极速性能

采用 Rust 构建，实现极致速度：

- **原生编译代码**：无 Python 解释器开销
- **零拷贝数据传输**：通过 Apache Arrow 直接内存映射
- **并行执行**：ConnectorX 多线程查询处理
- **SIMD 优化**：Polars 中的向量化操作

**性能基准**：
- 典型工作负载下比 SQLAlchemy 快 2-5 倍
- 接近原生查询执行速度
- 最小内存开销

### 🔌 多数据库支持

跨数据库的统一接口：

```python
import industrydb as idb

# 所有数据库使用相同 API
pg_conn = idb.Connection.from_uri("postgresql://...")
sqlite_conn = idb.Connection.from_uri("sqlite://...")
mssql_conn = idb.Connection.from_uri("mssql://...")

# 完全相同的操作
for conn in [pg_conn, sqlite_conn, mssql_conn]:
    df = conn.select("users", where="age > ?", params=[18])
```

支持的数据库：
- **PostgreSQL** 9.5+
- **SQLite** 3.x
- **Microsoft SQL Server** 2012+

### 📊 原生 Polars 集成

一流的 DataFrame 支持：

```python
import polars as pl
import industrydb as idb

# 创建 DataFrame
df = pl.DataFrame({
    "name": ["Alice", "Bob"],
    "age": [25, 30]
})

# 直接插入 - 无需转换
conn.insert("users", df)

# 查询返回 Polars DataFrame
result = conn.select("users")

# 链式 Polars 操作
result.filter(pl.col("age") > 25).sort("name")
```

优势：
- **零转换开销**：直接 Arrow → Polars
- **类型保持**：列类型保持不变
- **惰性求值**：优化查询链
- **丰富操作**：完整的 Polars API 可用

### 🛡️ 类型安全

全面的类型提示，提供出色的 IDE 支持：

```python
import industrydb as idb
import polars as pl

# 完整类型检查
conn: idb.Connection = idb.Connection.from_uri("...")
df: pl.DataFrame = conn.execute("SELECT * FROM users")

# IDE 自动补全
conn.select(  # IDE 显示所有参数
    table="users",
    columns=["name", "age"],
    where="age > ?",
    params=[18]
)
```

特点：
- 完整的 `.pyi` 类型桩文件
- MyPy 兼容
- PyRight 支持
- 运行时类型验证

### 🔧 简洁 API

Pythonic 且直观：

```python
import industrydb as idb

# 上下文管理器支持
with idb.Connection.from_uri("sqlite://./db.db") as conn:
    # CRUD 操作
    conn.insert("users", {"name": ["Alice"], "age": [25]})
    df = conn.select("users")
    conn.update("users", {"age": 26}, where="name = ?", params=["Alice"])
    conn.delete("users", where="age < ?", params=[18])
    
# 自动清理 - 连接关闭
```

### ⚙️ 灵活配置

多种配置方法：

**1. URI 字符串**：
```python
conn = idb.Connection.from_uri("postgresql://user:pass@host/db")
```

**2. TOML 文件**：
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

**3. 环境变量**：
```python
import os
uri = os.environ["DATABASE_URL"]
conn = idb.Connection.from_uri(uri)
```

## 高级特性

### 参数化查询

带参数绑定的安全 SQL：

```python
# 位置参数
df = conn.execute(
    "SELECT * FROM users WHERE age > ? AND city = ?",
    [18, "NYC"]
)

# 命名参数（如果数据库支持）
df = conn.execute(
    "SELECT * FROM users WHERE age > :age",
    {"age": 18}
)
```

### 批量操作

高效的批量插入：

```python
import polars as pl

# 大型 DataFrame
df = pl.DataFrame({
    "id": range(10000),
    "value": range(10000)
})

# 高效批量插入
conn.insert("data", df)  # 快速批量操作
```

### 错误处理

类型化异常，精确的错误处理：

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
    # 处理连接失败
    pass

try:
    conn.execute("SELECT * FROM nonexistent")
except QueryExecutionError as e:
    # 处理查询错误
    print(f"查询失败: {e}")

try:
    conn.insert("users", {"id": [1]})  # 重复
except ConstraintViolationError:
    # 处理约束违反
    pass
```

### 上下文管理器支持

自动资源清理：

```python
# 连接自动关闭
with idb.Connection.from_uri("...") as conn:
    df = conn.select("users")
    # 处理数据

# 保证连接关闭，即使发生异常
```

## 数据库特定功能

### PostgreSQL

- 完整 JSONB 支持
- 数组类型
- 自定义类型
- 窗口函数
- CTE（公共表表达式）

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

- 内存数据库
- 基于文件的数据库
- 无需服务器设置
- ACID 事务

```python
# 测试用内存数据库
conn = idb.Connection.from_uri("sqlite://:memory:")

# 持久化的基于文件数据库
conn = idb.Connection.from_uri("sqlite://./app.db")
```

### MSSQL

- Windows 身份验证
- SQL Server 特定类型
- TOP 子句支持
- 存储过程

```python
# 使用 TOP 而非 LIMIT
df = conn.execute("SELECT TOP 10 * FROM users")
```

## 即将推出

未来版本计划的功能：

### 第二阶段
- ⏳ 连接池
- ⏳ 事务支持
- ⏳ 异步 API
- ⏳ 预编译语句

### 第三阶段
- ⏳ MySQL 支持
- ⏳ Oracle 支持
- ⏳ 查询构建器
- ⏳ Schema 迁移工具
- ⏳ ORM 层（可选）

## 对比矩阵

| 特性 | IndustryDB | SQLAlchemy | pandas |
|------|-----------|------------|--------|
| 速度 | ⚡⚡⚡⚡⚡ | ⚡⚡⚡ | ⚡⚡ |
| 类型提示 | ✅ 完整 | ⚠️ 部分 | ❌ 有限 |
| DataFrame | ✅ Polars | ⚠️ Pandas | ✅ Pandas |
| 多数据库 | ✅ 3 个 | ✅ 10+ | ⚠️ 通过 SQL |
| ORM | ❌ 无 | ✅ 有 | ❌ 无 |
| 学习曲线 | 🟢 简单 | 🟡 中等 | 🟢 简单 |
| 内存 | ✅ 低 | ⚠️ 中等 | ❌ 高 |

## 了解更多

- [快速开始](/zh/guide/getting-started) - 安装和第一个查询
- [配置](/zh/guide/configuration) - 配置文件设置
- [CRUD 操作](/zh/guide/crud-operations) - 数据操作
- [示例](/zh/examples/quick-start) - 实用示例
