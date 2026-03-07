use clap::{Parser, Subcommand, ValueEnum};

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
        }
    }

    #[test]
    fn test_parse_check_with_custom_config() {
        let cli = Cli::try_parse_from(["mille", "check", "--config", "custom.toml"]).unwrap();
        match cli.command {
            Command::Check { config, .. } => assert_eq!(config, "custom.toml"),
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
        }
    }

    #[test]
    fn test_parse_format_github_actions() {
        let cli = Cli::try_parse_from(["mille", "check", "--format", "github-actions"]).unwrap();
        match cli.command {
            Command::Check { format, .. } => assert_eq!(format, Format::GithubActions),
        }
    }

    #[test]
    fn test_parse_format_json() {
        let cli = Cli::try_parse_from(["mille", "check", "--format", "json"]).unwrap();
        match cli.command {
            Command::Check { format, .. } => assert_eq!(format, Format::Json),
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
}
