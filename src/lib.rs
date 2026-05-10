use pyo3::prelude::*;
use serde_json::{json, Value};

use grep::regex::RegexMatcherBuilder;
use grep::searcher::{SearcherBuilder, sinks::UTF8};
use ignore::WalkBuilder;

use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use globset::{Glob, GlobSet, GlobSetBuilder};

use crossbeam_channel::{bounded, Receiver};
use std::thread;

// glob helper
fn build_glob(glob: &Option<String>) -> Option<GlobSet> {
    if let Some(g) = glob {
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new(g).unwrap());
        Some(builder.build().unwrap())
    } else {
        None
    }
}

// batch search
#[pyfunction]
fn search(
    pattern: String,
    root: String,
    glob: Option<String>,
    max_results: Option<usize>,
    ignore_case: Option<bool>,
    json: Option<bool>,
) -> PyResult<PyObject> {
    let is_case_insensitive = ignore_case.unwrap_or(false);
    let return_json = json.unwrap_or(false);
    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(is_case_insensitive)
        .build(&pattern)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

    let glob_matcher: Option<GlobSet> = build_glob(&glob);

    let results = Arc::new(Mutex::new(Vec::new()));

    let entries: Vec<_> = WalkBuilder::new(&root)
        .standard_filters(true)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .filter(|entry| {
            if let Some(ref gs) = glob_matcher {
                gs.is_match(entry.path())
            } else {
                true
            }
        })
        .collect();

    entries.par_iter().for_each(|entry| {
        let path = entry.path();

        // Optional: skip empty files (cheap win)
        if path.metadata().map(|m| m.len() == 0).unwrap_or(false) {
            return;
        }

        let mut searcher = SearcherBuilder::new().build();
        let results = Arc::clone(&results);

        let _ = searcher.search_path(
            &matcher,
            path,
            UTF8(|lnum, line| {
                let mut res = results.lock().unwrap();

                if let Some(max) = max_results {
                    if res.len() >= max {
                        return Ok(false); // early exit
                    }
                }

                res.push((
                    path.display().to_string(),
                    lnum as usize,
                    line.to_string(),
                ));

                Ok(true)
            }),
        );
    });

    let final_results = Arc::try_unwrap(results)
        .unwrap()
        .into_inner()
        .unwrap();

    if return_json {
        let json_results: Vec<Value> = final_results.into_iter()
            .map(|(file, line, content)| {
                json!({
                    "file": file,
                    "line": line,
                    "content": content.trim_end()
                })
            })
            .collect();
        
        let json_string = serde_json::to_string(&json_results).unwrap();
        Python::with_gil(|py| {
            let json_module = py.import("json")?;
            let json_obj = json_module.call_method("loads", (json_string,), None)?;
            Ok(json_obj.into())
        })
    } else {
        Python::with_gil(|py| {
            Ok(final_results.into_py(py))
        })
    }
}

// streaming iterator 
#[pyclass]
struct PyResultIterator {
    receiver: Receiver<(String, usize, String)>,
    json_mode: bool,
}

#[pymethods]
impl PyResultIterator {
    fn __iter__(slf: PyRef<Self>) -> Py<PyResultIterator> {
        slf.into()
    }

    fn __next__(slf: PyRefMut<Self>) -> Option<PyObject> {
        if let Some((file, line, content)) = slf.receiver.recv().ok() {
            if slf.json_mode {
                let py = slf.py();
                let json_obj = json!({
                    "file": file,
                    "line": line,
                    "content": content.trim_end()
                });
                let json_string = serde_json::to_string(&json_obj).unwrap();
                match py.import("json").and_then(|m| m.call_method("loads", (json_string,), None)) {
                    Ok(parsed) => Some(parsed.into()),
                    Err(_) => None,
                }
            } else {
                Some((file, line, content).into_py(slf.py()))
            }
        } else {
            None
        }
    }
}

#[pyfunction]
fn search_iter(
    pattern: String,
    root: String,
    glob: Option<String>,
    ignore_case: Option<bool>,
    json: Option<bool>,
) -> PyResult<PyResultIterator> {
    let (tx, rx) = bounded(1000);

    let is_case_insensitive = ignore_case.unwrap_or(false);
    let return_json = json.unwrap_or(false);

    thread::spawn(move || {
        let matcher = match RegexMatcherBuilder::new()
            .case_insensitive(is_case_insensitive)
            .build(&pattern)
        {
            Ok(m) => m,
            Err(_) => return,
        };

        let glob_matcher: Option<GlobSet> = build_glob(&glob);

        let walker = WalkBuilder::new(&root)
            .standard_filters(true)
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }

            if let Some(ref gs) = glob_matcher {
                if !gs.is_match(entry.path()) {
                    continue;
                }
            }

            let path = entry.path().to_path_buf();

            // skip empty files
            if path.metadata().map(|m| m.len() == 0).unwrap_or(false) {
                continue;
            }

            let mut searcher = SearcherBuilder::new().build();

            let _ = searcher.search_path(
                &matcher,
                &path,
                UTF8(|lnum, line| {
                    if tx.send((
                        path.display().to_string(),
                        lnum as usize,
                        line.to_string(),
                    )).is_err() {
                        return Ok(false); // stop if receiver gone
                    }
                    Ok(true)
                }),
            );
        }
    });

    Ok(PyResultIterator { receiver: rx, json_mode: return_json })
}

// python module
#[pymodule]
fn pyfastgrep(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(search, m)?)?;
    m.add_function(wrap_pyfunction!(search_iter, m)?)?;
    Ok(())
}