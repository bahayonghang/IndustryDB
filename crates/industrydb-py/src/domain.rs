//! Python bindings for MSSQL domain clients.

use std::sync::Arc;

use industrydb_core::{
    config::{ConnectionConfig, DatabaseType},
    error::IndustryDbError,
};
use industrydb_mssql::domain::{
    DecisionHistoryClient, DecisionMakingClient, DomainFactory, ModelStatusClient, OperationClient,
    QualityClient, RealtimePredictAlterClient, RealtimePredictClient,
    RealtimePredictSequenceClient, TimeSeriesClient, WebClient,
};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use tokio::runtime::Runtime;

use crate::config::PyDatabaseConfig;
use crate::errors::to_py_err;

fn ensure_mssql(config: &ConnectionConfig) -> Result<(), IndustryDbError> {
    if config.db_type != DatabaseType::Mssql {
        Err(IndustryDbError::config_error("Config is not MSSQL"))
    } else {
        Ok(())
    }
}

fn dataframe_to_py_dict(py: Python, df: &polars::prelude::DataFrame) -> PyResult<Py<PyDict>> {
    use polars::prelude::*;

    let dict = PyDict::new_bound(py);

    for col in df.get_columns() {
        let col_name = col.name().as_str();
        let values = PyList::empty_bound(py);

        for i in 0..col.len() {
            if col.is_null().get(i).unwrap_or(false) {
                values.append(py.None())?;
            } else {
                match col.dtype() {
                    DataType::Int32 => values.append(col.i32().unwrap().get(i))?,
                    DataType::Int64 => values.append(col.i64().unwrap().get(i))?,
                    DataType::Float64 => values.append(col.f64().unwrap().get(i))?,
                    DataType::String => {
                        let val = col.str().unwrap().get(i).unwrap_or("");
                        values.append(val)?;
                    }
                    DataType::Boolean => values.append(col.bool().unwrap().get(i))?,
                    _ => {
                        let val = col.get(i).unwrap().to_string();
                        values.append(val)?;
                    }
                }
            }
        }

        dict.set_item(col_name, values)?;
    }

    Ok(dict.unbind())
}

fn py_dict_to_dataframe(data: &Bound<'_, PyDict>) -> PyResult<polars::prelude::DataFrame> {
    use polars::prelude::*;

    let mut columns: Vec<Column> = Vec::new();

    for (key, value) in data.iter() {
        let col_name: String = key.extract()?;
        let list: &Bound<'_, PyList> = value.downcast()?;

        let mut values_str: Vec<Option<String>> = Vec::new();
        for item in list.iter() {
            if item.is_none() {
                values_str.push(None);
            } else if let Ok(val) = item.extract::<String>() {
                values_str.push(Some(val));
            } else if let Ok(val) = item.extract::<i64>() {
                values_str.push(Some(val.to_string()));
            } else if let Ok(val) = item.extract::<f64>() {
                values_str.push(Some(val.to_string()));
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "Unsupported type in column {}",
                    col_name
                )));
            }
        }

        let series = Series::new(col_name.as_str().into(), values_str);
        columns.push(series.into_column());
    }

    DataFrame::new(columns)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
}

macro_rules! build_client {
    ($ctor:ident, $inner:ty) => {
        #[pyclass]
        pub struct $ctor {
            inner: $inner,
            runtime: Arc<Runtime>,
        }

        impl $ctor {
            fn new_with_config(config: &PyDatabaseConfig) -> PyResult<Self> {
                let runtime = Arc::new(Runtime::new().map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?);
                let cfg = config.inner();
                ensure_mssql(cfg).map_err(to_py_err)?;

                let client = runtime
                    .block_on(async {
                        let connector = DomainFactory::create_connector(cfg).await?;
                        Ok(<$inner>::new(connector))
                    })
                    .map_err(to_py_err)?;

                Ok(Self {
                    inner: client,
                    runtime,
                })
            }
        }
    };
}

