use clap::{Parser, Subcommand, ValueEnum};

/// Threshold for exit code 1 in `mille check`.
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum FailOn {
    /// Exit 1 only when there are error-severity violations (default).
    Error,
    /// Exit 1 when there are any violations (error or warning).
    Warning,
}

/// Output format for `mille check`.
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum Format {
    /// Human-readable terminal output (default)
    Terminal,
    /// JSON output
    Json,
    /// GitHub Actions annotation format (`::error file=...,line=N::msg`)
    GithubActions,
}

/// Output format for `mille analyze`.
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum AnalyzeFormat {
    /// Human-readable terminal output (default)
    Terminal,
    /// JSON graph data
    Json,
    /// Graphviz DOT format
    Dot,
    /// Self-contained SVG image
    Svg,
}

#[derive(Parser, Debug)]
#[command(
    name = "mille",
    version,
    about = "Architecture Checker — multi-language architecture linter"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Check architecture dependency rules against source files.
    Check {
        /// Path to mille.toml (default: ./mille.toml)
        #[arg(long, default_value = "mille.toml")]
        config: String,
        /// Output format: terminal (default), json, github-actions
        #[arg(long, value_enum, default_value_t = Format::Terminal)]
        format: Format,
        /// Exit with code 1 when violations at this severity or above are found.
        /// Default: exit 1 only on errors. Use --fail-on warning to also fail on warnings.
        #[arg(long, value_enum)]
        fail_on: Option<FailOn>,
    },
    /// Visualize the dependency graph without applying rules.
    Analyze {
        /// Path to mille.toml (default: ./mille.toml)
        #[arg(long, default_value = "mille.toml")]
        config: String,
        /// Output format: terminal (default), json, dot, svg
        #[arg(long, value_enum, default_value_t = AnalyzeFormat::Terminal)]
        format: AnalyzeFormat,
        /// Write output to this file instead of stdout. Refuses to overwrite existing files.
        #[arg(long)]
        output: Option<String>,
    },
    /// Scan the project and generate a mille.toml configuration file.
    Init {
        /// Output path for the generated config (default: mille.toml)
        #[arg(long, default_value = "mille.toml")]
        output: String,
        /// Overwrite existing file without prompting
        #[arg(long, default_value_t = false)]
        force: bool,
        /// Layer detection depth from project root (auto-detected if not set).
        /// Example: --depth 2 for src/domain, src/usecase structure.
        #[arg(long)]
        depth: Option<usize>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn test_parse_check_uses_default_config() {
        let cli = Cli::try_parse_from(["mille", "check"]).unwrap();
        match cli.command {
            Command::Check { config, .. } => assert_eq!(config, "mille.toml"),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_check_with_custom_config() {
        let cli = Cli::try_parse_from(["mille", "check", "--config", "custom.toml"]).unwrap();
        match cli.command {
            Command::Check { config, .. } => assert_eq!(config, "custom.toml"),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_unknown_subcommand_returns_error() {
        assert!(Cli::try_parse_from(["mille", "unknown"]).is_err());
    }

    #[test]
    fn test_parse_format_defaults_to_terminal() {
        let cli = Cli::try_parse_from(["mille", "check"]).unwrap();
        match cli.command {
            Command::Check { format, .. } => assert_eq!(format, Format::Terminal),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_format_github_actions() {
        let cli = Cli::try_parse_from(["mille", "check", "--format", "github-actions"]).unwrap();
        match cli.command {
            Command::Check { format, .. } => assert_eq!(format, Format::GithubActions),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_format_json() {
        let cli = Cli::try_parse_from(["mille", "check", "--format", "json"]).unwrap();
        match cli.command {
            Command::Check { format, .. } => assert_eq!(format, Format::Json),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_help_subcommand_displays_help() {
        let err = Cli::try_parse_from(["mille", "help"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);

        let msg = err.to_string();
        assert!(msg.contains("Usage: mille <COMMAND>"));
        assert!(msg.contains("Commands:"));
        assert!(msg.contains("help"));
    }

    #[test]
    fn test_parse_help_for_check_displays_subcommand_help() {
        let err = Cli::try_parse_from(["mille", "help", "check"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);

        let msg = err.to_string();
        assert!(msg.contains("Usage: mille check"));
        assert!(msg.contains("--config"));
        assert!(msg.contains("--format"));
    }

    #[test]
    fn test_parse_dashdash_help_displays_help() {
        let err = Cli::try_parse_from(["mille", "--help"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_parse_init_default_output() {
        let cli = Cli::try_parse_from(["mille", "init"]).unwrap();
        match cli.command {
            Command::Init { output, force, .. } => {
                assert_eq!(output, "mille.toml");
                assert!(!force);
            }
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_parse_init_custom_output() {
        let cli = Cli::try_parse_from(["mille", "init", "--output", "custom.toml"]).unwrap();
        match cli.command {
            Command::Init { output, .. } => assert_eq!(output, "custom.toml"),
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_parse_init_force_flag() {
        let cli = Cli::try_parse_from(["mille", "init", "--force"]).unwrap();
        match cli.command {
            Command::Init { force, .. } => assert!(force),
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_parse_init_depth_flag() {
        let cli = Cli::try_parse_from(["mille", "init", "--depth", "2"]).unwrap();
        match cli.command {
            Command::Init { depth, .. } => assert_eq!(depth, Some(2)),
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_parse_init_depth_defaults_to_none() {
        let cli = Cli::try_parse_from(["mille", "init"]).unwrap();
        match cli.command {
            Command::Init { depth, .. } => assert_eq!(depth, None),
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_parse_fail_on_warning() {
        let cli = Cli::try_parse_from(["mille", "check", "--fail-on", "warning"]).unwrap();
        match cli.command {
            Command::Check { fail_on, .. } => assert_eq!(fail_on, Some(FailOn::Warning)),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_fail_on_error() {
        let cli = Cli::try_parse_from(["mille", "check", "--fail-on", "error"]).unwrap();
        match cli.command {
            Command::Check { fail_on, .. } => assert_eq!(fail_on, Some(FailOn::Error)),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_fail_on_defaults_to_none() {
        let cli = Cli::try_parse_from(["mille", "check"]).unwrap();
        match cli.command {
            Command::Check { fail_on, .. } => assert_eq!(fail_on, None),
            _ => panic!("expected Check command"),
        }
    }
}
