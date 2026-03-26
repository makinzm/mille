use clap::{Args, Parser, Subcommand, ValueEnum};

/// Output format for `mille report external`.
#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ReportExternalFormat {
    /// Human-readable terminal output (default)
    Terminal,
    /// JSON output
    Json,
}

/// Subcommands under `mille report`.
#[derive(Subcommand, Debug)]
pub enum ReportCommand {
    /// Show external library dependencies for each layer.
    External {
        #[command(flatten)]
        common: CommonArgs,
        /// Path to mille.toml (default: ./mille.toml)
        #[arg(long, default_value = "mille.toml")]
        config: String,
        /// Output format: terminal (default), json
        #[arg(long, value_enum, default_value_t = ReportExternalFormat::Terminal)]
        format: ReportExternalFormat,
        /// Write output to this file instead of stdout. Refuses to overwrite existing files.
        #[arg(long)]
        output: Option<String>,
    },
}

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

/// Common arguments shared by all subcommands.
///
/// Every new subcommand **must** include `#[command(flatten)] common: CommonArgs`.
/// `Command::common()` enforces this at compile time via exhaustive match.
#[derive(Args, Debug, Clone)]
pub struct CommonArgs {
    /// Project directory to check (default: current directory).
    #[arg(default_value = ".")]
    pub path: String,
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
        #[command(flatten)]
        common: CommonArgs,
        /// Path to mille.toml (default: ./mille.toml)
        #[arg(long, default_value = "mille.toml")]
        config: String,
        /// Output format: terminal (default), json, github-actions
        #[arg(long, value_enum, default_value_t = Format::Terminal)]
        format: Format,
        /// Minimum severity that causes exit code 1. Defaults to "error".
        /// Use --fail-on warning to also fail on warnings.
        #[arg(long, value_enum, default_value_t = FailOn::Error)]
        fail_on: FailOn,
    },
    /// Visualize the dependency graph without applying rules.
    Analyze {
        #[command(flatten)]
        common: CommonArgs,
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
    /// Report external library dependencies by layer.
    Report {
        #[command(subcommand)]
        subcommand: ReportCommand,
    },
    /// Scan the project and generate a mille.toml configuration file.
    Init {
        #[command(flatten)]
        common: CommonArgs,
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
    /// Add a directory as a new layer to an existing mille.toml.
    Add {
        #[command(flatten)]
        common: CommonArgs,
        /// Path to mille.toml (default: ./mille.toml)
        #[arg(long, default_value = "mille.toml")]
        config: String,
        /// Layer name (default: directory basename)
        #[arg(long)]
        name: Option<String>,
        /// Overwrite existing layer with overlapping paths without prompting
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}

impl Command {
    /// Returns the common arguments shared by all subcommands.
    ///
    /// Adding a new variant to `Command` without a `common: CommonArgs` field
    /// will cause a compile error here — this is intentional.
    pub fn common(&self) -> &CommonArgs {
        match self {
            Command::Check { common, .. } => common,
            Command::Analyze { common, .. } => common,
            Command::Report { subcommand } => subcommand.common(),
            Command::Init { common, .. } => common,
            Command::Add { common, .. } => common,
        }
    }
}

impl ReportCommand {
    /// Returns the common arguments for report subcommands.
    pub fn common(&self) -> &CommonArgs {
        match self {
            ReportCommand::External { common, .. } => common,
        }
    }
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
            Command::Check { fail_on, .. } => assert_eq!(fail_on, FailOn::Warning),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_fail_on_error() {
        let cli = Cli::try_parse_from(["mille", "check", "--fail-on", "error"]).unwrap();
        match cli.command {
            Command::Check { fail_on, .. } => assert_eq!(fail_on, FailOn::Error),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_fail_on_defaults_to_error() {
        let cli = Cli::try_parse_from(["mille", "check"]).unwrap();
        match cli.command {
            Command::Check { fail_on, .. } => assert_eq!(fail_on, FailOn::Error),
            _ => panic!("expected Check command"),
        }
    }

    // ---------------------------------------------------------------
    // PATH positional argument tests
    // ---------------------------------------------------------------

    #[test]
    fn test_parse_check_default_path() {
        let cli = Cli::try_parse_from(["mille", "check"]).unwrap();
        assert_eq!(cli.command.common().path, ".");
    }

    #[test]
    fn test_parse_check_custom_path() {
        let cli = Cli::try_parse_from(["mille", "check", "./foo"]).unwrap();
        assert_eq!(cli.command.common().path, "./foo");
    }

    #[test]
    fn test_parse_check_path_with_config() {
        let cli =
            Cli::try_parse_from(["mille", "check", "./foo", "--config", "custom.toml"]).unwrap();
        assert_eq!(cli.command.common().path, "./foo");
        match cli.command {
            Command::Check { config, .. } => assert_eq!(config, "custom.toml"),
            _ => panic!("expected Check command"),
        }
    }

    #[test]
    fn test_parse_analyze_default_path() {
        let cli = Cli::try_parse_from(["mille", "analyze"]).unwrap();
        assert_eq!(cli.command.common().path, ".");
    }

    #[test]
    fn test_parse_analyze_custom_path() {
        let cli = Cli::try_parse_from(["mille", "analyze", "./bar"]).unwrap();
        assert_eq!(cli.command.common().path, "./bar");
    }

    #[test]
    fn test_parse_report_external_default_path() {
        let cli = Cli::try_parse_from(["mille", "report", "external"]).unwrap();
        match &cli.command {
            Command::Report { subcommand } => match subcommand {
                ReportCommand::External { common, .. } => assert_eq!(common.path, "."),
            },
            _ => panic!("expected Report command"),
        }
    }

    #[test]
    fn test_parse_report_external_custom_path() {
        let cli = Cli::try_parse_from(["mille", "report", "external", "./baz"]).unwrap();
        match &cli.command {
            Command::Report { subcommand } => match subcommand {
                ReportCommand::External { common, .. } => assert_eq!(common.path, "./baz"),
            },
            _ => panic!("expected Report command"),
        }
    }

    #[test]
    fn test_parse_init_default_path() {
        let cli = Cli::try_parse_from(["mille", "init"]).unwrap();
        assert_eq!(cli.command.common().path, ".");
    }

    #[test]
    fn test_parse_init_custom_path() {
        let cli = Cli::try_parse_from(["mille", "init", "./qux"]).unwrap();
        assert_eq!(cli.command.common().path, "./qux");
    }

    // ---------------------------------------------------------------
    // ADD subcommand tests
    // ---------------------------------------------------------------

    #[test]
    fn test_parse_add_basic() {
        let cli = Cli::try_parse_from(["mille", "add", "src/newlayer"]).unwrap();
        match &cli.command {
            Command::Add {
                common,
                config,
                name,
                force,
            } => {
                assert_eq!(common.path, "src/newlayer");
                assert_eq!(config, "mille.toml");
                assert!(name.is_none());
                assert!(!force);
            }
            _ => panic!("expected Add command"),
        }
    }

    #[test]
    fn test_parse_add_with_config() {
        let cli = Cli::try_parse_from(["mille", "add", "src/newlayer", "--config", "custom.toml"])
            .unwrap();
        match &cli.command {
            Command::Add { config, .. } => assert_eq!(config, "custom.toml"),
            _ => panic!("expected Add command"),
        }
    }

    #[test]
    fn test_parse_add_with_name() {
        let cli =
            Cli::try_parse_from(["mille", "add", "src/newlayer", "--name", "my_layer"]).unwrap();
        match &cli.command {
            Command::Add { name, .. } => assert_eq!(name.as_deref(), Some("my_layer")),
            _ => panic!("expected Add command"),
        }
    }

    #[test]
    fn test_parse_add_with_force() {
        let cli = Cli::try_parse_from(["mille", "add", "src/newlayer", "--force"]).unwrap();
        match &cli.command {
            Command::Add { force, .. } => assert!(*force),
            _ => panic!("expected Add command"),
        }
    }

    #[test]
    fn test_parse_add_default_target() {
        let cli = Cli::try_parse_from(["mille", "add"]).unwrap();
        assert_eq!(cli.command.common().path, ".");
    }
}
