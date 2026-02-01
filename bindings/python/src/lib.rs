//! Python bindings for Chakra ORM
//!
//! This crate provides Python bindings using PyO3.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

mod connection;
mod model;
mod query;
mod types;

use connection::{PyConnection, PyPool};
use model::PyModel;
use query::PyQueryBuilder;
use types::PyValue;

/// Chakra ORM Python module
#[pymodule]
fn chakra(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register classes
    m.add_class::<PyConnection>()?;
    m.add_class::<PyPool>()?;
    m.add_class::<PyQueryBuilder>()?;
    m.add_class::<PyValue>()?;

    // Register functions
    m.add_function(wrap_pyfunction!(connect, m)?)?;
    m.add_function(wrap_pyfunction!(connect_async, m)?)?;

    // Add version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}

/// Connect to a database synchronously
#[pyfunction]
fn connect(url: &str) -> PyResult<PyConnection> {
    // TODO: Implement sync connection
    Err(pyo3::exceptions::PyNotImplementedError::new_err(
        "Synchronous connection not yet implemented",
    ))
}

/// Connect to a database asynchronously
#[pyfunction]
fn connect_async<'py>(py: Python<'py>, url: &str) -> PyResult<Bound<'py, PyAny>> {
    let url = url.to_string();

    pyo3_asyncio_0_21::tokio::future_into_py(py, async move {
        // TODO: Implement async connection
        Err::<PyConnection, _>(pyo3::exceptions::PyNotImplementedError::new_err(
            "Async connection not yet implemented",
        ))
    })
}