build_client!(PyWebClient, WebClient);
build_client!(PyQualityClient, QualityClient);
build_client!(PyTimeSeriesClient, TimeSeriesClient);
build_client!(PyRealtimePredictClient, RealtimePredictClient);
build_client!(PyRealtimePredictAlterClient, RealtimePredictAlterClient);
build_client!(
    PyRealtimePredictSequenceClient,
    RealtimePredictSequenceClient
);
build_client!(PyDecisionHistoryClient, DecisionHistoryClient);
build_client!(PyDecisionMakingClient, DecisionMakingClient);
build_client!(PyOperationClient, OperationClient);
build_client!(PyModelStatusClient, ModelStatusClient);

#[pymethods]
impl PyWebClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn get_prediction_info(&self, project_name: String) -> PyResult<(String, String)> {
        self.runtime
            .block_on(self.inner.get_prediction_info(&project_name))
            .map_err(to_py_err)
    }

    fn get_rt_info(&self, py: Python, project_name: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_rt_info(&project_name))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_model_parameter(&self, project_name: String) -> PyResult<(String, String)> {
        self.runtime
            .block_on(self.inner.get_model_parameter(&project_name))
            .map_err(to_py_err)
    }

    fn get_sample_table_information(
        &self,
        py: Python,
        sample_name: String,
    ) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_sample_table_information(&sample_name))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_optimization_info(
        &self,
        py: Python,
        project_name: String,
    ) -> PyResult<(
        Py<PyDict>,
        Py<PyDict>,
        Vec<String>,
        Vec<String>,
        Vec<Vec<String>>,
    )> {
        let (args_df, cons_df, table_list, model_list, input_names) = self
            .runtime
            .block_on(self.inner.get_optimization_info(&project_name))
            .map_err(to_py_err)?;
        let args = dataframe_to_py_dict(py, &args_df)?;
        let cons = dataframe_to_py_dict(py, &cons_df)?;
        Ok((args, cons, table_list, model_list, input_names))
    }

    fn get_constraint_table(&self, py: Python, project_name: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_constraint_table(&project_name))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_constraint_table_full(&self, py: Python, project_name: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_constraint_table_full(&project_name))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn check_train_info_table(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.check_train_info_table())
            .map_err(to_py_err)
    }

    fn insert_train_info(
        &self,
        project_name: String,
        epoch: i32,
        train_loss: f64,
        vali_loss: f64,
        model_type: String,
        uuid: String,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.insert_train_info(
                &project_name,
                epoch,
                train_loss,
                vali_loss,
                &model_type,
                &uuid,
            ))
            .map_err(to_py_err)
    }

    fn init_thread_status_table(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.init_thread_status_table())
            .map_err(to_py_err)
    }

    fn fresh_thread_status_table(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.fresh_thread_status_table())
            .map_err(to_py_err)
    }

    fn update_thread_status(
        &self,
        project_name: String,
        project_type: String,
        train_status: i32,
        predict_status: i32,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.update_thread_status(
                &project_name,
                &project_type,
                train_status,
                predict_status,
            ))
            .map_err(to_py_err)
    }

    fn check_online_trainer_table(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.check_online_trainer_table())
            .map_err(to_py_err)
    }

    fn insert_online_trainer_info(
        &self,
        project_name: String,
        epoch: i32,
        train_loss: f64,
        test_loss: f64,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.insert_online_trainer_info(
                &project_name,
                epoch,
                train_loss,
                test_loss,
            ))
            .map_err(to_py_err)
    }

    fn get_online_trainer_info(&self, py: Python, project_name: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_online_trainer_info(&project_name))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn ensure_table_exist_operation_status(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.ensure_table_exist_operation_status())
            .map_err(to_py_err)
    }

    fn init_operation_status(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.init_operation_status())
            .map_err(to_py_err)
    }

    fn update_operation_status(
        &self,
        project_name: String,
        project_type: String,
        operation_status: i32,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.update_operation_status(
                &project_name,
                &project_type,
                operation_status,
            ))
            .map_err(to_py_err)
    }

    fn write_device_status(&self, project_name: String, device_status: i32) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.write_device_status(&project_name, device_status))
            .map_err(to_py_err)
    }

    fn insert_device_status(&self, project_name: String, device_status: i32) -> PyResult<()> {
        self.runtime
            .block_on(
                self.inner
                    .insert_device_status(&project_name, device_status),
            )
            .map_err(to_py_err)
    }
}

