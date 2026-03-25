//! End-to-end tests for multi-file main layer pattern.
//!
//! Verifies that a "main" layer spanning multiple files (e.g. `src/main.rs` +
//! `src/runner.rs`) works correctly, including:
//!   - both files are recognised as belonging to the main layer
//!   - dependency violations in runner.rs are detected
//!   - allow_call_patterns apply to runner.rs as well
//!   - allow_call_patterns work on non-main layers (usecase)

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rust_multifile_main")
}

fn mille_at(dir: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to execute mille binary")
}

/// RAII wrapper: writes a temp TOML into the fixture directory with a unique
/// name and removes it on drop.  The fixture's original `mille.toml` is never
/// touched, so parallel test execution is safe.
struct TempConfig {
    path: PathBuf,
}

impl TempConfig {
    fn new(dir: &PathBuf, name: &str, content: &str) -> Self {
        let path = dir.join(name);
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
// Broken configs — inline TOML for violation tests
//
// All configs use root = "." and are written to the fixture directory.
// ---------------------------------------------------------------------------

/// Main layer does NOT allow infrastructure → runner.rs importing infrastructure
/// should trigger a DependencyViolation.
const MAIN_DENY_INFRA_TOML: &str = r#"
[project]
name = "rust-multifile-main"
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
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "main"
paths = ["src/main.rs", "src/runner.rs"]
dependency_mode = "opt-in"
allow = ["usecase", "domain"]
external_mode = "opt-out"
external_deny = []
"#;

/// Main layer allow_call_patterns restricts usecase calls to only "nonexistent"
/// method → runner.rs calling greet::hello() should trigger CallPatternViolation.
const MAIN_CALL_PATTERN_VIOLATION_TOML: &str = r#"
[project]
name = "rust-multifile-main"
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
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "main"
paths = ["src/main.rs", "src/runner.rs"]
dependency_mode = "opt-in"
allow = ["usecase", "infrastructure", "domain"]
external_mode = "opt-out"
external_deny = []

  [[layers.allow_call_patterns]]
  callee_layer = "usecase"
  allow_methods = ["nonexistent"]

  [[layers.allow_call_patterns]]
  callee_layer = "infrastructure"
  allow_methods = ["print"]
"#;

/// allow_call_patterns on usecase layer — restricts which domain methods
/// usecase may call.  greet.rs calls User { ... } (struct literal) which is
/// parsed as a static call.  Only "nonexistent" is allowed →
/// CallPatternViolation.
const USECASE_CALL_PATTERN_VIOLATION_TOML: &str = r#"
[project]
name = "rust-multifile-main"
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
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

  [[layers.allow_call_patterns]]
  callee_layer = "domain"
  allow_methods = ["nonexistent"]

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "main"
paths = ["src/main.rs", "src/runner.rs"]
dependency_mode = "opt-in"
allow = ["usecase", "infrastructure", "domain"]
external_mode = "opt-out"
external_deny = []
"#;

// ---------------------------------------------------------------------------
// Normal cases — multi-file main layer
// ---------------------------------------------------------------------------

#[test]
fn test_multifile_main_clean_exits_zero() {
    let dir = fixture_dir();
    let out = mille_at(&dir, &["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "clean multi-file main fixture must exit 0\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_multifile_main_both_files_in_main_layer() {
    let dir = fixture_dir();
    let out = mille_at(&dir, &["check"]);
    let s = stdout(&out);
    assert!(
        s.contains("main"),
        "output must mention 'main' layer\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Failure case — runner.rs dependency violation
// ---------------------------------------------------------------------------

#[test]
fn test_multifile_main_runner_dep_violation_exits_one() {
    let dir = fixture_dir();
    let cfg = TempConfig::new(&dir, "mille_e2e_dep.toml", MAIN_DENY_INFRA_TOML);
    let out = mille_at(&dir, &["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "runner.rs importing infrastructure (not allowed) must trigger violation\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_multifile_main_runner_dep_violation_mentions_runner() {
    let dir = fixture_dir();
    let cfg = TempConfig::new(&dir, "mille_e2e_dep2.toml", MAIN_DENY_INFRA_TOML);
    let out = mille_at(&dir, &["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("runner.rs"),
        "violation output must mention runner.rs as the offending file\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Failure case — allow_call_patterns on main layer applies to runner.rs
// ---------------------------------------------------------------------------

#[test]
fn test_multifile_main_call_pattern_violation_exits_one() {
    let dir = fixture_dir();
    let cfg = TempConfig::new(
        &dir,
        "mille_e2e_call.toml",
        MAIN_CALL_PATTERN_VIOLATION_TOML,
    );
    let out = mille_at(&dir, &["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "runner.rs calling forbidden usecase method must trigger CallPatternViolation\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_multifile_main_call_pattern_violation_mentions_hello() {
    let dir = fixture_dir();
    let cfg = TempConfig::new(
        &dir,
        "mille_e2e_call2.toml",
        MAIN_CALL_PATTERN_VIOLATION_TOML,
    );
    let out = mille_at(&dir, &["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("hello"),
        "violation output must mention the forbidden method 'hello'\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Failure case — allow_call_patterns on non-main layer (usecase)
// ---------------------------------------------------------------------------

#[test]
fn test_usecase_call_pattern_violation_exits_one() {
    let dir = fixture_dir();
    let cfg = TempConfig::new(
        &dir,
        "mille_e2e_usecase_call.toml",
        USECASE_CALL_PATTERN_VIOLATION_TOML,
    );
    let out = mille_at(&dir, &["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "usecase calling forbidden domain method must trigger CallPatternViolation\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_usecase_call_pattern_violation_mentions_usecase() {
    let dir = fixture_dir();
    let cfg = TempConfig::new(
        &dir,
        "mille_e2e_usecase_call2.toml",
        USECASE_CALL_PATTERN_VIOLATION_TOML,
    );
    let out = mille_at(&dir, &["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("usecase"),
        "violation output must mention 'usecase' as the offending layer\nstdout:\n{}",
        s
    );
}
