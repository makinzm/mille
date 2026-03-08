use pyo3::prelude::*;

/// CLI entry point — called by the `mille` script installed by pip.
///
/// Delegates to [`mille_core::runner::run_cli_from`] using Python's `sys.argv`
/// instead of `std::env::args()`.  When the OS runs a pip entry-point script
/// via its shebang (e.g. `python3 /path/.venv/bin/mille check`), Rust's
/// `std::env::args()` sees `["python3", "/path/.venv/bin/mille", "check"]`
/// and clap misinterprets the script path as a subcommand.  Python's
/// `sys.argv` is already set to `["/path/.venv/bin/mille", "check"]` by the
/// interpreter, so passing it to clap gives the correct parse result.
#[pyfunction]
fn _main(py: Python<'_>) {
    let argv: Vec<String> = py
        .import_bound("sys")
        .expect("failed to import sys")
        .getattr("argv")
        .expect("failed to get sys.argv")
        .extract()
        .expect("failed to extract sys.argv as Vec<String>");
    mille_core::runner::run_cli_from(argv);
}

#[pymodule]
fn mille(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_main, m)?)?;
    Ok(())
}