#[pymethods]
impl PyQualityClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn get_output_data_train(
        &self,
        py: Python,
        table: String,
        begin: String,
        end: String,
    ) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_output_data_train(&table, &begin, &end))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_output_data_history(
        &self,
        py: Python,
        table: String,
        begin: String,
        end: String,
    ) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_output_data_history(&table, &begin, &end))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_latest_true_value(&self, py: Python, table: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_latest_true_value(&table))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }
}

#[pymethods]
impl PyTimeSeriesClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn get_latest_input_data(&self, py: Python, table: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_latest_input_data(&table))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_latest_input_data_view(
        &self,
        py: Python,
        table: String,
        time_length: i64,
        unit: String,
    ) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(
                self.inner
                    .get_latest_input_data_view(&table, time_length, &unit),
            )
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_latest_true_value_by_column(
        &self,
        py: Python,
        col: String,
    ) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_latest_true_value_by_column(&col))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_latest_true_value(&self, py: Python, table: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_latest_true_value(&table))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_input_data_online(
        &self,
        py: Python,
        names: Vec<String>,
    ) -> PyResult<Py<PyDict>> {
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(self.inner.get_input_data_online(&name_refs))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_output_data_online(
        &self,
        py: Python,
        names: Vec<String>,
    ) -> PyResult<Py<PyDict>> {
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(self.inner.get_output_data_online(&name_refs))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_input_data_train(
        &self,
        py: Python,
        table: String,
        begin: String,
        end: String,
    ) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_input_data_train(&table, &begin, &end))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    #[pyo3(signature = (names, begin, end, chunk_size=None, use_temp_file=false))]
    fn get_input_data_history(
        &self,
        py: Python,
        names: Vec<String>,
        begin: String,
        end: String,
        chunk_size: Option<String>,
        use_temp_file: bool,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(self.inner.get_input_data_history(
                &refs,
                &begin,
                &end,
                chunk_size.as_deref(),
                use_temp_file,
            ))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    #[pyo3(signature = (names, begin, end, chunk_size=None))]
    fn get_output_data_history(
        &self,
        py: Python,
        names: Vec<String>,
        begin: String,
        end: String,
        chunk_size: Option<String>,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(self.inner.get_output_data_history(
                &refs,
                &begin,
                &end,
                chunk_size.as_deref(),
            ))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_latest_input_data_by_column(
        &self,
        py: Python,
        names: Vec<String>,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(self.inner.get_latest_input_data_by_column(&refs))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_latest_input_data_by_column_time(
        &self,
        py: Python,
        names: Vec<String>,
        hours: i64,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(
                self.inner
                    .get_latest_input_data_by_column_time(&refs, hours),
            )
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_latest_input_data_by_column_time_hour(
        &self,
        py: Python,
        names: Vec<String>,
        hours: i64,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(
                self.inner
                    .get_latest_input_data_by_column_time_hour(&refs, hours),
            )
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_latest_input_data_by_column_time_min(
        &self,
        py: Python,
        names: Vec<String>,
        minutes: i64,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(
                self.inner
                    .get_latest_input_data_by_column_time_min(&refs, minutes),
            )
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_history_data_by_name_list(
        &self,
        py: Python,
        names: Vec<String>,
        begin: String,
        end: String,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(
                self.inner
                    .get_history_data_by_name_list(&refs, &begin, &end),
            )
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn check_tagname_exist(&self, tag_name: String) -> PyResult<bool> {
        self.runtime
            .block_on(self.inner.check_tagname_exist(&tag_name))
            .map_err(to_py_err)
    }

    fn update_tagval(&self, tag_name: String, tag_val: f64) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.update_tagval(&tag_name, tag_val))
            .map_err(to_py_err)
    }

    fn get_feed_amount_column_name(&self, table: String) -> PyResult<Vec<String>> {
        self.runtime
            .block_on(self.inner.get_feed_amount_column_name(&table))
            .map_err(to_py_err)
    }

    fn check_device_running_status(&self, var_name: String) -> PyResult<bool> {
        self.runtime
            .block_on(self.inner.check_device_running_status(&var_name))
            .map_err(to_py_err)
    }

    fn get_latest_input_data_model_parameter(
        &self,
        py: Python,
        names: Vec<String>,
        filter_length: String,
        time_length: i64,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(self.inner.get_latest_input_data_model_parameter(
                &refs,
                &filter_length,
                time_length,
            ))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn get_optimization_mode(&self, optimization_procedure: String) -> PyResult<String> {
        self.runtime
            .block_on(self.inner.get_optimization_mode(&optimization_procedure))
            .map_err(to_py_err)
    }

    fn get_cement_mill_target(&self, optimization_procedure: String) -> PyResult<(i64, f64)> {
        self.runtime
            .block_on(self.inner.get_cement_mill_target(&optimization_procedure))
            .map_err(to_py_err)
    }

    fn get_cement_mill_a_target(&self) -> PyResult<(i64, f64)> {
        self.runtime
            .block_on(self.inner.get_cement_mill_a_target())
            .map_err(to_py_err)
    }

    fn get_cement_mill_b_target(&self) -> PyResult<(i64, f64)> {
        self.runtime
            .block_on(self.inner.get_cement_mill_b_target())
            .map_err(to_py_err)
    }

    fn query_decision_history_data(
        &self,
        py: Python,
        table: String,
        minutes: i64,
    ) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.query_decision_history_data(&table, minutes))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn query_decision_history_data_by_name(
        &self,
        py: Python,
        names: Vec<String>,
        time_query: i64,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(
                self.inner
                    .query_decision_history_data_by_name(&refs, time_query),
            )
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn write_decision_result(
        &self,
        optimization_project_name: String,
        optimization_name_list: Vec<String>,
        optimization_solution_results: Vec<f64>,
        optimization_type: String,
    ) -> PyResult<()> {
        let name_refs: Vec<&str> = optimization_name_list.iter().map(|s| s.as_str()).collect();
        self.runtime
            .block_on(self.inner.write_decision_result(
                &optimization_project_name,
                &name_refs,
                &optimization_solution_results,
                &optimization_type,
            ))
            .map_err(to_py_err)
    }

    fn query_history_data_latest(&self, py: Python) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.query_history_data_latest())
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn query_decision_history_data_latest_by_name_list(
        &self,
        py: Python,
        names: Vec<String>,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(
                self.inner
                    .query_decision_history_data_latest_by_name_list(&refs),
            )
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn query_tagdatabase_latest(&self, py: Python) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.query_tagdatabase_latest())
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn query_tagdatabase_latest_by_name_list(
        &self,
        py: Python,
        names: Vec<String>,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(self.inner.query_tagdatabase_latest_by_name_list(&refs))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn check_decision_table(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.check_decision_table())
            .map_err(to_py_err)
    }

    fn query_decision_solution_rt_by_column(
        &self,
        py: Python,
        columns: Vec<String>,
        optimization_type: String,
    ) -> PyResult<Py<PyDict>> {
        let refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
        let df = self
            .runtime
            .block_on(
                self.inner
                    .query_decision_solution_rt_by_column(&refs, &optimization_type),
            )
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }
}

#[pymethods]
impl PyRealtimePredictClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn check_table(&self, table: String) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.check_table(&table))
            .map_err(to_py_err)
    }

    fn update_realvalue(&self, table: String, realvalue: f64, time_real: String) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.update_realvalue(&table, realvalue, &time_real))
            .map_err(to_py_err)
    }

    fn update_predictvalue(
        &self,
        table: String,
        predictvalue: f64,
        time_predict: String,
    ) -> PyResult<()> {
        self.runtime
            .block_on(
                self.inner
                    .update_predictvalue(&table, predictvalue, &time_predict),
            )
            .map_err(to_py_err)
    }

    fn query_latest_predict_value(&self, py: Python, table: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.query_latest_predict_value(&table))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }

    fn add_ai_predict_column(&self, table: String) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.add_ai_predict_column(&table))
            .map_err(to_py_err)
    }

    fn update_ai_predict_value(&self, table: String, data: &Bound<'_, PyDict>) -> PyResult<()> {
        let df = py_dict_to_dataframe(data)?;
        self.runtime
            .block_on(self.inner.update_ai_predict_value(&table, &df))
            .map_err(to_py_err)
    }

    fn update_ai_predict_index(
        &self,
        table: String,
        ai_index: f64,
        time_predict: String,
    ) -> PyResult<()> {
        self.runtime
            .block_on(
                self.inner
                    .update_ai_predict_index(&table, ai_index, &time_predict),
            )
            .map_err(to_py_err)
    }
}

