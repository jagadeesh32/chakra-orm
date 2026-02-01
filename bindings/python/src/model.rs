//! Model types for Python bindings

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyType};
use std::collections::HashMap;

/// Base model class for Python
#[pyclass(subclass)]
pub struct PyModel {
    data: HashMap<String, PyObject>,
}

#[pymethods]
impl PyModel {
    #[new]
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Get a field value
    fn get(&self, py: Python<'_>, key: &str) -> Option<PyObject> {
        self.data.get(key).map(|v| v.clone_ref(py))
    }

    /// Set a field value
    fn set(&mut self, key: &str, value: PyObject) {
        self.data.insert(key.to_string(), value);
    }

    /// Get all data as a dict
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new_bound(py);
        for (k, v) in &self.data {
            dict.set_item(k, v.clone_ref(py))?;
        }
        Ok(dict)
    }

    /// Create from a dict
    #[classmethod]
    fn from_dict(_cls: &Bound<'_, PyType>, dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut data = HashMap::new();
        for (key, value) in dict.iter() {
            let key_str: String = key.extract()?;
            data.insert(key_str, value.into_py(dict.py()));
        }
        Ok(Self { data })
    }

    fn __str__(&self) -> String {
        format!("Model({:?})", self.data.keys().collect::<Vec<_>>())
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}

/// Model metadata for Python
#[pyclass]
pub struct PyModelMeta {
    /// Model name
    #[pyo3(get)]
    name: String,
    /// Table name
    #[pyo3(get)]
    table: String,
    /// Field names
    #[pyo3(get)]
    fields: Vec<String>,
    /// Primary key fields
    #[pyo3(get)]
    primary_key: Vec<String>,
}

#[pymethods]
impl PyModelMeta {
    #[new]
    fn new(name: &str, table: &str) -> Self {
        Self {
            name: name.to_string(),
            table: table.to_string(),
            fields: Vec::new(),
            primary_key: Vec::new(),
        }
    }

    fn add_field(&mut self, name: &str) {
        self.fields.push(name.to_string());
    }

    fn set_primary_key(&mut self, fields: Vec<String>) {
        self.primary_key = fields;
    }

    fn __str__(&self) -> String {
        format!("ModelMeta(name={}, table={})", self.name, self.table)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }
}
