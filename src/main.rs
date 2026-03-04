use clap::Parser;
use mille::domain::entity::violation::Severity;
use mille::infrastructure::parser::rust::RustParser;
use mille::infrastructure::repository::fs_source_file_repository::FsSourceFileRepository;
use mille::infrastructure::repository::toml_config_repository::TomlConfigRepository;
use mille::infrastructure::resolver::rust::RustResolver;
use mille::presentation::cli::args::{Cli, Command};
use mille::presentation::formatter::terminal::{
    format_layer_stats, format_summary, format_violation,
};
use mille::usecase::check_architecture;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { config } => {
            match check_architecture::check(
                &config,
                &TomlConfigRepository,
                &FsSourceFileRepository,
                &RustParser,
                &RustResolver,
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
