use crate::utils::*;
use pyfastgrep_core::{SearchConfig, SearchHit};
use pyo3::prelude::*;
use std::path::PathBuf;

pub fn build_config(
    pattern: String,
    root: String,
    glob: Option<String>,
    max_results: Option<usize>,
    ignore_case: Option<bool>,
    fixed_strings: Option<bool>,
    byte_offset: Option<bool>,
) -> SearchConfig {
    SearchConfig {
        pattern,
        root: PathBuf::from(root),
        glob,
        max_results,
        ignore_case: ignore_case.unwrap_or(false),
        fixed_strings: fixed_strings.unwrap_or(false),
        byte_offset: byte_offset.unwrap_or(false),
    }
}

pub fn hits_to_json(py: Python<'_>, mut hits: Vec<SearchHit>) -> PyResult<Py<PyAny>> {
    // Trim trailing whitespace from content before serialization
    for hit in &mut hits {
        hit.content = hit.content.trim_end().to_string();
    }

    let json_string = serde_json::to_string(&hits).unwrap();
    let json_module = py.import("json")?;
    let parsed = json_module.call_method("loads", (json_string,), None)?;
    Ok(parsed.into())
}

pub fn hits_to_tuples(py: Python<'_>, hits: Vec<SearchHit>) -> PyResult<Py<PyAny>> {
    let tuples: Vec<(String, usize, String)> = hits
        .into_iter()
        .map(|hit| (hit.file, hit.line, hit.content))
        .collect();

    Ok(tuples.into_pyobject(py)?.into_any().unbind())
}

pub fn hits_to_csv(py: Python<'_>, hits: Vec<SearchHit>) -> PyResult<Py<PyAny>> {
    let mut csv_output = hits_to_csv_header();

    for hit in hits {
        csv_output.push_str(&hit_to_csv_row(&hit.file, hit.line, &hit.content));
    }

    Ok(csv_output.into_pyobject(py)?.into_any().unbind())
}
