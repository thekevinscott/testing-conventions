//! pyo3 binding — thin glue only; all logic lives in agent-context-core.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass(frozen)]
struct PrintResult {
    #[pyo3(get)]
    files: Vec<String>,
    #[pyo3(get)]
    text: String,
}

#[pyfunction]
#[pyo3(name = "print", signature = (globs, cwd=None))]
fn print_py(globs: Vec<String>, cwd: Option<String>) -> PyResult<PrintResult> {
    let r = agent_context_core::print(&globs, cwd.as_deref()).map_err(PyValueError::new_err)?;
    Ok(PrintResult {
        files: r.files,
        text: r.text,
    })
}

#[pyfunction]
fn write_block(path: String, id: String, content: String) -> PyResult<bool> {
    agent_context_core::write_block(&path, &id, &content).map_err(PyValueError::new_err)
}

#[pyfunction]
fn crash(mode: String) -> PyResult<()> {
    agent_context_core::crash(&mode).map_err(PyValueError::new_err)
}

#[pyfunction]
fn run(argv: Vec<String>) -> i32 {
    agent_context_core::run(argv)
}

/// Console-script entry point: forward sys.argv[1:] to the single-sourced CLI.
#[pyfunction]
fn main(py: Python<'_>) -> PyResult<()> {
    let argv: Vec<String> = py.import("sys")?.getattr("argv")?.extract()?;
    std::process::exit(agent_context_core::run(argv.into_iter().skip(1)));
}

#[pymodule]
fn agent_context(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PrintResult>()?;
    m.add_function(wrap_pyfunction!(print_py, m)?)?;
    m.add_function(wrap_pyfunction!(write_block, m)?)?;
    m.add_function(wrap_pyfunction!(crash, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(main, m)?)?;
    Ok(())
}
