---
layout: home

hero:
  name: "IndustryDB"
  text: "高性能数据库中间件"
  tagline: 基于 Rust 和 Polars 构建，实现极速数据操作
  image:
    src: /logo.svg
    alt: IndustryDB
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/guide/getting-started
    - theme: alt
      text: GitHub 仓库
      link: https://github.com/yourusername/industrydb

features:
  - icon: ⚡
    title: 极速性能
    details: 采用 Rust 编写，提供极致性能。通过 Apache Arrow 格式实现零拷贝数据传输，最大限度降低开销。
    
  - icon: 🔌
    title: 多数据库支持
    details: 无缝支持 PostgreSQL、SQLite 和 MSSQL，提供统一的接口。
    
  - icon: 📊
    title: Polars 集成
    details: 原生 DataFrame 支持，与 Polars 深度集成，实现高效数据处理。
    
  - icon: 🛡️
    title: 类型安全
    details: 全面的类型提示和类型桩文件，提供出色的 IDE 支持和类型检查。
    
  - icon: 🔧
    title: 简单易用
    details: 简洁的 Pythonic API，自然易懂。支持上下文管理器进行资源管理。
    
  - icon: 🚀
    title: 生产就绪
    details: 模块化架构，全面的错误处理和广泛的测试覆盖。
---

## 快速示例

::: code-group
```python [快速开始]
import industrydb as idb

# 使用 URI 连接
conn = idb.Connection.from_uri(
    "postgresql://user:pass@localhost/mydb"
)

# 执行查询并获取 Polars DataFrame
df = conn.execute("SELECT * FROM users WHERE age > ?", [18])
print(df)

conn.close()
```

```python [配置文件]
import industrydb as idb

# 从 TOML 配置加载
configs = idb.load_config("database.toml")

# 使用上下文管理器
with configs["my_postgres"].connect() as conn:
    # CRUD 操作
    conn.insert("users", {"name": ["Alice"], "age": [25]})
    df = conn.select("users", where="name = ?", params=["Alice"])
    print(df)
```

```python [DataFrame 操作]
import polars as pl
import industrydb as idb

# 创建 DataFrame
df = pl.DataFrame({
    "name": ["Alice", "Bob", "Charlie"],
    "age": [25, 30, 35]
})

# 直接插入 DataFrame
conn = idb.Connection.from_uri("sqlite://./test.db")
conn.insert("users", df)

# 以 DataFrame 形式查询
result = conn.select("users", where="age >= ?", params=[30])
print(result)
```
:::

## 为什么选择 IndustryDB？

IndustryDB 在高性能 Rust 代码和 Python 易用性之间架起了桥梁。无论您是构建数据管道、分析工具，还是需要在 Python 中高效访问数据库，IndustryDB 都能提供：

- **性能**：Rust 编译代码，零拷贝数据传输
- **简洁**：自然直观的 Pythonic API
- **灵活**：统一接口支持多种数据库
- **安全**：类型安全操作，全面的错误处理

## 安装

```bash
pip install industrydb
```

或从源码安装：

```bash
git clone https://github.com/yourusername/industrydb.git
cd industrydb
uv pip install maturin
uv run maturin develop
```

## 社区

- **GitHub**: [yourusername/industrydb](https://github.com/yourusername/industrydb)
- **问题反馈**: [报告 Bug 或请求新功能](https://github.com/yourusername/industrydb/issues)
- **许可证**: [MIT 许可证](https://github.com/yourusername/industrydb/blob/main/LICENSE)

---

<div style="text-align: center; margin-top: 2rem; padding: 1rem;">
  <p style="color: #666;">使用 Rust、Polars、ConnectorX 和 PyO3 精心打造</p>
</div>
