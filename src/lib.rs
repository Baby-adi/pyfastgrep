use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyfastgrep_core::{search as core_search, search_stream as core_search_stream, SearchConfig, SearchHit, SearchReceiver};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn build_config(
    pattern: String,
    root: String,
    glob: Option<String>,
    max_results: Option<usize>,
    ignore_case: Option<bool>,
) -> SearchConfig {
    SearchConfig {
        pattern,
        root: PathBuf::from(root),
        glob,
        max_results,
        ignore_case: ignore_case.unwrap_or(false),
    }
}

fn hits_to_json(py: Python<'_>, hits: Vec<SearchHit>) -> PyResult<PyObject> {
    let json_results: Vec<Value> = hits
        .into_iter()
        .map(|hit| {
            json!({
                "file": hit.file,
                "line": hit.line,
                "content": hit.content.trim_end()
            })
        })
        .collect();

    let json_string = serde_json::to_string(&json_results).unwrap();
    let json_module = py.import("json")?;
    let parsed = json_module.call_method("loads", (json_string,), None)?;
    Ok(parsed.into())
}

fn hits_to_tuples(py: Python<'_>, hits: Vec<SearchHit>) -> PyResult<PyObject> {
    let tuples: Vec<(String, usize, String)> = hits
        .into_iter()
        .map(|hit| (hit.file, hit.line, hit.content))
        .collect();

    Ok(tuples.into_py(py))
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn hit_to_csv_row(hit: &SearchHit) -> String {
    format!(
        "{},{},{}\n",
        csv_escape(&hit.file),
        hit.line,
        csv_escape(hit.content.trim_end())
    )
}

fn hits_to_csv(py: Python<'_>, hits: Vec<SearchHit>) -> PyResult<PyObject> {
    let mut csv_output = String::from("file,line,content\n");

    for hit in hits {
        csv_output.push_str(&hit_to_csv_row(&hit));
    }

    Ok(csv_output.into_py(py))
}

fn write_csv_file(output_path: &str, csv_content: &str) -> Result<(), String> {
    let mut file = File::create(output_path).map_err(|e| e.to_string())?;
    file.write_all(csv_content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[pyfunction]
fn search(
    pattern: String,
    root: String,
    glob: Option<String>,
    max_results: Option<usize>,
    ignore_case: Option<bool>,
    json: Option<bool>,
    csv: Option<bool>,
    output_path: Option<String>,
) -> PyResult<PyObject> {
    let config = build_config(pattern, root, glob, max_results, ignore_case);
    let return_json = json.unwrap_or(false);
    let return_csv = csv.unwrap_or(false);

    if return_json && return_csv {
        return Err(PyValueError::new_err("json and csv output modes are mutually exclusive"));
    }

    if output_path.is_some() && !return_csv {
        return Err(PyValueError::new_err("output_path is only supported with csv output"));
    }

    let hits = core_search(&config).map_err(PyValueError::new_err)?;

    Python::with_gil(|py| {
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

#[pyclass]
struct PyResultIterator {
    receiver: SearchReceiver,
    json_mode: bool,
    csv_mode: bool,
    csv_header_emitted: bool,
    csv_writer: Option<File>,
}

#[pymethods]
impl PyResultIterator {
    fn __iter__(slf: PyRef<Self>) -> Py<PyResultIterator> {
        slf.into()
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        if slf.csv_mode && !slf.csv_header_emitted {
            slf.csv_header_emitted = true;
            if let Some(writer) = slf.csv_writer.as_mut() {
                let _ = writer.write_all(b"file,line,content\n");
            }
            return Python::with_gil(|py| Some("file,line,content\n".into_py(py)));
        }

        let hit = slf.receiver.recv().ok()?;

        Python::with_gil(|py| {
            if slf.json_mode {
                let json_obj = json!({
                    "file": hit.file,
                    "line": hit.line,
                    "content": hit.content.trim_end()
                });
                let json_string = serde_json::to_string(&json_obj).unwrap();
                let json_module = py.import("json").ok()?;
                let parsed = json_module.call_method("loads", (json_string,), None).ok()?;
                Some(parsed.into())
            } else if slf.csv_mode {
                let row = hit_to_csv_row(&hit);
                if let Some(writer) = slf.csv_writer.as_mut() {
                    let _ = writer.write_all(row.as_bytes());
                }
                Some(row.into_py(py))
            } else {
                Some((hit.file, hit.line, hit.content).into_py(py))
            }
        })
    }
}

#[pyfunction]
fn search_iter(
    pattern: String,
    root: String,
    glob: Option<String>,
    ignore_case: Option<bool>,
    json: Option<bool>,
    csv: Option<bool>,
    output_path: Option<String>,
) -> PyResult<PyResultIterator> {
    let config = build_config(pattern, root, glob, None, ignore_case);
    let receiver = core_search_stream(config).map_err(PyValueError::new_err)?;

    let return_json = json.unwrap_or(false);
    let return_csv = csv.unwrap_or(false);

    if return_json && return_csv {
        return Err(PyValueError::new_err("json and csv output modes are mutually exclusive"));
    }

    if output_path.is_some() && !return_csv {
        return Err(PyValueError::new_err("output_path is only supported with csv output"));
    }

    let csv_writer = if let Some(path) = output_path.as_deref() {
        let mut file = File::create(path).map_err(PyValueError::new_err)?;
        file.write_all(b"file,line,content\n").map_err(PyValueError::new_err)?;
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

#[pymodule]
fn pyfastgrep(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(search_iter, m)?)?;
    Ok(())
}