#[pymethods]
impl PyRealtimePredictAlterClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn check_table(&self, table: String) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.check_table(&table))
            .map_err(to_py_err)
    }

    fn update_predictvalue(
        &self,
        table: String,
        predictvalue: f64,
        time_predict: String,
    ) -> PyResult<()> {
        self.runtime
            .block_on(
                self.inner
                    .update_predictvalue(&table, predictvalue, &time_predict),
            )
            .map_err(to_py_err)
    }

    fn get_predict_data_by_table_name(&self, py: Python, table: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_predict_data_by_table_name(&table))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }
}

#[pymethods]
impl PyRealtimePredictSequenceClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn create_sequence_table(&self, table: String, names: Vec<String>) -> PyResult<()> {
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        self.runtime
            .block_on(self.inner.create_sequence_table(&table, &name_refs))
            .map_err(to_py_err)
    }

    fn create_sequence_predict_table(&self, table: String, names: Vec<String>) -> PyResult<()> {
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        self.runtime
            .block_on(self.inner.create_sequence_predict_table(&table, &name_refs))
            .map_err(to_py_err)
    }

    fn update_realvalue(
        &self,
        _py: Python,
        table: String,
        data: &Bound<'_, PyDict>,
    ) -> PyResult<()> {
        let df = py_dict_to_dataframe(data)?;
        self.runtime
            .block_on(self.inner.update_realvalue(&table, &df))
            .map_err(to_py_err)
    }

    fn get_predict_data_by_table_name(&self, py: Python, table: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_predict_data_by_table_name(&table))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }
}

