# 快速开始

本指南将帮助您安装 IndustryDB 并编写第一个数据库查询。

## 先决条件

安装 IndustryDB 之前，请确保您有：

- **Python 3.8 或更高版本**
- **pip** 或 **uv**（推荐）

如果从源码构建，还需要：
- **Rust 1.70+**（从 [rustup.rs](https://rustup.rs/) 安装）
- **系统依赖**（见[构建要求](#构建要求)）

## 安装

### 从 PyPI 安装（推荐）

IndustryDB 发布后，可通过 pip 安装：

```bash
pip install industrydb
```

或使用 [uv](https://github.com/astral-sh/uv)（更快）：

```bash
uv pip install industrydb
```

### 从源码安装

获取最新开发版本：

```bash
# 克隆仓库
git clone https://github.com/yourusername/industrydb.git
cd industrydb

# 创建虚拟环境
uv venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate

# 安装构建依赖
uv pip install maturin

# 构建并安装
uv run maturin develop
```

### 构建要求

::: details Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    libkrb5-dev \
    libssl-dev \
    pkg-config
```
:::

::: details macOS
```bash
brew install krb5 openssl
```
:::

::: details Windows
安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)，包含 C++ 开发工具。
:::

## 快速入门

### 第一个查询

创建文件 `quickstart.py`：

```python
import industrydb as idb

# 连接到 SQLite（最容易上手）
conn = idb.Connection.from_uri("sqlite://./test.db")

# 执行查询
df = conn.execute("""
    SELECT 'Hello' as message, 
           'IndustryDB' as name,
           42 as answer
""")

print(df)
conn.close()
```

运行：

```bash
python quickstart.py
```

预期输出：
```
shape: (1, 3)
┌─────────┬────────────┬────────┐
│ message ┆ name       ┆ answer │
│ ---     ┆ ---        ┆ ---    │
│ str     ┆ str        ┆ i64    │
╞═════════╪════════════╪════════╡
│ Hello   ┆ IndustryDB ┆ 42     │
└─────────┴────────────┴────────┘
```

### 使用上下文管理器

更好的实践，自动清理资源：

```python
import industrydb as idb

# 上下文管理器自动关闭连接
with idb.Connection.from_uri("sqlite://./test.db") as conn:
    df = conn.execute("SELECT * FROM users")
    print(df)
    
# 连接在此自动关闭
```

## 配置文件

生产环境中，将连接详情存储在 TOML 文件中。

### 创建配置

创建 `database.toml`：

```toml
[connections.dev_db]
type = "sqlite"
path = "./dev.db"

[connections.prod_db]
type = "postgres"
host = "localhost"
port = 5432
database = "myapp"
username = "dbuser"
password = "secret"

[connections.analytics]
type = "mssql"
host = "analytics.example.com"
port = 1433
database = "warehouse"
username = "analyst"
password = "secret"
```

### 加载配置

```python
import industrydb as idb

# 加载所有配置
configs = idb.load_config("database.toml")

# 连接到特定数据库
with configs["dev_db"].connect() as conn:
    df = conn.execute("SELECT * FROM users")
    print(df)

# 轻松切换到其他数据库
with configs["prod_db"].connect() as conn:
    df = conn.execute("SELECT * FROM orders")
    print(df)
```

## 基础 CRUD 操作

### 插入数据

```python
import industrydb as idb

conn = idb.Connection.from_uri("sqlite://./test.db")

# 首先创建表
conn.execute("""
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        age INTEGER
    )
""")

# 插入单条记录
conn.insert("users", {
    "name": ["Alice"],
    "age": [25]
})

# 插入多条记录
conn.insert("users", {
    "name": ["Bob", "Charlie"],
    "age": [30, 35]
})

conn.close()
```

### 查询数据

```python
# 查询所有
df = conn.select("users")

# 带条件查询
df = conn.select("users", where="age > ?", params=[25])

# 查询指定列
df = conn.select("users", columns=["name", "age"])

# 组合选项
df = conn.select(
    "users",
    columns=["name"],
    where="age >= ? AND age <= ?",
    params=[25, 35]
)
```

### 更新数据

```python
# 更新记录
rows_affected = conn.update(
    "users",
    {"age": 26},
    where="name = ?",
    params=["Alice"]
)
print(f"更新了 {rows_affected} 行")
```

### 删除数据

```python
# 删除记录
rows_affected = conn.delete(
    "users",
    where="age < ?",
    params=[18]
)
print(f"删除了 {rows_affected} 行")
```

## 使用 DataFrame

IndustryDB 与 Polars 无缝集成：

```python
import polars as pl
import industrydb as idb

# 创建 Polars DataFrame
df = pl.DataFrame({
    "id": [1, 2, 3],
    "name": ["Alice", "Bob", "Charlie"],
    "age": [25, 30, 35],
    "city": ["NYC", "LA", "SF"]
})

# 直接插入 DataFrame
conn = idb.Connection.from_uri("sqlite://./test.db")
conn.insert("users", df)

# 查询返回 Polars DataFrame
result = conn.select("users")

# 使用 Polars 操作
filtered = result.filter(pl.col("age") > 25)
sorted_df = filtered.sort("age", descending=True)
print(sorted_df)

conn.close()
```

## 错误处理

IndustryDB 提供类型化的异常：

```python
import industrydb as idb
from industrydb import (
    ConnectionError,
    QueryExecutionError,
    ConstraintViolationError
)

try:
    conn = idb.Connection.from_uri("postgresql://invalid")
except ConnectionError as e:
    print(f"连接失败: {e}")

try:
    conn.execute("INVALID SQL")
except QueryExecutionError as e:
    print(f"查询失败: {e}")

try:
    # 重复键插入
    conn.insert("users", {"id": [1], "name": ["Alice"]})
except ConstraintViolationError as e:
    print(f"违反约束: {e}")
```

## 数据库特定示例

### PostgreSQL

```python
import industrydb as idb

conn = idb.Connection.from_uri(
    "postgresql://user:password@localhost:5432/mydb"
)

# PostgreSQL 特定功能
df = conn.execute("""
    SELECT * FROM users
    WHERE created_at > NOW() - INTERVAL '7 days'
    ORDER BY created_at DESC
    LIMIT 100
""")

conn.close()
```

### SQLite

```python
import industrydb as idb

# 内存数据库
conn = idb.Connection.from_uri("sqlite://:memory:")

# 基于文件的数据库
conn = idb.Connection.from_uri("sqlite://./myapp.db")

# 相对路径
conn = idb.Connection.from_uri("sqlite://./data/app.db")

conn.close()
```

### MSSQL

```python
import industrydb as idb

conn = idb.Connection.from_uri(
    "mssql://user:password@server:1433/database"
)

# MSSQL 使用 TOP 而非 LIMIT
df = conn.execute("SELECT TOP 10 * FROM orders")

conn.close()
```

## 下一步

完成基础知识后：

- [配置指南](/zh/guide/configuration) - 高级配置选项
- [CRUD 操作](/zh/guide/crud-operations) - 详细的 CRUD 文档
- [DataFrame 集成](/zh/guide/dataframe) - 使用 Polars
- [错误处理](/zh/guide/error-handling) - 全面的错误处理
- [示例](/zh/examples/quick-start) - 更多实用示例

## 故障排除

### 导入错误

如果看到 `ModuleNotFoundError: No module named 'industrydb'`：

```bash
# 验证安装
pip list | grep industrydb

# 必要时重新安装
pip install --force-reinstall industrydb
```

### 连接错误

连接失败时：

1. **检查数据库运行**：验证数据库服务器可访问
2. **验证凭据**：确保用户名/密码正确
3. **检查网络**：确保防火墙允许连接
4. **使用原生客户端测试**：尝试使用 psql、sqlite3 或 sqlcmd 连接

### 构建错误

从源码构建失败时：

1. **更新 Rust**：`rustup update`
2. **安装系统依赖**：见[构建要求](#构建要求)
3. **查看问题跟踪**：[GitHub Issues](https://github.com/yourusername/industrydb/issues)
