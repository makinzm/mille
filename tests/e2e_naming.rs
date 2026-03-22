//! End-to-end tests for naming convention check (`name_deny` / `name_targets`).
//!
//! These tests invoke the compiled binary directly and verify:
//!   - exit codes (0 = clean, 1 = violations found)
//!   - stdout content (violation message contains matched keyword and target kind)
//!
//! Fixture design principle:
//!   - Only `usecase` layer has `name_deny = ["aws"]`.
//!   - `domain` layer has no `name_deny` → aws references in domain are allowed.
//!   - Both layers use `dependency_mode = "opt-out"` and `external_mode = "opt-out"`
//!     to prevent false positives from dependency/external checks.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

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
// Base config builder (all layers opt-out, usecase has name_deny)
// ---------------------------------------------------------------------------

fn naming_config(name_deny: &[&str], name_targets: Option<&[&str]>) -> String {
    let deny_list = name_deny
        .iter()
        .map(|k| format!("\"{}\"", k))
        .collect::<Vec<_>>()
        .join(", ");

    let targets_line = if let Some(targets) = name_targets {
        let t = targets
            .iter()
            .map(|k| format!("\"{}\"", k))
            .collect::<Vec<_>>()
            .join(", ");
        format!("name_targets = [{}]\n", t)
    } else {
        String::new()
    };

    format!(
        r#"
[project]
name = "mille-e2e-naming"
root = "."
languages = ["rust"]

[[layers]]
name = "usecase"
paths = ["tests/fixtures/naming/usecase/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
name_deny = [{deny_list}]
{targets_line}
[[layers]]
name = "domain"
paths = ["tests/fixtures/naming/domain/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#,
        deny_list = deny_list,
        targets_line = targets_line,
    )
}

fn naming_config_with_severity(name_deny: &[&str], severity: &str) -> String {
    let deny_list = name_deny
        .iter()
        .map(|k| format!("\"{}\"", k))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"
[project]
name = "mille-e2e-naming"
root = "."
languages = ["rust"]

[[layers]]
name = "usecase"
paths = ["tests/fixtures/naming/usecase/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
name_deny = [{deny_list}]

[[layers]]
name = "domain"
paths = ["tests/fixtures/naming/domain/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[severity]
naming_violation = "{severity}"
"#,
        deny_list = deny_list,
        severity = severity,
    )
}

// ---------------------------------------------------------------------------
// Tests: file-level naming violation
// ---------------------------------------------------------------------------

#[test]
fn test_naming_file_violation() {
    // tests/fixtures/naming/usecase/aws_client.rs — filename contains "aws"
    let cfg = TempConfig::new(
        "e2e_naming_file.toml",
        &naming_config(&["aws"], Some(&["file"])),
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "file with 'aws' in name should cause exit 1\nstdout: {}",
        stdout(&out)
    );
    let s = stdout(&out);
    assert!(
        s.contains("aws"),
        "output should contain matched keyword 'aws'\nstdout: {s}"
    );
}

// ---------------------------------------------------------------------------
// Tests: symbol-level naming violation
// ---------------------------------------------------------------------------

#[test]
fn test_naming_symbol_violation_rust() {
    // tests/fixtures/naming/usecase/symbol_violation.rs contains fn aws_connect()
    let cfg = TempConfig::new(
        "e2e_naming_symbol.toml",
        &naming_config(&["aws"], Some(&["symbol"])),
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "symbol 'aws_connect' should cause exit 1\nstdout: {}",
        stdout(&out)
    );
    let s = stdout(&out);
    assert!(
        s.contains("aws"),
        "output should contain matched keyword 'aws'\nstdout: {s}"
    );
}

// ---------------------------------------------------------------------------
// Tests: variable-level naming violation
// ---------------------------------------------------------------------------

#[test]
fn test_naming_variable_violation_rust() {
    // tests/fixtures/naming/usecase/variable_violation.rs contains let aws_url
    let cfg = TempConfig::new(
        "e2e_naming_variable.toml",
        &naming_config(&["aws"], Some(&["variable"])),
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "variable 'aws_url' should cause exit 1\nstdout: {}",
        stdout(&out)
    );
    let s = stdout(&out);
    assert!(
        s.contains("aws"),
        "output should contain matched keyword 'aws'\nstdout: {s}"
    );
}

// ---------------------------------------------------------------------------
// Tests: comment-level naming violation
// ---------------------------------------------------------------------------

#[test]
fn test_naming_comment_violation_rust() {
    // tests/fixtures/naming/usecase/comment_violation.rs contains "// use aws for storage"
    let cfg = TempConfig::new(
        "e2e_naming_comment.toml",
        &naming_config(&["aws"], Some(&["comment"])),
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "comment with 'aws' should cause exit 1\nstdout: {}",
        stdout(&out)
    );
    let s = stdout(&out);
    assert!(
        s.contains("aws"),
        "output should contain matched keyword 'aws'\nstdout: {s}"
    );
}

// ---------------------------------------------------------------------------
// Tests: clean file — no violations
// ---------------------------------------------------------------------------

#[test]
fn test_naming_no_violation_when_clean() {
    // Only target clean.rs by using a more restrictive paths pattern — but since all usecase/**
    // files are scanned, we use a keyword that none of the files contain.
    let cfg = TempConfig::new(
        "e2e_naming_clean.toml",
        &naming_config(&["gcp"], None), // "gcp" appears nowhere in fixtures
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        0,
        "no 'gcp' references → should exit 0\nstdout: {}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// Tests: output format
// ---------------------------------------------------------------------------

#[test]
fn test_naming_violation_output_contains_keyword_and_target() {
    let cfg = TempConfig::new(
        "e2e_naming_output.toml",
        &naming_config(&["aws"], Some(&["symbol"])),
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("aws"),
        "output should contain matched keyword 'aws'\nstdout: {s}"
    );
    assert!(
        s.contains("symbol") || s.contains("naming") || s.contains("Naming"),
        "output should mention target kind or naming violation\nstdout: {s}"
    );
}

// ---------------------------------------------------------------------------
// Tests: severity configuration
// ---------------------------------------------------------------------------

#[test]
fn test_naming_severity_warning_exits_0() {
    // naming_violation = "warning" → exit 0 even with violations
    let cfg = TempConfig::new(
        "e2e_naming_sev_warn.toml",
        &naming_config_with_severity(&["aws"], "warning"),
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        0,
        "naming_violation=warning should not cause exit 1\nstdout: {}",
        stdout(&out)
    );
    let s = stdout(&out);
    assert!(
        s.contains("WARN") || s.contains("warn") || s.contains("warning"),
        "output should mention warnings\nstdout: {s}"
    );
}

#[test]
fn test_naming_severity_warning_exits_1_with_fail_on_warning() {
    let cfg = TempConfig::new(
        "e2e_naming_sev_fail.toml",
        &naming_config_with_severity(&["aws"], "warning"),
    );
    let out = mille(&["check", "--config", cfg.file_name(), "--fail-on", "warning"]);
    assert_eq!(
        exit_code(&out),
        1,
        "--fail-on warning must exit 1 for naming warning violations\nstdout: {}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// Tests: name_targets filter
// ---------------------------------------------------------------------------

#[test]
fn test_naming_target_filter_file_only_ignores_symbol_violations() {
    // name_targets = ["file"] → symbol violations in symbol_violation.rs are ignored
    // But aws_client.rs (file name) should still be caught
    let cfg = TempConfig::new(
        "e2e_naming_target_file.toml",
        &naming_config(&["aws"], Some(&["file"])),
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    // aws_client.rs filename → still a violation
    assert_eq!(
        exit_code(&out),
        1,
        "file-only target should still catch aws_client.rs\nstdout: {}",
        stdout(&out)
    );
}

#[test]
fn test_naming_no_violation_for_domain_layer_without_name_deny() {
    // domain layer has AwsConfig struct but no name_deny → should not be flagged
    let cfg = TempConfig::new(
        "e2e_naming_domain_clean.toml",
        &naming_config(&["aws"], None),
    );
    // Run and check — domain violations should NOT appear
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    // domain/entity.rs should not appear in violations
    assert!(
        !s.contains("domain/entity.rs"),
        "domain layer should not produce naming violations (no name_deny set)\nstdout: {s}"
    );
}
