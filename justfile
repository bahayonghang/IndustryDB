# IndustryDB Justfile
# 使用 `just <command>` 运行命令
# 安装 just: cargo install just 或 https://github.com/casey/just

# 默认命令 - 显示所有可用命令
default:
    @just --list

# === 环境设置 ===

# 创建虚拟环境并同步依赖
setup:
    @echo "📦 设置开发环境..."
    uv venv
    @echo "✅ 虚拟环境已创建"
    @echo "💡 请运行: source .venv/bin/activate"

# 同步 Python 依赖
sync:
    @echo "🔄 同步依赖..."
    uv sync --all-extras

# 安装开发依赖
install-dev:
    @echo "📦 安装开发依赖..."
    uv pip install maturin pytest mypy ruff

# === 构建命令 ===

# 检查 Rust 代码（快速）
check:
    @echo "🔍 检查 Rust 代码..."
    cargo check --workspace

# 构建所有 Rust crate（debug 模式）
build:
    @echo "🔨 构建项目（debug）..."
    cargo build --workspace

# 构建 release 版本
build-release:
    @echo "🔨 构建项目（release）..."
    cargo build --workspace --release

# 开发模式：编译并安装 Python 包
develop:
    @echo "🚀 开发模式构建..."
    uv run maturin develop

# 开发模式 + release 优化
develop-release:
    @echo "🚀 开发模式构建（release）..."
    uv run maturin develop --release

# 构建 Python wheel
wheel:
    @echo "🎡 构建 wheel..."
    maturin build --release

# 构建所有平台的 wheel
wheel-all:
    @echo "🎡 构建所有平台 wheel..."
    maturin build --release --interpreter python3.8 python3.9 python3.10 python3.11 python3.12

# === 测试命令 ===

# 运行所有测试
test: test-rust test-python

# 运行 Rust 测试
test-rust:
    @echo "🧪 运行 Rust 测试..."
    cargo test --workspace

# 运行 Python 测试
test-python:
    @echo "🧪 运行 Python 测试..."
    @if command -v pytest >/dev/null 2>&1; then \
        pytest tests/ -v; \
    else \
        echo "⚠️  pytest 未安装，跳过 Python 测试"; \
        echo "💡 运行 'just install-dev' 安装开发工具"; \
    fi

# 运行特定 Python 测试
test-file FILE:
    @echo "🧪 运行测试文件: {{FILE}}"
    @if command -v pytest >/dev/null 2>&1; then \
        pytest {{FILE}} -v; \
    else \
        echo "⚠️  pytest 未安装"; \
        echo "💡 运行 'just install-dev' 安装开发工具"; \
    fi

# 运行测试并显示覆盖率
test-coverage:
    @echo "📊 运行测试并生成覆盖率报告..."
    @if command -v pytest >/dev/null 2>&1; then \
        pytest tests/ --cov=industrydb --cov-report=html --cov-report=term; \
    else \
        echo "⚠️  pytest 未安装"; \
        echo "💡 运行 'just install-dev' 安装开发工具"; \
    fi

# === 代码质量 ===

# 运行所有检查
lint: lint-rust lint-python

# Rust 代码检查
lint-rust:
    @echo "🔍 Rust 代码检查..."
    cargo clippy --workspace -- -D warnings

# Python 代码检查
lint-python:
    @echo "🔍 Python 代码检查..."
    @if command -v ruff >/dev/null 2>&1; then \
        ruff check python/; \
    else \
        echo "⚠️  ruff 未安装，跳过 Python 检查"; \
        echo "💡 运行 'just install-dev' 安装开发工具"; \
    fi

# 格式化所有代码
fmt: fmt-rust fmt-python

# 格式化 Rust 代码
fmt-rust:
    @echo "✨ 格式化 Rust 代码..."
    cargo fmt --all

# 格式化 Python 代码
fmt-python:
    @echo "✨ 格式化 Python 代码..."
    @if command -v ruff >/dev/null 2>&1; then \
        ruff format python/; \
    else \
        echo "⚠️  ruff 未安装，跳过 Python 格式化"; \
        echo "💡 运行 'just install-dev' 安装开发工具"; \
    fi

# 类型检查
type-check:
    @echo "🔬 类型检查..."
    @if command -v mypy >/dev/null 2>&1; then \
        mypy python/industrydb; \
    else \
        echo "⚠️  mypy 未安装，跳过类型检查"; \
        echo "💡 运行 'just install-dev' 安装开发工具"; \
    fi

