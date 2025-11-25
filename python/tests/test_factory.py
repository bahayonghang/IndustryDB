import importlib
import pytest

spec = importlib.util.find_spec("industrydb")
if spec is None:
    pytest.skip("industrydb extension not built", allow_module_level=True)

try:
    from industrydb import (  # type: ignore
        ConfigurationError,
        DatabaseConfig,
        create_timeseries_client,
        create_web_client,
    )
except ImportError:
    pytest.skip("industrydb extension not importable in this environment", allow_module_level=True)


def test_factory_rejects_non_mssql():
    cfg = DatabaseConfig(type="postgres", host="localhost", port=5432, database="db")
    with pytest.raises(ConfigurationError):
        create_timeseries_client(cfg)


def test_factory_missing_target_defaults_to_error():
    # timeseries/web creation without mssql config should raise configuration error
    with pytest.raises(ConfigurationError):
        create_web_client(DatabaseConfig(type="sqlite", path=":memory:"))
