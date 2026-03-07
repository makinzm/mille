use clap::Parser;
use mille::domain::entity::violation::Severity;
use mille::domain::repository::config_repository::ConfigRepository;
use mille::infrastructure::parser::DispatchingParser;
use mille::infrastructure::repository::fs_source_file_repository::FsSourceFileRepository;
use mille::infrastructure::repository::toml_config_repository::TomlConfigRepository;
use mille::infrastructure::resolver::DispatchingResolver;
use mille::presentation::cli::args::Format;
use mille::presentation::cli::args::{Cli, Command};
use mille::presentation::formatter::github_actions::format_all_ga;
use mille::presentation::formatter::json::format_json;
use mille::presentation::formatter::terminal::{
    format_layer_stats, format_summary, format_violation,
};
use mille::usecase::check_architecture;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { config, format } => {
            // Pre-load config to build the resolver, then pass path to check().
            // NOTE: Double-load is acceptable for a CLI tool.
            let config_repo = TomlConfigRepository;
            let app_config = match config_repo.load(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(3);
                }
            };

            let parser = DispatchingParser::new();
            let resolver = DispatchingResolver::from_config(&app_config, &config);

            match check_architecture::check(
                &config,
                &config_repo,
                &FsSourceFileRepository,
                &parser,
                &resolver,
            ) {
                Ok(result) => {
                    match format {
                        Format::Terminal => {
                            for v in &result.violations {
                                print!("{}", format_violation(v));
                            }
                            print!("{}", format_layer_stats(&result.layer_stats));
                            print!("{}", format_summary(&result.violations));
                        }
                        Format::GithubActions => {
                            print!("{}", format_all_ga(&result.violations));
                        }
                        Format::Json => {
                            print!("{}", format_json(&result.violations));
                        }
                    }

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
