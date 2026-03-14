//! End-to-end tests for `mille check` with Java projects.
//!
//! Tests invoke the compiled binary against the `tests/fixtures/java_sample/` fixture
//! to verify Java language support works correctly.

use std::path::PathBuf;
use std::process::{Command, Output};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn java_fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/java_sample")
}

/// Run `mille check` from the Java fixture directory.
fn mille_in_java_fixture(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(java_fixture_dir())
        .output()
        .expect("failed to execute mille binary")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn exit_code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}

// ---------------------------------------------------------------------------
// Happy path: valid Java fixture
// ---------------------------------------------------------------------------

#[test]
fn test_java_valid_config_exits_zero() {
    let out = mille_in_java_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "java_sample mille.toml should produce no violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_java_valid_config_summary_shows_zero_errors() {
    let out = mille_in_java_fixture(&["check"]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary should show 0 error(s)\nstdout:\n{}",
        s
    );
}

#[test]
fn test_java_valid_config_all_layers_clean() {
    let out = mille_in_java_fixture(&["check"]);
    let s = stdout(&out);
    assert!(
        s.contains('✅'),
        "all layers should be ✅ with valid config\nstdout:\n{}",
        s
    );
    assert!(
        !s.contains('❌'),
        "no layer should be ❌ with valid config\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Broken config: usecase allow=[] → violation when importing domain
// ---------------------------------------------------------------------------

const USECASE_BLOCKS_DOMAIN_TOML: &str = r#"
[project]
name = "javasample"
root = "."
languages = ["java"]

[resolve.java]
module_name = "com.example.javasample"

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["usecase", "infrastructure"]
external_mode = "opt-out"

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
"#;

#[test]
fn test_java_broken_usecase_exits_one() {
    use std::fs;

    let config_path = java_fixture_dir().join("mille_e2e_broken_usecase.toml");
    fs::write(&config_path, USECASE_BLOCKS_DOMAIN_TOML).expect("failed to write config");

    let out = mille_in_java_fixture(&["check", "--config", "mille_e2e_broken_usecase.toml"]);
    let _ = fs::remove_file(&config_path);

    assert_eq!(
        exit_code(&out),
        1,
        "usecase importing domain with allow=[] must trigger a violation\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_java_broken_usecase_violation_mentions_usecase() {
    use std::fs;

    let config_path = java_fixture_dir().join("mille_e2e_broken_usecase2.toml");
    fs::write(&config_path, USECASE_BLOCKS_DOMAIN_TOML).expect("failed to write config");

    let out = mille_in_java_fixture(&["check", "--config", "mille_e2e_broken_usecase2.toml"]);
    let _ = fs::remove_file(&config_path);

    let s = stdout(&out);
    assert!(
        s.contains("usecase"),
        "violation output must mention 'usecase'\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Broken config: infrastructure external_allow=[] → violation when importing java.util.List
// ---------------------------------------------------------------------------

const INFRA_EMPTY_EXTERNAL_ALLOW_TOML: &str = r#"
[project]
name = "javasample"
root = "."
languages = ["java"]

[resolve.java]
module_name = "com.example.javasample"

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["usecase", "infrastructure"]
external_mode = "opt-out"

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []
"#;

#[test]
fn test_java_infra_empty_external_allow_exits_one() {
    use std::fs;

    let config_path = java_fixture_dir().join("mille_e2e_infra_ext_allow.toml");
    fs::write(&config_path, INFRA_EMPTY_EXTERNAL_ALLOW_TOML).expect("failed to write config");

    let out = mille_in_java_fixture(&["check", "--config", "mille_e2e_infra_ext_allow.toml"]);
    let _ = fs::remove_file(&config_path);

    assert_eq!(
        exit_code(&out),
        1,
        "infrastructure imports java.util.List with external_allow=[]: must trigger violation\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_java_infra_empty_external_allow_mentions_java_util() {
    use std::fs;

    let config_path = java_fixture_dir().join("mille_e2e_infra_ext_allow2.toml");
    fs::write(&config_path, INFRA_EMPTY_EXTERNAL_ALLOW_TOML).expect("failed to write config");

    let out = mille_in_java_fixture(&["check", "--config", "mille_e2e_infra_ext_allow2.toml"]);
    let _ = fs::remove_file(&config_path);

    let s = stdout(&out);
    assert!(
        s.contains("java.util") || s.contains("List"),
        "violation output must mention 'java.util.List'\nstdout:\n{}",
        s
    );
}
