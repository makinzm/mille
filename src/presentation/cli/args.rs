use clap::{Parser, Subcommand};

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
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_check_uses_default_config() {
        let cli = Cli::try_parse_from(["mille", "check"]).unwrap();
        match cli.command {
            Command::Check { config } => assert_eq!(config, "mille.toml"),
        }
    }

    #[test]
    fn test_parse_check_with_custom_config() {
        let cli = Cli::try_parse_from(["mille", "check", "--config", "custom.toml"]).unwrap();
        match cli.command {
            Command::Check { config } => assert_eq!(config, "custom.toml"),
        }
    }

    #[test]
    fn test_parse_unknown_subcommand_returns_error() {
        assert!(Cli::try_parse_from(["mille", "unknown"]).is_err());
    }
}
