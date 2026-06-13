use crate::common::*;
use crate::utils::*;
use pyfastgrep_core::{
    search as core_search, search_count as core_search_count,
    search_files_with_matches as core_search_files_with_matches,
    search_stream as core_search_stream, SearchReceiver,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::fs::File;
use std::io::Write;

#[pyclass]
pub struct PyResultIterator {
    receiver: SearchReceiver,
    json_mode: bool,
    csv_mode: bool,
    csv_header_emitted: bool,
    csv_writer: Option<File>,
}

#[pymethods]
impl PyResultIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Py<PyAny>> {
        if slf.csv_mode && !slf.csv_header_emitted {
            slf.csv_header_emitted = true;
            if let Some(writer) = slf.csv_writer.as_mut() {
                let _ = writer.write_all(b"file,line,content\n");
            }
            return Python::attach(|py| {
                Some(
                    "file,line,content\n"
                        .into_pyobject(py)
                        .ok()?
                        .into_any()
                        .unbind(),
                )
            });
        }

        let hit = slf.receiver.recv().ok()?;

        Python::attach(|py| {
            if slf.json_mode {
                let mut hit_clone = hit.clone();
                hit_clone.content = hit_clone.content.trim_end().to_string();
                let json_string = serde_json::to_string(&hit_clone).unwrap();
                let json_module = py.import("json").ok()?;
                let parsed = json_module
                    .call_method("loads", (json_string,), None)
                    .ok()?;
                Some(parsed.into())
            } else if slf.csv_mode {
                let row = hit_to_csv_row(&hit.file, hit.line, &hit.content);
                if let Some(writer) = slf.csv_writer.as_mut() {
                    let _ = writer.write_all(row.as_bytes());
                }
                Some(row.into_pyobject(py).ok()?.into_any().unbind())
            } else {
                Some(
                    (hit.file, hit.line, hit.content)
                        .into_pyobject(py)
                        .ok()?
                        .into_any()
                        .unbind(),
                )
            }
        })
    }
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (pattern, root, glob=None, max_results=None, ignore_case=None, json=None, csv=None, output_path=None, fixed_strings=None, byte_offset=None))]
pub fn search(
    pattern: String,
    root: String,
    glob: Option<String>,
    max_results: Option<usize>,
    ignore_case: Option<bool>,
    json: Option<bool>,
    csv: Option<bool>,
    output_path: Option<String>,
    fixed_strings: Option<bool>,
    byte_offset: Option<bool>,
) -> PyResult<Py<PyAny>> {
    let config = build_config(
        pattern,
        root,
        glob,
        max_results,
        ignore_case,
        fixed_strings,
        byte_offset,
    );
    let return_json = json.unwrap_or(false);
    let return_csv = csv.unwrap_or(false);

    if return_json && return_csv {
        return Err(PyValueError::new_err(
            "json and csv output modes are mutually exclusive",
        ));
    }

    if output_path.is_some() && !return_csv {
        return Err(PyValueError::new_err(
            "output_path is only supported with csv output",
        ));
    }

    let hits = core_search(&config).map_err(PyValueError::new_err)?;

    Python::attach(|py| {
        if return_json {
            hits_to_json(py, hits)
        } else if return_csv {
            let csv_output = hits_to_csv(py, hits)?;
            if let Some(path) = output_path.as_deref() {
                let csv_string: String = csv_output.extract(py)?;
                write_csv_file(path, &csv_string).map_err(PyValueError::new_err)?;
            }
            Ok(csv_output)
        } else {
            hits_to_tuples(py, hits)
        }
    })
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (pattern, root, glob=None, ignore_case=None, json=None, csv=None, output_path=None, fixed_strings=None, byte_offset=None))]
pub fn search_iter(
    pattern: String,
    root: String,
    glob: Option<String>,
    ignore_case: Option<bool>,
    json: Option<bool>,
    csv: Option<bool>,
    output_path: Option<String>,
    fixed_strings: Option<bool>,
    byte_offset: Option<bool>,
) -> PyResult<PyResultIterator> {
    let _ = byte_offset; // Not used for streaming search yet
    let config = build_config(pattern, root, glob, None, ignore_case, fixed_strings, None);
    let receiver = core_search_stream(config).map_err(PyValueError::new_err)?;

    let return_json = json.unwrap_or(false);
    let return_csv = csv.unwrap_or(false);

    if return_json && return_csv {
        return Err(PyValueError::new_err(
            "json and csv output modes are mutually exclusive",
        ));
    }

    if output_path.is_some() && !return_csv {
        return Err(PyValueError::new_err(
            "output_path is only supported with csv output",
        ));
    }

    let csv_writer = if let Some(path) = output_path.as_deref() {
        let mut file = File::create(path).map_err(PyValueError::new_err)?;
        file.write_all(b"file,line,content\n")
            .map_err(PyValueError::new_err)?;
        Some(file)
    } else {
        None
    };

    Ok(PyResultIterator {
        receiver,
        json_mode: return_json,
        csv_mode: return_csv,
        csv_header_emitted: false,
        csv_writer,
    })
}

#[pyfunction]
#[pyo3(signature = (pattern, root, glob=None, ignore_case=None, fixed_strings=None))]
pub fn search_count(
    pattern: String,
    root: String,
    glob: Option<String>,
    ignore_case: Option<bool>,
    fixed_strings: Option<bool>,
) -> PyResult<Vec<(String, usize)>> {
    let config = build_config(pattern, root, glob, None, ignore_case, fixed_strings, None);
    core_search_count(&config).map_err(PyValueError::new_err)
}

#[pyfunction]
#[pyo3(signature = (pattern, root, glob=None, ignore_case=None, fixed_strings=None))]
pub fn search_files_with_matches(
    pattern: String,
    root: String,
    glob: Option<String>,
    ignore_case: Option<bool>,
    fixed_strings: Option<bool>,
) -> PyResult<Vec<String>> {
    let config = build_config(pattern, root, glob, None, ignore_case, fixed_strings, None);
    core_search_files_with_matches(&config).map_err(PyValueError::new_err)
}
