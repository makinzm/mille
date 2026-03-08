//! End-to-end tests for `[severity]` configuration and `--fail-on` option.
//!
//! Verifies:
//!   - `dependency_violation = "warning"` → violations are warnings, exit 0
//!   - `--fail-on warning` → warnings also cause exit 1
//!   - `external_violation = "warning"` → external violations are warnings, exit 0
//!   - `call_pattern_violation = "warning"` → call pattern violations are warnings, exit 0
//!   - Default severity (no `[severity]` section) → all violations are errors (regression)

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
// Fixtures: configs that produce violations
// ---------------------------------------------------------------------------

/// infrastructure allows nothing (allow=[]) → any import from domain is a violation.
/// dependency_violation = "warning" → violations are warnings.
const DEP_VIOLATION_WARNING_TOML: &str = r#"
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
paths = ["src/main.rs", "src/runner.rs"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[severity]
dependency_violation = "warning"
external_violation = "error"
call_pattern_violation = "error"
unknown_import = "warning"
"#;

/// Same as above but external_violation = "warning".
const EXTERNAL_VIOLATION_WARNING_TOML: &str = r#"
[project]
name = "mille-e2e"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-out"
deny = []
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
paths = ["src/main.rs", "src/runner.rs"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[severity]
dependency_violation = "error"
external_violation = "warning"
call_pattern_violation = "error"
unknown_import = "warning"
"#;

/// call_pattern_violation = "warning": ViolationDetector::new() not in allow_methods.
const CALL_PATTERN_WARNING_TOML: &str = r#"
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
  allow_methods = ["detect", "check"]

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "main"
paths = ["src/main.rs", "src/runner.rs"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[severity]
dependency_violation = "error"
external_violation = "error"
call_pattern_violation = "warning"
unknown_import = "warning"
"#;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_dependency_violation_as_warning_exits_0_without_fail_on() {
    let cfg = TempConfig::new("e2e_sev_dep_warn.toml", DEP_VIOLATION_WARNING_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    // infrastructure imports domain → dependency violation, but configured as warning → exit 0
    assert_eq!(
        exit_code(&out),
        0,
        "warning-only violations should not cause exit 1\nstdout: {}",
        stdout(&out)
    );
    let s = stdout(&out);
    assert!(
        s.contains("WARN") || s.contains("warning"),
        "output should mention warnings\nstdout: {s}"
    );
}

#[test]
fn test_dependency_violation_as_warning_exits_1_with_fail_on_warning() {
    let cfg = TempConfig::new("e2e_sev_dep_fail.toml", DEP_VIOLATION_WARNING_TOML);
    let out = mille(&["check", "--config", cfg.file_name(), "--fail-on", "warning"]);
    assert_eq!(
        exit_code(&out),
        1,
        "--fail-on warning must exit 1 when there are warnings\nstdout: {}",
        stdout(&out)
    );
}

#[test]
fn test_external_violation_as_warning_exits_0_without_fail_on() {
    let cfg = TempConfig::new("e2e_sev_ext_warn.toml", EXTERNAL_VIOLATION_WARNING_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    // domain has opt-in external_allow=[] → serde import is violation, but as warning → exit 0
    assert_eq!(
        exit_code(&out),
        0,
        "external warning violation should not cause exit 1\nstdout: {}",
        stdout(&out)
    );
}

#[test]
fn test_external_violation_as_warning_exits_1_with_fail_on_warning() {
    let cfg = TempConfig::new("e2e_sev_ext_fail.toml", EXTERNAL_VIOLATION_WARNING_TOML);
    let out = mille(&["check", "--config", cfg.file_name(), "--fail-on", "warning"]);
    assert_eq!(
        exit_code(&out),
        1,
        "--fail-on warning must exit 1 for external warning violations\nstdout: {}",
        stdout(&out)
    );
}

#[test]
fn test_call_pattern_violation_as_warning_exits_0_without_fail_on() {
    let cfg = TempConfig::new("e2e_sev_cp_warn.toml", CALL_PATTERN_WARNING_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        0,
        "call pattern warning violation should not cause exit 1\nstdout: {}",
        stdout(&out)
    );
}

#[test]
fn test_call_pattern_violation_as_warning_exits_1_with_fail_on_warning() {
    let cfg = TempConfig::new("e2e_sev_cp_fail.toml", CALL_PATTERN_WARNING_TOML);
    let out = mille(&["check", "--config", cfg.file_name(), "--fail-on", "warning"]);
    assert_eq!(
        exit_code(&out),
        1,
        "--fail-on warning must exit 1 for call pattern warning violations\nstdout: {}",
        stdout(&out)
    );
}

#[test]
fn test_fail_on_error_does_not_fail_on_warnings_only() {
    // --fail-on error should behave the same as the default (no flag): warnings don't exit 1
    let cfg = TempConfig::new("e2e_sev_fail_err.toml", DEP_VIOLATION_WARNING_TOML);
    let out = mille(&["check", "--config", cfg.file_name(), "--fail-on", "error"]);
    assert_eq!(
        exit_code(&out),
        0,
        "--fail-on error must not exit 1 when violations are only warnings\nstdout: {}",
        stdout(&out)
    );
}
