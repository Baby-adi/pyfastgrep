use pyo3::prelude::*;

mod ast_bindings;
mod common;
mod context_bindings;
mod regex_bindings;
mod utils;

#[pymodule]
fn pyfastgrep(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(regex_bindings::search, m)?)?;
    m.add_function(wrap_pyfunction!(regex_bindings::search_iter, m)?)?;
    m.add_function(wrap_pyfunction!(regex_bindings::search_count, m)?)?;
    m.add_function(wrap_pyfunction!(regex_bindings::search_files_with_matches, m)?)?;
    m.add_function(wrap_pyfunction!(context_bindings::search_with_context, m)?)?;
    m.add_function(wrap_pyfunction!(ast_bindings::search_functions, m)?)?;
    m.add_function(wrap_pyfunction!(ast_bindings::search_classes, m)?)?;
    m.add_function(wrap_pyfunction!(ast_bindings::search_imports, m)?)?;
    m.add_function(wrap_pyfunction!(ast_bindings::search_functions_iter, m)?)?;
    m.add_function(wrap_pyfunction!(ast_bindings::search_classes_iter, m)?)?;
    m.add_function(wrap_pyfunction!(ast_bindings::search_imports_iter, m)?)?;
    Ok(())
}
