# GitHub Actions 工作流说明

本目录包含 IndustryDB 项目的 CI/CD 工作流配置。

## 📋 工作流概览

### 1. CI 工作流 (`ci.yml`)

**触发条件：** 
- 推送到 `main`/`master`/`develop` 分支
- 针对上述分支的 Pull Request

**包含的检查：**

#### Python 代码质量
- **Ruff 格式检查：** 确保 Python 代码符合格式规范
- **Ruff Lint：** 检查代码质量问题（包括 pycodestyle、pyflakes、isort 等）
- **MyPy 类型检查：** 静态类型检查

#### Rust 代码质量
- **Cargo fmt：** Rust 代码格式检查
- **Cargo clippy：** Rust linter，检查常见错误和不良实践
- **Cargo test：** 运行所有 Rust 单元测试

#### Python 测试
- 在多个平台（Ubuntu、Windows、macOS）上测试
- 测试 Python 3.8-3.12 的兼容性

### 2. Release 工作流 (`release.yml`)

**触发条件：**
- 创建 `v*.*.*` 格式的 Git tag（例如 `v0.1.0`）
- 手动触发（workflow_dispatch）

**构建平台：**

| 平台 | 架构 | 说明 |
|------|------|------|
| **Linux** | x86_64 | 64位 Intel/AMD |
| **Linux** | aarch64 | 64位 ARM（树莓派等） |
| **Windows** | x64 | 64位 Windows |
| **macOS** | x86_64 | Intel Mac |
| **macOS** | aarch64 | Apple Silicon (M1/M2/M3) |

**输出：**
- 各平台的 Python wheels（`.whl` 文件）
- Source distribution（`.tar.gz` 文件）
- GitHub Release（自动创建，包含所有构建产物）

### 3. Dependabot 配置 (`dependabot.yml`)

自动检查并创建 PR 以更新：
- GitHub Actions 版本
- Cargo 依赖
- Python 依赖

## 🚀 发布新版本

### 步骤：

1. **更新版本号**
   ```bash
   # 更新 Cargo.toml 中的版本
   # 更新 pyproject.toml 中的版本
   ```

2. **提交更改**
   ```bash
   git add Cargo.toml pyproject.toml
   git commit -m "chore: bump version to 0.1.0"
   git push
   ```

3. **创建并推送 tag**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. **等待构建完成**
   - 访问 [Actions 页面](../../actions) 查看构建进度
   - 构建完成后，wheels 会自动上传到 [Releases](../../releases)

5. **（可选）发布到 PyPI**
   - 取消 `release.yml` 中的 `publish-pypi` job 注释
   - 在 GitHub 仓库设置中添加 `PYPI_API_TOKEN` secret
   - 或使用 Trusted Publishing（推荐）

## 🔧 本地测试

### Python 代码检查

```bash
# 安装开发依赖
uv pip install ruff mypy types-toml

# 格式检查
ruff format --check .

# Lint 检查
ruff check .

# 类型检查
mypy python/industrydb
```

### Rust 代码检查

```bash
# 格式检查
cargo fmt --all -- --check

# Clippy 检查
cargo clippy --workspace --all-targets --all-features

# 运行测试
cargo test --workspace
```

### 本地构建 Wheel

```bash
# 安装 maturin
pip install maturin

# 构建开发版本
maturin develop

# 构建发布版本
maturin build --release
```

## 📝 最佳实践

### CI 失败处理
1. 查看失败的 job 日志
2. 在本地运行相同的检查命令
3. 修复问题后重新提交

### 发布检查清单
- [ ] 所有 CI 检查通过
- [ ] 更新 CHANGELOG.md
- [ ] 更新版本号
- [ ] 测试重要功能
- [ ] 创建 tag

### 优化构建时间
- Rust 编译使用缓存（`Swatinem/rust-cache`）
- Maturin 使用 sccache 加速
- Python 依赖使用 uv 缓存

## 🔍 故障排除

### 常见问题

**Q: Linux aarch64 构建失败**  
A: 检查是否有 C/C++ 依赖需要交叉编译工具链。参考 [manylinux-cross](https://github.com/messense/manylinux-cross)。

**Q: macOS ARM 构建失败**  
A: 确保 Rust 工具链支持 `aarch64-apple-darwin` target。

**Q: Windows 构建慢**  
A: Windows 编译较慢是正常的，sccache 会帮助加速后续构建。

**Q: PyPI 发布失败**  
A: 检查 PyPI token 配置，或使用 Trusted Publishing（无需 token）。

## 📚 相关资源

- [Maturin 文档](https://www.maturin.rs/)
- [PyO3 指南](https://pyo3.rs/)
- [GitHub Actions 文档](https://docs.github.com/actions)
- [Ruff 文档](https://docs.astral.sh/ruff/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
