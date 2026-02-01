//! Type conversions for Python bindings

use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyFloat, PyInt, PyList, PyNone, PyString};

/// Python value wrapper
#[pyclass]
#[derive(Clone)]
pub struct PyValue {
    inner: ValueKind,
}

#[derive(Clone)]
enum ValueKind {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<PyValue>),
    Dict(Vec<(String, PyValue)>),
}

#[pymethods]
impl PyValue {
    /// Create a null value
    #[staticmethod]
    fn null() -> Self {
        Self { inner: ValueKind::Null }
    }

    /// Create from bool
    #[staticmethod]
    fn from_bool(v: bool) -> Self {
        Self { inner: ValueKind::Bool(v) }
    }

    /// Create from int
    #[staticmethod]
    fn from_int(v: i64) -> Self {
        Self { inner: ValueKind::Int(v) }
    }

    /// Create from float
    #[staticmethod]
    fn from_float(v: f64) -> Self {
        Self { inner: ValueKind::Float(v) }
    }

    /// Create from string
    #[staticmethod]
    fn from_string(v: &str) -> Self {
        Self { inner: ValueKind::String(v.to_string()) }
    }

    /// Create from bytes
    #[staticmethod]
    fn from_bytes(v: &[u8]) -> Self {
        Self { inner: ValueKind::Bytes(v.to_vec()) }
    }

    /// Check if null
    fn is_null(&self) -> bool {
        matches!(self.inner, ValueKind::Null)
    }

    /// Convert to Python object
    fn to_python<'py>(&self, py: Python<'py>) -> PyObject {
        match &self.inner {
            ValueKind::Null => py.None(),
            ValueKind::Bool(b) => b.into_py(py),
            ValueKind::Int(i) => i.into_py(py),
            ValueKind::Float(f) => f.into_py(py),
            ValueKind::String(s) => s.into_py(py),
            ValueKind::Bytes(b) => PyBytes::new_bound(py, b).into_py(py),
            ValueKind::List(l) => {
                let list = PyList::empty_bound(py);
                for v in l {
                    list.append(v.to_python(py)).unwrap();
                }
                list.into_py(py)
            }
            ValueKind::Dict(d) => {
                let dict = PyDict::new_bound(py);
                for (k, v) in d {
                    dict.set_item(k, v.to_python(py)).unwrap();
                }
                dict.into_py(py)
            }
        }
    }

    fn __str__(&self) -> String {
        match &self.inner {
            ValueKind::Null => "null".to_string(),
            ValueKind::Bool(b) => b.to_string(),
            ValueKind::Int(i) => i.to_string(),
            ValueKind::Float(f) => f.to_string(),
            ValueKind::String(s) => format!("\"{}\"", s),
            ValueKind::Bytes(b) => format!("<bytes len={}>", b.len()),
            ValueKind::List(l) => format!("[{} items]", l.len()),
            ValueKind::Dict(d) => format!("{{{} items}}", d.len()),
        }
    }

    fn __repr__(&self) -> String {
        format!("Value({})", self.__str__())
    }
}

/// Convert Python object to Chakra Value
pub fn py_to_value(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<chakra_core::types::Value> {
    if obj.is_none() {
        return Ok(chakra_core::types::Value::Null);
    }

    if let Ok(b) = obj.extract::<bool>() {
        return Ok(chakra_core::types::Value::Bool(b));
    }

    if let Ok(i) = obj.extract::<i64>() {
        return Ok(chakra_core::types::Value::Int64(i));
    }

    if let Ok(f) = obj.extract::<f64>() {
        return Ok(chakra_core::types::Value::Float64(f));
    }

    if let Ok(s) = obj.extract::<String>() {
        return Ok(chakra_core::types::Value::String(s));
    }

    if let Ok(b) = obj.extract::<Vec<u8>>() {
        return Ok(chakra_core::types::Value::Bytes(b));
    }

    // Default to string representation
    let s = obj.str()?.to_string();
    Ok(chakra_core::types::Value::String(s))
}

/// Convert Chakra Value to Python object
pub fn value_to_py(py: Python<'_>, value: &chakra_core::types::Value) -> PyObject {
    match value {
        chakra_core::types::Value::Null => py.None(),
        chakra_core::types::Value::Bool(b) => b.into_py(py),
        chakra_core::types::Value::Int32(i) => i.into_py(py),
        chakra_core::types::Value::Int64(i) => i.into_py(py),
        chakra_core::types::Value::Float64(f) => f.into_py(py),
        chakra_core::types::Value::Decimal(d) => d.to_string().into_py(py),
        chakra_core::types::Value::String(s) => s.into_py(py),
        chakra_core::types::Value::Bytes(b) => PyBytes::new_bound(py, b).into_py(py),
        chakra_core::types::Value::Uuid(u) => u.to_string().into_py(py),
        chakra_core::types::Value::DateTime(dt) => dt.to_rfc3339().into_py(py),
        chakra_core::types::Value::Date(d) => d.to_string().into_py(py),
        chakra_core::types::Value::Time(t) => t.to_string().into_py(py),
        chakra_core::types::Value::Json(j) => j.to_string().into_py(py),
        chakra_core::types::Value::Array(arr) => {
            let list = PyList::empty_bound(py);
            for v in arr {
                list.append(value_to_py(py, v)).unwrap();
            }
            list.into_py(py)
        }
    }
}
