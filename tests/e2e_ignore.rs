//! End-to-end tests for `[ignore]` section in mille.toml.
//!
//! Verifies that `ignore.paths` excludes files from architecture checks entirely,
//! and `ignore.test_patterns` excludes files from violation detection
//! (they are still counted in layer stats).

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

// ---------------------------------------------------------------------------
// Helpers
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
// Fixture design
//
// All layers are opt-out (permissive) EXCEPT infrastructure, which is opt-in
// with allow=[] — meaning any import from domain triggers a violation.
// This ensures only infrastructure files produce violations, and ignoring them
// makes the check clean.
// ---------------------------------------------------------------------------

/// Baseline: only infrastructure has violations (all other layers are opt-out).
const ONLY_INFRA_VIOLATIONS_TOML: &str = r#"
[project]
name = "mille-e2e"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;

/// Same as above but with ignore.paths excluding infrastructure files.
const IGNORE_INFRA_PATHS_TOML: &str = r#"
[project]
name = "mille-e2e"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[ignore]
paths = ["src/infrastructure/**"]
"#;

/// Same as ONLY_INFRA_VIOLATIONS but with ignore.test_patterns for infrastructure.
const TEST_PATTERNS_INFRA_TOML: &str = r#"
[project]
name = "mille-e2e"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[ignore]
test_patterns = ["src/infrastructure/**"]
"#;

// ---------------------------------------------------------------------------
// Baseline: no [ignore] → infrastructure violations exist
// ---------------------------------------------------------------------------

#[test]
fn test_baseline_no_ignore_exits_one() {
    let cfg = TempConfig::new("mille_e2e_ignore_baseline.toml", ONLY_INFRA_VIOLATIONS_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "without [ignore], infra violations must appear\nstdout:\n{}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// ignore.paths tests
// ---------------------------------------------------------------------------

#[test]
fn test_ignore_paths_removes_violations() {
    let cfg = TempConfig::new("mille_e2e_ignore_paths.toml", IGNORE_INFRA_PATHS_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        0,
        "ignore.paths for infrastructure must suppress all infra violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_ignore_paths_summary_shows_zero_errors() {
    let cfg = TempConfig::new("mille_e2e_ignore_paths2.toml", IGNORE_INFRA_PATHS_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary must show 0 error(s) when infra is in ignore.paths\nstdout:\n{s}"
    );
}

#[test]
fn test_ignore_paths_reduces_infrastructure_file_count_to_zero() {
    // With ignore.paths = ["src/infrastructure/**"], infrastructure file count must be 0.
    let cfg = TempConfig::new("mille_e2e_ignore_paths3.toml", IGNORE_INFRA_PATHS_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    // The layer stat line for "infrastructure" must show 0 file(s).
    let infra_line = s
        .lines()
        .find(|l| l.contains("infrastructure"))
        .unwrap_or("");
    assert!(
        infra_line.contains("0 file(s)"),
        "ignored files must not be counted in layer stats\ninfra line: {infra_line}\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// ignore.test_patterns tests
// ---------------------------------------------------------------------------

#[test]
fn test_test_patterns_removes_violations() {
    let cfg = TempConfig::new("mille_e2e_test_patterns.toml", TEST_PATTERNS_INFRA_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        0,
        "test_patterns for infrastructure must suppress infra violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_test_patterns_summary_shows_zero_errors() {
    let cfg = TempConfig::new("mille_e2e_test_patterns2.toml", TEST_PATTERNS_INFRA_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary must show 0 error(s) when infra is in test_patterns\nstdout:\n{s}"
    );
}

#[test]
fn test_test_patterns_keeps_infrastructure_file_count() {
    // With test_patterns, files are still counted in layer stats but not violation-checked.
    let cfg = TempConfig::new("mille_e2e_test_patterns3.toml", TEST_PATTERNS_INFRA_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    let infra_line = s
        .lines()
        .find(|l| l.contains("infrastructure"))
        .unwrap_or("");
    assert!(
        !infra_line.is_empty() && !infra_line.contains("  0 file(s)"),
        "test_patterns files must still be counted in layer stats (non-zero)\ninfra line: {infra_line}\nstdout:\n{s}"
    );
}
