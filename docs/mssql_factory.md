# MSSQL Factory Helpers

This library now mirrors the `ref/industrytslib` MSSQL factory interface with Rust-backed clients.

## Targets

Supported `target` values for `type = "mssql"` configs:

- base (raw connector)
- web
- timeseries
- quality
- online_trainer
- realtime_predict
- realtime_predict_alter
- realtime_predict_sequence
- decision_history
- decision_making
- operation
- model_status

## Python usage

```python
from industrydb import (
    DatabaseConfig,
    create_web_client,
    create_timeseries_client,
    create_quality_client,
    create_realtime_predict_client,
    create_realtime_predict_alter_client,
    create_realtime_predict_sequence_client,
    create_decision_history_client,
    create_decision_making_client,
    create_operation_client,
    create_model_status_client,
)

cfg = DatabaseConfig(
    db_type="mssql",
    host="localhost",
    port=1433,
    database="production",
    username="sa",
    password="YourStrong!Passw0rd",
)

web = create_web_client(cfg)
sample, model = web.get_prediction_info("my_project")

ts = create_timeseries_client(cfg)
latest = ts.get_latest_input_data("TagDatabase")
```

## Notes

- Backed by Rust connector; no `pyodbc` dependency.
- Errors surface as `IndustryDbError` family.
- Table/column semantics follow the reference interfaces (see `openspec/changes/add-mssql-factory-clients/specs/mssql-factory/spec.md`).
