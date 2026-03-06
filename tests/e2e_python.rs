//! End-to-end tests for `mille check` with Python projects.
//!
//! Tests invoke the compiled binary against the `tests/fixtures/python_sample/` fixture
//! to verify Python language support works correctly.

use std::path::PathBuf;
use std::process::{Command, Output};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn python_fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/python_sample")
}

/// Run `mille check` from the Python fixture directory.
fn mille_in_python_fixture(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(python_fixture_dir())
        .output()
        .expect("failed to execute mille binary")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn exit_code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}

// ---------------------------------------------------------------------------
// Happy path: valid Python fixture
// ---------------------------------------------------------------------------

#[test]
fn test_python_valid_config_exits_zero() {
    let out = mille_in_python_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "python_sample mille.toml should produce no violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_python_valid_config_summary_shows_zero_errors() {
    let out = mille_in_python_fixture(&["check"]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary must show 0 errors, got:\n{s}"
    );
}

#[test]
fn test_python_valid_config_all_layers_clean() {
    let out = mille_in_python_fixture(&["check"]);
    let s = stdout(&out);
    assert!(s.contains("domain"), "output must list domain layer");
    assert!(s.contains("usecase"), "output must list usecase layer");
    assert!(
        s.contains("infrastructure"),
        "output must list infrastructure layer"
    );
}

// ---------------------------------------------------------------------------
// Broken fixture: usecase imports infrastructure (violates dependency_mode=opt-in)
// ---------------------------------------------------------------------------

#[test]
fn test_python_broken_usecase_exits_one() {
    // Temporarily patch the config to break the usecase allow list
    let broken_config = python_fixture_dir().join("mille_broken_usecase.toml");
    // Write a broken config on the fly
    let config_content = r#"
[project]
name = "python-sample"
root = "."
languages = ["python"]

[resolve.python]
src_root = "."
package_names = ["domain", "usecase", "infrastructure"]

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["usecase/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_python_fixture(&["check", "--config", "mille_broken_usecase.toml"]);
    std::fs::remove_file(&broken_config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "broken usecase config must exit 1\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_python_broken_usecase_violation_mentions_usecase() {
    let broken_config = python_fixture_dir().join("mille_broken_usecase2.toml");
    let config_content = r#"
[project]
name = "python-sample"
root = "."
languages = ["python"]

[resolve.python]
src_root = "."
package_names = ["domain", "usecase", "infrastructure"]

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["usecase/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_python_fixture(&["check", "--config", "mille_broken_usecase2.toml"]);
    std::fs::remove_file(&broken_config).ok();

    let s = stdout(&out);
    assert!(
        s.contains("usecase"),
        "violation output must mention 'usecase' layer\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// Broken fixture: infrastructure denies domain (dependency_mode=opt-out)
// ---------------------------------------------------------------------------

/// infrastructure/db.py imports `from domain.entity import User`.
/// Setting `deny = ["domain"]` must produce a violation.
#[test]
fn test_python_broken_infra_deny_domain_exits_one() {
    let broken_config = python_fixture_dir().join("mille_broken_infra_deny.toml");
    let config_content = r#"
[project]
name = "python-sample"
root = "."
languages = ["python"]

[resolve.python]
src_root = "."
package_names = ["domain", "usecase", "infrastructure"]

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = ["domain"]
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_python_fixture(&["check", "--config", "mille_broken_infra_deny.toml"]);
    std::fs::remove_file(&broken_config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "infrastructure deny=domain must exit 1\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_python_broken_infra_deny_domain_mentions_infrastructure() {
    let broken_config = python_fixture_dir().join("mille_broken_infra_deny2.toml");
    let config_content = r#"
[project]
name = "python-sample"
root = "."
languages = ["python"]

[resolve.python]
src_root = "."
package_names = ["domain", "usecase", "infrastructure"]

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = ["domain"]
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_python_fixture(&["check", "--config", "mille_broken_infra_deny2.toml"]);
    std::fs::remove_file(&broken_config).ok();

    let s = stdout(&out);
    assert!(
        s.contains("infrastructure"),
        "violation must mention 'infrastructure' layer\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// Broken fixture: domain denies "os" (external_mode=opt-out + external_deny)
// ---------------------------------------------------------------------------

/// domain/entity.py has `import os`.
/// Setting `external_deny = ["os"]` must produce an external violation.
#[test]
fn test_python_broken_external_deny_os_exits_one() {
    let broken_config = python_fixture_dir().join("mille_broken_ext_deny.toml");
    let config_content = r#"
[project]
name = "python-sample"
root = "."
languages = ["python"]

[resolve.python]
src_root = "."
package_names = ["domain", "usecase", "infrastructure"]

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = ["os"]

[[layers]]
name = "usecase"
paths = ["usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_python_fixture(&["check", "--config", "mille_broken_ext_deny.toml"]);
    std::fs::remove_file(&broken_config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "external_deny=[os] must exit 1\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_python_broken_external_deny_os_mentions_domain() {
    let broken_config = python_fixture_dir().join("mille_broken_ext_deny2.toml");
    let config_content = r#"
[project]
name = "python-sample"
root = "."
languages = ["python"]

[resolve.python]
src_root = "."
package_names = ["domain", "usecase", "infrastructure"]

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = ["os"]

[[layers]]
name = "usecase"
paths = ["usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_python_fixture(&["check", "--config", "mille_broken_ext_deny2.toml"]);
    std::fs::remove_file(&broken_config).ok();

    let s = stdout(&out);
    assert!(
        s.contains("domain"),
        "violation must mention 'domain' layer\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// allow_call_patterns: forbidden method call on domain entity
// ---------------------------------------------------------------------------

/// main/app.py calls User.create() but allow_methods=[] → CallPatternViolation.
const PYTHON_BROKEN_CALL_PATTERN_TOML: &str = r#"
[project]
name = "python-sample"
root = "."
languages = ["python"]

[resolve.python]
src_root = "."
package_names = ["domain", "usecase", "infrastructure", "main"]

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "main"
paths = ["main/**"]
dependency_mode = "opt-in"
allow = ["domain", "usecase", "infrastructure"]
external_mode = "opt-out"
external_deny = []

  [[layers.allow_call_patterns]]
  callee_layer = "domain"
  allow_methods = []
"#;

#[test]
fn test_python_broken_call_pattern_exits_one() {
    let config = python_fixture_dir().join("mille_e2e_py_call_pattern.toml");
    std::fs::write(&config, PYTHON_BROKEN_CALL_PATTERN_TOML).expect("failed to write config");

    let out = mille_in_python_fixture(&["check", "--config", "mille_e2e_py_call_pattern.toml"]);
    std::fs::remove_file(&config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "User.create() is forbidden (allow_methods=[]): must trigger CallPatternViolation\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}
