pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod usecase;

use clap::Parser;
use domain::entity::violation::Severity;
use presentation::cli::args::{Cli, Command};
use presentation::formatter::terminal::{format_layer_stats, format_summary, format_violation};
use usecase::check_architecture;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Check { config } => match check_architecture::check(&config) {
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
        },
    }
}
