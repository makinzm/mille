//! End-to-end tests for `mille check --format <fmt>`.
//!
//! Verifies that `--format github-actions` and `--format json` produce the
//! expected output shapes, and that exit codes are consistent with the
//! default terminal formatter.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

// ---------------------------------------------------------------------------
// Helpers (same pattern as e2e_check.rs)
// ---------------------------------------------------------------------------

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn mille(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(project_root())
        .output()
        .expect("failed to execute mille binary")
}

struct TempConfig {
    path: PathBuf,
}

impl TempConfig {
    fn new(name: &str, content: &str) -> Self {
        let path = project_root().join(name);
        fs::write(&path, content).expect("failed to write temp config");
        TempConfig { path }
    }

    fn file_name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn exit_code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}

// ---------------------------------------------------------------------------
// Config fixtures
// ---------------------------------------------------------------------------

const VALID_CONFIG: &str = "mille.toml";

/// infrastructure is opt-in with allow=[] → violations guaranteed.
const INFRA_BLOCKS_DOMAIN_TOML: &str = r#"
[project]
name = "mille-e2e"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-in"
external_allow = ["serde", "toml", "tree_sitter", "glob"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-in"
allow = ["usecase", "domain"]
external_mode = "opt-in"
external_allow = ["clap"]

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-in"
allow = ["domain", "infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["clap"]
"#;

// ---------------------------------------------------------------------------
// --format github-actions
// ---------------------------------------------------------------------------

#[test]
fn test_ga_format_violation_uses_annotation_syntax() {
    let cfg = TempConfig::new("mille_e2e_fmt_ga_violation.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&[
        "check",
        "--config",
        cfg.file_name(),
        "--format",
        "github-actions",
    ]);
    let s = stdout(&out);
    assert!(
        s.contains("::error "),
        "github-actions format must use ::error annotation\nstdout:\n{s}"
    );
}

#[test]
fn test_ga_format_violation_contains_file_and_line() {
    let cfg = TempConfig::new("mille_e2e_fmt_ga_file.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&[
        "check",
        "--config",
        cfg.file_name(),
        "--format",
        "github-actions",
    ]);
    let s = stdout(&out);
    assert!(
        s.contains("file="),
        "annotation must include file= key\nstdout:\n{s}"
    );
    assert!(
        s.contains("line="),
        "annotation must include line= key\nstdout:\n{s}"
    );
}

#[test]
fn test_ga_format_violation_exits_one() {
    let cfg = TempConfig::new("mille_e2e_fmt_ga_exit.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&[
        "check",
        "--config",
        cfg.file_name(),
        "--format",
        "github-actions",
    ]);
    assert_eq!(
        exit_code(&out),
        1,
        "github-actions format must still exit 1 when violations exist\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_ga_format_no_violation_output_is_empty() {
    let out = mille(&[
        "check",
        "--config",
        VALID_CONFIG,
        "--format",
        "github-actions",
    ]);
    let s = stdout(&out);
    assert!(
        s.is_empty(),
        "github-actions format must produce empty output when there are no violations\nstdout:\n{s}"
    );
}

#[test]
fn test_ga_format_no_violation_exits_zero() {
    let out = mille(&[
        "check",
        "--config",
        VALID_CONFIG,
        "--format",
        "github-actions",
    ]);
    assert_eq!(
        exit_code(&out),
        0,
        "github-actions format must exit 0 when there are no violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_ga_format_no_terminal_artifacts() {
    // Terminal-specific output (✅, ❌, Summary:) must NOT appear in github-actions format.
    let cfg = TempConfig::new("mille_e2e_fmt_ga_noterm.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&[
        "check",
        "--config",
        cfg.file_name(),
        "--format",
        "github-actions",
    ]);
    let s = stdout(&out);
    assert!(
        !s.contains("Summary:"),
        "github-actions output must not contain 'Summary:'\nstdout:\n{s}"
    );
    assert!(
        !s.contains('✅'),
        "github-actions output must not contain ✅\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// --format json
// ---------------------------------------------------------------------------

#[test]
fn test_json_format_violation_is_valid_json_shape() {
    let cfg = TempConfig::new("mille_e2e_fmt_json_shape.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&["check", "--config", cfg.file_name(), "--format", "json"]);
    let s = stdout(&out);
    assert!(
        s.trim().starts_with('{'),
        "json output must start with '{{\nstdout:\n{s}"
    );
    assert!(
        s.trim().ends_with('}'),
        "json output must end with '}}\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"summary\""),
        "json output must contain 'summary' key\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"violations\""),
        "json output must contain 'violations' key\nstdout:\n{s}"
    );
}

#[test]
fn test_json_format_violation_exits_one() {
    let cfg = TempConfig::new("mille_e2e_fmt_json_exit.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&["check", "--config", cfg.file_name(), "--format", "json"]);
    assert_eq!(
        exit_code(&out),
        1,
        "json format must still exit 1 when violations exist\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_json_format_no_violation_errors_zero() {
    let out = mille(&["check", "--config", VALID_CONFIG, "--format", "json"]);
    let s = stdout(&out);
    assert!(
        s.contains("\"errors\":0"),
        "json output must show errors:0 when clean\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"warnings\":0"),
        "json output must show warnings:0 when clean\nstdout:\n{s}"
    );
}

#[test]
fn test_json_format_no_violation_exits_zero() {
    let out = mille(&["check", "--config", VALID_CONFIG, "--format", "json"]);
    assert_eq!(
        exit_code(&out),
        0,
        "json format must exit 0 when there are no violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_json_format_violation_has_nonzero_errors() {
    let cfg = TempConfig::new("mille_e2e_fmt_json_errcnt.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&["check", "--config", cfg.file_name(), "--format", "json"]);
    let s = stdout(&out);
    assert!(
        !s.contains("\"errors\":0"),
        "json errors count must be > 0 when violations exist\nstdout:\n{s}"
    );
}

#[test]
fn test_json_format_no_terminal_artifacts() {
    let cfg = TempConfig::new("mille_e2e_fmt_json_noterm.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&["check", "--config", cfg.file_name(), "--format", "json"]);
    let s = stdout(&out);
    assert!(
        !s.contains("Summary:"),
        "json output must not contain 'Summary:'\nstdout:\n{s}"
    );
    assert!(
        !s.contains("::error"),
        "json output must not contain GitHub Actions annotations\nstdout:\n{s}"
    );
}
