use pyo3::prelude::*;

use mille_core::{
    domain::{
        entity::violation::Severity,
        repository::config_repository::ConfigRepository,
    },
    infrastructure::{
        parser::DispatchingParser,
        repository::{
            fs_source_file_repository::FsSourceFileRepository,
            toml_config_repository::TomlConfigRepository,
        },
        resolver::{
            go::GoResolver,
            python::PythonResolver,
            typescript::TypeScriptResolver,
            DispatchingResolver,
        },
    },
    presentation::formatter::terminal::{format_layer_stats, format_summary, format_violation},
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

    let go_module = app_config
        .resolve
        .as_ref()
        .and_then(|r| r.go.as_ref())
        .map(|g| g.module_name.clone())
        .unwrap_or_default();
    let python_packages = app_config
        .resolve
        .as_ref()
        .and_then(|r| r.python.as_ref())
        .map(|p| p.package_names.clone())
        .unwrap_or_default();

    let parser = DispatchingParser::new();
    let resolver = DispatchingResolver::new(
        GoResolver::new(go_module),
        PythonResolver::new(python_packages),
        TypeScriptResolver::new(),
    );

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
/// Reads sys.argv, runs `mille check [--config <path>]`, prints results,
/// and exits with the appropriate code (0 / 1 / 3).
#[pyfunction]
fn _main() {
    let args: Vec<String> = std::env::args().collect();

    // Find --config <path> anywhere in the args (after optional "check")
    let config_path = args
        .windows(2)
        .find(|w| w[0] == "--config" || w[0] == "-c")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "mille.toml".to_string());

    match wire_and_check(&config_path) {
        Ok(result) => {
            for v in &result.violations {
                print!("{}", format_violation(v));
            }
            print!("{}", format_layer_stats(&result.layer_stats));
            print!("{}", format_summary(&result.violations));

            let has_error = result
                .violations
                .iter()
                .any(|v| v.severity == Severity::Error);

            if has_error {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(3);
        }
    }
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
