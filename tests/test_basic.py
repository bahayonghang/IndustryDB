"""Basic tests for IndustryDB."""

import polars as pl
import pytest

try:
    import industrydb as idb
except ImportError:
    pytest.skip("IndustryDB not built yet", allow_module_level=True)


def test_import():
    """Test that the module can be imported."""
    assert hasattr(idb, "__version__")
    assert hasattr(idb, "Connection")
    assert hasattr(idb, "DatabaseConfig")


def test_database_config_creation():
    """Test creating database configurations."""
    # SQLite config
    config = idb.DatabaseConfig(db_type="sqlite", path="./test.db")
    assert config.db_type == "sqlite"

    # Postgres config
    config = idb.DatabaseConfig(
        db_type="postgres",
        host="localhost",
        port=5432,
        database="test",
        username="user",
        password="pass",
    )
    assert config.db_type == "postgres"


def test_database_config_to_uri():
    """Test URI generation."""
    config = idb.DatabaseConfig(
        db_type="postgres",
        host="localhost",
        port=5432,
        database="mydb",
        username="user",
        password="secret",
    )
    uri = config.to_uri()
    assert uri.startswith("postgresql://")
    assert "localhost:5432" in uri


def test_database_config_from_uri():
    """Test creating config from URI."""
    uri = "postgresql://user:pass@localhost:5432/mydb"
    config = idb.DatabaseConfig.from_uri(uri)
    assert config.db_type == "postgres"


def test_sqlite_connection(tmp_path):
    """Test SQLite connection."""
    db_path = tmp_path / "test.db"

    config = idb.DatabaseConfig(db_type="sqlite", path=str(db_path))

    with idb.Connection(config) as conn:
        assert not conn.is_closed()

        # Create table
        conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")

        # Insert data
        df = pl.DataFrame({"id": [1, 2, 3], "name": ["Alice", "Bob", "Charlie"]})
        rows = conn.insert("users", df)
        assert rows == 3

        # Select data
        result = conn.select("users")
        assert result.height == 3
        assert "name" in result.columns

        # Select with WHERE
        result = conn.select("users", where_clause="id > 1")
        assert result.height == 2

    assert conn.is_closed()


def test_context_manager(tmp_path):
    """Test connection as context manager."""
    db_path = tmp_path / "test_cm.db"

    config = idb.DatabaseConfig(db_type="sqlite", path=str(db_path))

    conn = idb.Connection(config)
    assert not conn.is_closed()

    # Use as context manager
    with conn:
        conn.execute("CREATE TABLE test (id INTEGER)")

    # Should be closed after exiting context
    assert conn.is_closed()


def test_execute_query(tmp_path):
    """Test executing arbitrary SQL."""
    db_path = tmp_path / "test_exec.db"

    config = idb.DatabaseConfig(db_type="sqlite", path=str(db_path))

    with idb.Connection(config) as conn:
        # DDL
        conn.execute("CREATE TABLE products (id INTEGER, name TEXT, price REAL)")

        # INSERT
        conn.execute("INSERT INTO products VALUES (1, 'Widget', 9.99)")
        conn.execute("INSERT INTO products VALUES (2, 'Gadget', 19.99)")

        # SELECT
        df = conn.execute("SELECT * FROM products WHERE price > 10")
        assert df.height == 1
        assert df["name"][0] == "Gadget"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
