use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyfastgrep_core::{
    search_ast, search_ast_stream,
    AstQueryType, AstResultReceiver,
};
use std::path::PathBuf;

#[pyclass]
pub struct PyAstResultIterator {
    receiver: AstResultReceiver,
}

#[pymethods]
impl PyAstResultIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        let item = slf.receiver.recv().ok()?;
        Python::attach(|py| {
            Some(item.into_pyobject(py).ok()?.into_any().unbind())
        })
    }
}

#[pyfunction]
#[pyo3(signature = (target_name, root, glob=None))]
pub fn search_functions(
    target_name: String,
    root: String,
    glob: Option<String>,
) -> PyResult<Vec<(String, usize, String)>> {
    search_ast(&target_name, &PathBuf::from(&root), &glob, AstQueryType::Function)
        .map_err(PyValueError::new_err)
}

#[pyfunction]
#[pyo3(signature = (target_name, root, glob=None))]
pub fn search_classes(
    target_name: String,
    root: String,
    glob: Option<String>,
) -> PyResult<Vec<(String, usize, String)>> {
    search_ast(&target_name, &PathBuf::from(&root), &glob, AstQueryType::Class)
        .map_err(PyValueError::new_err)
}

#[pyfunction]
#[pyo3(signature = (target_name, root, glob=None))]
pub fn search_imports(
    target_name: String,
    root: String,
    glob: Option<String>,
) -> PyResult<Vec<(String, usize, String)>> {
    search_ast(&target_name, &PathBuf::from(&root), &glob, AstQueryType::Import)
        .map_err(PyValueError::new_err)
}

#[pyfunction]
#[pyo3(signature = (target_name, root, glob=None))]
pub fn search_functions_iter(
    target_name: String,
    root: String,
    glob: Option<String>,
) -> PyResult<PyAstResultIterator> {
    let rx = search_ast_stream(target_name, root, glob, AstQueryType::Function)
        .map_err(PyValueError::new_err)?;
    Ok(PyAstResultIterator { receiver: rx })
}

#[pyfunction]
#[pyo3(signature = (target_name, root, glob=None))]
pub fn search_classes_iter(
    target_name: String,
    root: String,
    glob: Option<String>,
) -> PyResult<PyAstResultIterator> {
    let rx = search_ast_stream(target_name, root, glob, AstQueryType::Class)
        .map_err(PyValueError::new_err)?;
    Ok(PyAstResultIterator { receiver: rx })
}

#[pyfunction]
#[pyo3(signature = (target_name, root, glob=None))]
pub fn search_imports_iter(
    target_name: String,
    root: String,
    glob: Option<String>,
) -> PyResult<PyAstResultIterator> {
    let rx = search_ast_stream(target_name, root, glob, AstQueryType::Import)
        .map_err(PyValueError::new_err)?;
    Ok(PyAstResultIterator { receiver: rx })
}