# === 清理命令 ===

# 清理所有构建产物
clean:
    @echo "🧹 清理构建产物..."
    cargo clean
    rm -rf target/
    rm -rf python/industrydb/*.so
    find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
    find . -type d -name "*.egg-info" -exec rm -rf {} + 2>/dev/null || true
    find . -type f -name "*.pyc" -delete
    @echo "✅ 清理完成"

# 清理 Python 缓存
clean-python:
    @echo "🧹 清理 Python 缓存..."
    find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
    find . -type d -name "*.egg-info" -exec rm -rf {} + 2>/dev/null || true
    find . -type f -name "*.pyc" -delete

# === 开发工作流 ===

# 快速开发循环：格式化 + 检查 + 构建
dev: fmt check develop
    @echo "✅ 开发构建完成"

# 完整检查：格式化 + 检查 + 测试
ci: version-sync
    @echo "🔍 运行 Ruff 格式检查..."
    uvx ruff format --check --diff .
    @echo "🔍 运行 Ruff Lint..."
    uvx ruff check .
    @echo "🦀 检查 Rust 格式..."
    cargo fmt --all -- --check
    @echo "🦀 运行 Clippy（与 CI 一致）..."
    RUSTFLAGS="-D warnings" cargo clippy --workspace --all-targets --all-features -- -D warnings
    @echo "🧪 运行 Rust 测试（全特性）..."
    RUSTFLAGS="-D warnings" cargo test --workspace --all-features
    @echo "📦 构建 Python wheel（与 CI 一致）..."
    uv run maturin build --release --out dist
    @echo "🧪 准备 Python 虚拟环境（优先复用 .venv）..."
    @if [ -d ".venv" ]; then \
        echo "♻️ 复用现有 .venv"; \
    else \
        echo "✨ 创建新的 .venv"; \
        uv venv; \
    fi
    uv pip install --python .venv/bin/python dist/*.whl pytest pytest-asyncio
    @echo "🧪 运行 Python 测试（wheel 环境）..."
    uv run --python .venv/bin/python pytest tests/ -v
    @echo "🔬 运行 mypy 类型检查..."
    uv pip install --python .venv/bin/python mypy types-toml
    uv run --python .venv/bin/python mypy python/industrydb --ignore-missing-imports
    @echo "✅ CI 检查完成"

# 重新构建（清理 + 构建 + 开发模式）
rebuild: clean build develop
    @echo "✅ 重新构建完成"

# === 文档命令 ===

# 生成 Rust 文档
doc:
    @echo "📚 生成 Rust 文档..."
    cargo doc --workspace --no-deps --open

# 生成 Rust 文档（包含私有项）
doc-all:
    @echo "📚 生成完整 Rust 文档..."
    cargo doc --workspace --no-deps --document-private-items --open

# === 示例和演示 ===

# 运行快速开始示例
example:
    @echo "🎯 运行快速开始示例..."
    python examples/quickstart.py

# === 数据库相关 ===

# 创建示例数据库
create-example-db:
    @echo "📊 创建示例数据库..."
    python -c "import sqlite3; conn = sqlite3.connect('example.db'); conn.execute('CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)'); conn.commit(); conn.close()"
    @echo "✅ 示例数据库已创建: example.db"

# === 系统依赖检查 ===

# 检查系统依赖
check-deps:
    @echo "🔍 检查系统依赖..."
    @echo "Rust:"
    @rustc --version || echo "❌ Rust 未安装"
    @echo ""
    @echo "Python:"
    @python --version || echo "❌ Python 未安装"
    @echo ""
    @echo "uv:"
    @uv --version || echo "❌ uv 未安装"
    @echo ""
    @echo "maturin:"
    @maturin --version || echo "❌ maturin 未安装"
    @echo ""
    @echo "系统库:"
    @pkg-config --exists krb5 && echo "✅ libkrb5 已安装" || echo "❌ libkrb5 未安装 (sudo apt-get install libkrb5-dev)"
    @pkg-config --exists openssl && echo "✅ openssl 已安装" || echo "❌ openssl 未安装 (sudo apt-get install libssl-dev)"

# 安装系统依赖提示
install-sys-deps:
    @echo "📦 安装系统依赖..."
    @echo ""
    @echo "Ubuntu/Debian:"
    @echo "  sudo apt-get update"
    @echo "  sudo apt-get install -y libkrb5-dev libssl-dev build-essential"
    @echo ""
    @echo "Fedora/RHEL:"
    @echo "  sudo dnf install -y krb5-devel openssl-devel gcc"
    @echo ""
    @echo "macOS:"
    @echo "  brew install krb5 openssl"
    @echo "  export PKG_CONFIG_PATH=\"/usr/local/opt/krb5/lib/pkgconfig:\$PKG_CONFIG_PATH\""

# === 性能和基准测试 ===

# 运行基准测试
bench:
    @echo "⚡ 运行基准测试..."
    cargo bench --workspace

# === 发布命令 ===

# 发布前检查
pre-release: ci doc
    @echo "🔍 发布前检查..."
    @echo "✅ 所有检查通过"

# 版本更新（需要手动指定版本）
version VERSION:
    @echo "📝 更新版本到 {{VERSION}}..."
    @echo "请手动更新以下文件中的版本号:"
    @echo "  - Cargo.toml (workspace.package.version)"
    @echo "  - pyproject.toml (project.version)"

# 同步版本号：将仓库中所有版本号与 Cargo.toml 对齐
version-sync:
    @echo "🔄 同步版本号到 Cargo.toml 中定义的 workspace.package.version..."
    @python3 -c "from pathlib import Path; import re, sys; root = Path.cwd(); cargo_text = (root / 'Cargo.toml').read_text(encoding='utf-8'); pkg = re.search(r'(?ms)\[workspace\.package\](.*?)(?:\n\[|\Z)', cargo_text) or sys.exit('未找到 [workspace.package] 段落，无法同步版本'); version = re.search(r'(?m)^version\s*=\s*\"([^\"]+)\"', pkg.group(1)) or sys.exit('未找到 workspace.package.version 字段'); cargo_version = version.group(1); pyproject_path = root / 'pyproject.toml'; py_text = pyproject_path.read_text(encoding='utf-8'); project = re.search(r'(?ms)\[project\](.*?)(?:\n\[|\Z)', py_text) or sys.exit('未找到 [project] 段落，无法同步版本'); body = project.group(1); new_body, replaced = re.subn(r'(?m)^version\s*=\s*\"[^\"]+\"', f'version = \"{cargo_version}\"', body, count=1); replaced or sys.exit('[project] 内缺少 version 字段'); pyproject_path.write_text(py_text[: project.start(1)] + new_body + py_text[project.end(1):], encoding='utf-8'); print(f'版本同步完成：{cargo_version}')"

# === 监视模式 ===

# 监视 Rust 代码变化并自动重新编译
watch:
    @echo "👀 监视模式（需要安装 cargo-watch）..."
    @echo "安装: cargo install cargo-watch"
    cargo watch -x 'check --workspace' -x 'test --workspace'

# 监视并自动运行开发构建
watch-dev:
    @echo "👀 监视并自动构建开发版本..."
    cargo watch -s 'just develop'

# === 项目信息 ===

# 显示项目信息
info:
    @echo "📊 IndustryDB 项目信息"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Workspace Crates:"
    @cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]' | sed 's/.*#/  - /'
    @echo ""
    @echo "Lines of Code:"
    @find crates -name "*.rs" | xargs wc -l | tail -1 | awk '{print "  Rust: " $$1 " lines"}'
    @find python -name "*.py" | xargs wc -l | tail -1 | awk '{print "  Python: " $$1 " lines"}'
    @echo ""
    @echo "Dependencies:"
    @echo "  Rust crates: $(cargo tree --workspace --depth 0 | wc -l)"

# === 帮助 ===

# 显示帮助信息
help:
    @echo "🚀 IndustryDB 开发工具"
    @echo ""
    @echo "快速开始:"
    @echo "  just setup          # 设置开发环境"
    @echo "  just sync           # 同步依赖"
    @echo "  just develop        # 构建开发版本"
    @echo "  just test           # 运行所有测试"
    @echo ""
    @echo "常用命令:"
    @echo "  just dev            # 快速开发循环（格式化+检查+构建）"
    @echo "  just ci             # CI 检查（格式化+检查+测试）"
    @echo "  just clean          # 清理所有构建产物"
    @echo ""
    @echo "更多命令请运行: just --list"
