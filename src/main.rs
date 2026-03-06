use std::collections::HashMap;

use clap::Parser;
use mille::domain::entity::config::MilleConfig;
use mille::domain::entity::violation::Severity;
use mille::domain::repository::config_repository::ConfigRepository;
use mille::infrastructure::parser::DispatchingParser;
use mille::infrastructure::repository::fs_source_file_repository::FsSourceFileRepository;
use mille::infrastructure::repository::toml_config_repository::TomlConfigRepository;
use mille::infrastructure::resolver::go::GoResolver;
use mille::infrastructure::resolver::python::PythonResolver;
use mille::infrastructure::resolver::typescript::TypeScriptResolver;
use mille::infrastructure::resolver::DispatchingResolver;
use mille::presentation::cli::args::{Cli, Command};
use mille::presentation::formatter::terminal::{
    format_layer_stats, format_summary, format_violation,
};
use mille::usecase::check_architecture;

/// Load TypeScript path aliases from the tsconfig.json referenced in mille.toml.
///
/// Reads `resolve.typescript.tsconfig`, parses `compilerOptions.paths`, and
/// returns a flat map of pattern → first target.  Returns an empty map if the
/// field is absent, the file is missing, or it has no paths entries.
fn load_ts_aliases(config_path: &str, app_config: &MilleConfig) -> HashMap<String, String> {
    let tsconfig_rel = match app_config
        .resolve
        .as_ref()
        .and_then(|r| r.typescript.as_ref())
        .map(|t| t.tsconfig.as_str())
    {
        Some(p) => p.to_string(),
        None => return HashMap::new(),
    };

    // Resolve tsconfig path relative to the directory of mille.toml.
    let config_dir = std::path::Path::new(config_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let tsconfig_path = config_dir.join(&tsconfig_rel);

    let content = match std::fs::read_to_string(&tsconfig_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };

    // NOTE: serde_json cannot parse tsconfig files that contain comments (//),
    //       but the tsconfig.json produced by tsc --init includes comments.
    //       We strip single-line comments before parsing as a best-effort.
    let stripped = strip_json_line_comments(&content);

    let value: serde_json::Value = match serde_json::from_str(&stripped) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let paths = match value
        .get("compilerOptions")
        .and_then(|c| c.get("paths"))
        .and_then(|p| p.as_object())
    {
        Some(p) => p,
        None => return HashMap::new(),
    };

    let mut aliases = HashMap::new();
    for (pattern, targets) in paths {
        if let Some(first) = targets.as_array().and_then(|a| a.first()) {
            if let Some(target) = first.as_str() {
                aliases.insert(pattern.clone(), target.to_string());
            }
        }
    }
    aliases
}

/// Strip `//` single-line comments from a JSON string (best-effort for tsconfig files).
fn strip_json_line_comments(s: &str) -> String {
    s.lines()
        .map(|line| {
            // Only strip if `//` appears outside a string value.
            // Simple heuristic: find the first `//` not inside a quoted segment.
            let mut in_string = false;
            let mut escaped = false;
            let bytes = line.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if escaped {
                    escaped = false;
                } else if in_string {
                    if b == b'\\' {
                        escaped = true;
                    } else if b == b'"' {
                        in_string = false;
                    }
                } else if b == b'"' {
                    in_string = true;
                } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    return &line[..i];
                }
                i += 1;
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { config } => {
            // Pre-load config to extract the Go module name for GoResolver.
            // NOTE: Double-load is acceptable for a CLI tool — the first load
            // extracts the module_name to construct GoResolver, the second
            // load happens inside check_architecture::check().
            let config_repo = TomlConfigRepository;
            let app_config = match config_repo.load(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(3);
                }
            };
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
            let ts_aliases = load_ts_aliases(&config, &app_config);

            let parser = DispatchingParser::new();
            let resolver = DispatchingResolver::new(
                GoResolver::new(go_module),
                PythonResolver::new(python_packages),
                TypeScriptResolver::with_aliases(ts_aliases),
            );

            match check_architecture::check(
                &config,
                &config_repo,
                &FsSourceFileRepository,
                &parser,
                &resolver,
            ) {
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
    }
}
