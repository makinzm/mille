use pyo3::prelude::*;

use mille_core::{
    domain::repository::config_repository::ConfigRepository,
    infrastructure::{
        parser::DispatchingParser,
        repository::{
            fs_source_file_repository::FsSourceFileRepository,
            toml_config_repository::TomlConfigRepository,
        },
        resolver::DispatchingResolver,
    },
    usecase::check_architecture,
};

// ---------------------------------------------------------------------------
// Exposed Python types
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Clone)]
pub struct Violation {
    #[pyo3(get)]
    pub file: String,
    #[pyo3(get)]
    pub line: usize,
    #[pyo3(get)]
    pub from_layer: String,
    #[pyo3(get)]
    pub to_layer: String,
    #[pyo3(get)]
    pub import_path: String,
    #[pyo3(get)]
    pub kind: String,
}

#[pyclass]
#[derive(Clone)]
pub struct LayerStat {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub file_count: usize,
    #[pyo3(get)]
    pub violation_count: usize,
}

#[pyclass]
pub struct CheckResult {
    #[pyo3(get)]
    pub violations: Vec<Violation>,
    #[pyo3(get)]
    pub layer_stats: Vec<LayerStat>,
}

// ---------------------------------------------------------------------------
// Internal helpers (not exposed to Python)
// ---------------------------------------------------------------------------

fn wire_and_check(config_path: &str) -> Result<check_architecture::CheckResult, String> {
    let config_repo = TomlConfigRepository;
    let app_config = config_repo
        .load(config_path)
        .map_err(|e| e.to_string())?;

    let parser = DispatchingParser::new();
    let resolver = DispatchingResolver::from_config(&app_config, config_path);

    check_architecture::check(config_path, &config_repo, &FsSourceFileRepository, &parser, &resolver)
}

// ---------------------------------------------------------------------------
// Public Python functions
// ---------------------------------------------------------------------------

/// Run an architecture check against `config_path` and return the result.
///
/// Raises `RuntimeError` if the config file cannot be loaded.
#[pyfunction]
#[pyo3(signature = (config_path = "mille.toml"))]
fn check(config_path: &str) -> PyResult<CheckResult> {
    let result = wire_and_check(config_path)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))?;

    Ok(CheckResult {
        violations: result
            .violations
            .into_iter()
            .map(|v| Violation {
                file: v.file,
                line: v.line,
                from_layer: v.from_layer,
                to_layer: v.to_layer,
                import_path: v.import_path,
                kind: format!("{:?}", v.kind),
            })
            .collect(),
        layer_stats: result
            .layer_stats
            .into_iter()
            .map(|s| LayerStat {
                name: s.name,
                file_count: s.file_count,
                violation_count: s.violation_count,
            })
            .collect(),
    })
}

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

// ---------------------------------------------------------------------------
// Module definition
// ---------------------------------------------------------------------------

#[pymodule]
fn mille(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Violation>()?;
    m.add_class::<LayerStat>()?;
    m.add_class::<CheckResult>()?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    m.add_function(wrap_pyfunction!(_main, m)?)?;
    Ok(())
}