#[pymethods]
impl PyDecisionHistoryClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn check_table_decision_history(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.check_table_decision_history())
            .map_err(to_py_err)
    }

    fn insert_decision_history(
        &self,
        datetime: String,
        variable_name: String,
        project_name: String,
        opt_type: String,
        decision_value: f64,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.insert_decision_history(
                &datetime,
                &variable_name,
                &project_name,
                &opt_type,
                decision_value,
            ))
            .map_err(to_py_err)
    }

    fn query_decision_history_recent_1h(&self, py: Python) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.query_decision_history_recent_1h())
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }
}

#[pymethods]
impl PyDecisionMakingClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn check_decision_table(&self) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.check_decision_table())
            .map_err(to_py_err)
    }
}

#[pymethods]
impl PyOperationClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn check_operation_table(&self, project_name: String) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.check_operation_table(&project_name))
            .map_err(to_py_err)
    }

    fn insert_operation_data(
        &self,
        project_name: String,
        epoch: i32,
        train_loss: f64,
        vali_loss: f64,
        sample_size: i32,
        start_time: String,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.insert_operation_data(
                &project_name,
                epoch,
                train_loss,
                vali_loss,
                sample_size,
                &start_time,
            ))
            .map_err(to_py_err)
    }

    fn get_operation_data(&self, py: Python, project_name: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_operation_data(&project_name))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }
}

