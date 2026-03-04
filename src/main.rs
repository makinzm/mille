use clap::Parser;
use mille::domain::entity::violation::Severity;
use mille::domain::repository::config_repository::ConfigRepository;
use mille::infrastructure::parser::DispatchingParser;
use mille::infrastructure::repository::fs_source_file_repository::FsSourceFileRepository;
use mille::infrastructure::repository::toml_config_repository::TomlConfigRepository;
use mille::infrastructure::resolver::go::GoResolver;
use mille::infrastructure::resolver::DispatchingResolver;
use mille::presentation::cli::args::{Cli, Command};
use mille::presentation::formatter::terminal::{
    format_layer_stats, format_summary, format_violation,
};
use mille::usecase::check_architecture;

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

            let parser = DispatchingParser::new();
            let resolver = DispatchingResolver::new(GoResolver::new(go_module));

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
