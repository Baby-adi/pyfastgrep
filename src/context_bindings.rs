use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyfastgrep_core::{
    search_with_context as core_search_with_context,
    ContextConfig,
};
use crate::common::build_config;

#[pyfunction]
#[pyo3(signature = (pattern, root, glob=None, before_context=0, after_context=0, ignore_case=None, fixed_strings=None))]
pub fn search_with_context(
    pattern: String,
    root: String,
    glob: Option<String>,
    before_context: usize,
    after_context: usize,
    ignore_case: Option<bool>,
    fixed_strings: Option<bool>,
) -> PyResult<Vec<(String, usize, String, Vec<String>, Vec<String>)>> {
    let config = ContextConfig {
        base: build_config(pattern, root, glob, None, ignore_case, fixed_strings),
        before_context,
        after_context,
    };
    
    let hits = core_search_with_context(&config).map_err(PyValueError::new_err)?;
    
    let results: Vec<_> = hits.into_iter().map(|hit| {
        let before: Vec<String> = hit.before_context.into_iter().map(|ctx| ctx.content).collect();
        let after: Vec<String> = hit.after_context.into_iter().map(|ctx| ctx.content).collect();
        (hit.file, hit.line, hit.content, before, after)
    }).collect();
    
    Ok(results)
}
