# Build Instructions for IndustryDB

## Prerequisites

### System Dependencies

IndustryDB requires the following system dependencies to build:

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libkrb5-dev \
    libgssapi-krb5-2
```

#### Fedora/RHEL/CentOS
```bash
sudo dnf install -y \
    gcc \
    pkg-config \
    openssl-devel \
    krb5-devel
```

#### macOS
```bash
brew install krb5 openssl
```

#### Windows
- Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
- Install [Rust](https://rustup.rs/)
- GSSAPI is typically available through Windows' native implementation

### Rust Toolchain
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Should be 1.70+
```

### Python Environment
```bash
# Python 3.8 or later required
python --version

# Install uv (recommended)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Or use pip
pip install uv
```

## Quick Build

### Development Build

```bash
# 1. Clone and enter directory
cd /mnt/data/lyh/industrydb

# 2. Create virtual environment
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# 3. Install maturin
uv pip install maturin

# 4. Build and install in development mode
uv run maturin develop

# 5. Test the installation
python -c "import industrydb; print(industrydb.__version__)"
```

### Release Build

```bash
# Build optimized wheel
maturin build --release

# Install the wheel
pip install target/wheels/industrydb-*.whl
```

## Build Options

### For different Python versions
```bash
# Build for specific Python
maturin build --release --interpreter python3.11

# Build for all installed Python versions
maturin build --release --interpreter python3.8 python3.9 python3.10 python3.11 python3.12
```

### With features
```bash
# Build without optional features
maturin build --release --no-default-features

# Build with specific features
maturin build --release --features "some-feature"
```

### Cross-compilation
```bash
# For Linux (manylinux)
maturin build --release --target x86_64-unknown-linux-gnu

# For macOS
maturin build --release --target x86_64-apple-darwin
maturin build --release --target aarch64-apple-darwin

# For Windows
maturin build --release --target x86_64-pc-windows-msvc
```

## Testing

### Run Rust tests
```bash
cargo test --workspace
```

### Run Python tests
```bash
# Install test dependencies
uv pip install pytest pytest-asyncio

# Run tests
pytest tests/ -v
```

### Type checking
```bash
uv pip install mypy
mypy python/industrydb
```

### Linting
```bash
# Python
uv pip install ruff
ruff check python/

# Rust
cargo clippy --workspace -- -D warnings
```

## Troubleshooting

### GSSAPI Build Errors

**Error**: `fatal error: 'gssapi.h' file not found`

**Solution**: Install Kerberos development files:
```bash
# Ubuntu/Debian
sudo apt-get install libkrb5-dev

# Fedora/RHEL
sudo dnf install krb5-devel

# macOS
brew install krb5
export PKG_CONFIG_PATH="/usr/local/opt/krb5/lib/pkgconfig:$PKG_CONFIG_PATH"
```

### OpenSSL Build Errors

**Error**: `Could not find directory of OpenSSL installation`

**Solution**:
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)

# Or use vendored OpenSSL
export OPENSSL_STATIC=1
```

### Polars Version Conflicts

**Error**: `pyo3-ffi` version conflicts

**Solution**: This is fixed in the current configuration. Ensure you're using:
- pyo3 = "0.21"
- pyo3-polars = "0.15"
- polars = "0.44"

### Maturin Not Found

**Error**: `maturin: command not found`

**Solution**:
```bash
# Ensure maturin is installed
uv pip install maturin

# Or install globally
pip install maturin

# Verify installation
maturin --version
```

### Link Errors on Windows

**Error**: LNK errors during build

**Solution**:
1. Install Visual Studio Build Tools with C++ support
2. Run build from "x64 Native Tools Command Prompt"

## Development Workflow

### Edit-Compile-Test Loop

```bash
# 1. Make changes to Rust code
vim crates/industrydb-core/src/connection.rs

# 2. Rebuild (fast incremental build)
maturin develop

# 3. Test changes
python examples/quickstart.py

# Or run tests
pytest tests/test_basic.py -v
```

### Watch Mode (automatic rebuild)

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-rebuild on changes
cargo watch -x 'build --package industrydb-core'
```

## Performance Optimization

### Build with maximum optimization
```bash
maturin build --release
```

This enables:
- LTO (Link-Time Optimization)
- Optimization level 3
- Single codegen unit
- Symbol stripping

### Profile-Guided Optimization (PGO)

```bash
# Step 1: Build with instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" maturin build

# Step 2: Run representative workload
python benchmarks/run_workload.py

# Step 3: Build with profile data
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data" maturin build --release
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libkrb5-dev libssl-dev
      
      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
      
      - name: Build wheel
        run: |
          pip install maturin
          maturin build --release
      
      - name: Test
        run: |
          pip install target/wheels/*.whl
          pip install pytest
          pytest tests/
```

## Docker Build

```dockerfile
FROM rust:1.75 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libkrb5-dev \
    libssl-dev \
    python3-dev \
    python3-pip

# Install maturin
RUN pip3 install maturin

# Copy source
WORKDIR /app
COPY . .

# Build wheel
RUN maturin build --release

# Runtime image
FROM python:3.11-slim
COPY --from=builder /app/target/wheels/*.whl /tmp/
RUN pip install /tmp/*.whl && rm /tmp/*.whl
```

## Clean Build

```bash
# Clean Rust artifacts
cargo clean

# Clean Python artifacts
rm -rf target/ .venv/ python/industrydb/__pycache__
find . -type d -name "*.egg-info" -exec rm -rf {} +
find . -type f -name "*.pyc" -delete
find . -type f -name "*.so" -delete

# Start fresh
uv venv
source .venv/bin/activate
uv run maturin develop
```

## Environment Variables

Useful environment variables for building:

```bash
# Rust compilation
export RUSTFLAGS="-C target-cpu=native"  # Optimize for local CPU

# OpenSSL
export OPENSSL_DIR=/usr/local/opt/openssl
export OPENSSL_STATIC=1  # Use static linking

# Kerberos (macOS)
export PKG_CONFIG_PATH="/usr/local/opt/krb5/lib/pkgconfig:$PKG_CONFIG_PATH"

# Python
export PYTHON_SYS_EXECUTABLE=/usr/bin/python3.11

# Maturin
export MATURIN_PEP517_ARGS="--release"
```

## Platform-Specific Notes

### Linux (manylinux)
- Wheels are manylinux-compatible by default
- Uses musllinux for Alpine-based systems

### macOS (Universal Binary)
```bash
# Build universal wheel (Intel + Apple Silicon)
maturin build --release --universal2
```

### Windows
- Requires Visual Studio Build Tools 2019+
- Use PowerShell or cmd, not WSL
- May need to set MSVC paths manually

## Verification

After building, verify the installation:

```python
import industrydb as idb
import polars as pl

# Check version
print(f"Version: {idb.__version__}")

# Test basic functionality
config = idb.DatabaseConfig(db_type="sqlite", path=":memory:")
conn = idb.Connection(config)
print("✓ Connection created")

df = conn.execute("SELECT 1 as test")
print(f"✓ Query executed: {df}")

conn.close()
print("✓ All checks passed!")
```

## Need Help?

- Check [GitHub Issues](https://github.com/yourusername/industrydb/issues)
- Review [CONTRIBUTING.md](CONTRIBUTING.md)
- Ask in Discussions
