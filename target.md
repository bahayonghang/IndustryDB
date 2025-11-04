请基于polars库作为中间件使用Rust连接不同的数据库，目前需要支持的有：
- MSSQL
- Postgres
- SQLite

你需要先实现简单的增删改查和执行sql语句的功能，后续可以扩展为支持更多的数据库类型和功能。因此你需要仔细设计项目架构，保证扩展性。

这个库的目标是供python调用，使用uv包管理器，uv add maturin --dev， uv run maturin develop进行开发。

请搜索网络中的pyo3最佳实践，整理一个合理的项目架构，然后可以通过一个toml配置文件配置数据库连接，然后通过python的rtoml库读取配置文件，基于当前项目连接数据库并进行处理

此外，你需要学习polars 1.35版本以后的处理方法，将运行时和python库分离开来，确保在任何平台都能够能正常运行。此外，学习polars的项目结构，重新构建当前项目的文件夹结构，并确保在python库中有相应的pyi文件以满足typing需要