#[pymethods]
impl PyModelStatusClient {
    #[new]
    fn py_new(config: &PyDatabaseConfig) -> PyResult<Self> {
        Self::new_with_config(config)
    }

    fn init_model_status_table(&self, project_name: String) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.init_model_status_table(&project_name))
            .map_err(to_py_err)
    }

    fn insert_model_status(
        &self,
        project_name: String,
        algorithm_name: String,
        model_type: String,
        model_parameter_json: String,
        start_time: String,
        end_time: String,
        sample_size: i32,
        mae: f64,
        rmse: f64,
        mape: f64,
    ) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.insert_model_status(
                &project_name,
                &algorithm_name,
                &model_type,
                &model_parameter_json,
                &start_time,
                &end_time,
                sample_size,
                mae,
                rmse,
                mape,
            ))
            .map_err(to_py_err)
    }

    fn get_model_status(&self, py: Python, project_name: String) -> PyResult<Py<PyDict>> {
        let df = self
            .runtime
            .block_on(self.inner.get_model_status(&project_name))
            .map_err(to_py_err)?;
        dataframe_to_py_dict(py, &df)
    }
}

/// Top-level factory helpers to mirror create_*_client API.
#[pyfunction]
pub fn create_web_client(config: &PyDatabaseConfig) -> PyResult<PyWebClient> {
    PyWebClient::new_with_config(config)
}

#[pyfunction]
pub fn create_quality_client(config: &PyDatabaseConfig) -> PyResult<PyQualityClient> {
    PyQualityClient::new_with_config(config)
}

#[pyfunction]
pub fn create_timeseries_client(config: &PyDatabaseConfig) -> PyResult<PyTimeSeriesClient> {
    PyTimeSeriesClient::new_with_config(config)
}

#[pyfunction]
pub fn create_realtime_predict_client(
    config: &PyDatabaseConfig,
) -> PyResult<PyRealtimePredictClient> {
    PyRealtimePredictClient::new_with_config(config)
}

#[pyfunction]
pub fn create_realtime_predict_alter_client(
    config: &PyDatabaseConfig,
) -> PyResult<PyRealtimePredictAlterClient> {
    PyRealtimePredictAlterClient::new_with_config(config)
}

#[pyfunction]
pub fn create_realtime_predict_sequence_client(
    config: &PyDatabaseConfig,
) -> PyResult<PyRealtimePredictSequenceClient> {
    PyRealtimePredictSequenceClient::new_with_config(config)
}

#[pyfunction]
pub fn create_decision_history_client(
    config: &PyDatabaseConfig,
) -> PyResult<PyDecisionHistoryClient> {
    PyDecisionHistoryClient::new_with_config(config)
}

#[pyfunction]
pub fn create_decision_making_client(
    config: &PyDatabaseConfig,
) -> PyResult<PyDecisionMakingClient> {
    PyDecisionMakingClient::new_with_config(config)
}

#[pyfunction]
pub fn create_operation_client(config: &PyDatabaseConfig) -> PyResult<PyOperationClient> {
    PyOperationClient::new_with_config(config)
}

#[pyfunction]
pub fn create_model_status_client(config: &PyDatabaseConfig) -> PyResult<PyModelStatusClient> {
    PyModelStatusClient::new_with_config(config)
}

#[pyfunction]
pub fn create_online_trainer_client(config: &PyDatabaseConfig) -> PyResult<PyWebClient> {
    PyWebClient::new_with_config(config)
}
