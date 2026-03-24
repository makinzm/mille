//! End-to-end tests for `mille check` with YAML projects.
//!
//! YAML is a naming-only language — it has no imports, so dependency_mode,
//! external_mode, and allow_call_patterns tests are N/A.
//!
//! Tests invoke the compiled binary against the `tests/fixtures/yaml_sample/` fixture
//! to verify YAML language support works correctly.

use std::path::PathBuf;
use std::process::{Command, Output};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn yaml_fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/yaml_sample")
}

/// Run `mille check` (or other subcommand) from the YAML fixture directory.
fn mille_in_yaml_fixture(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(yaml_fixture_dir())
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
// Happy path: valid YAML fixture
// ---------------------------------------------------------------------------

#[test]
fn test_yaml_valid_config_exits_zero() {
    let out = mille_in_yaml_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "yaml_sample mille.toml should produce no violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_yaml_valid_config_summary_shows_zero_errors() {
    let out = mille_in_yaml_fixture(&["check"]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary should show 0 error(s)\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Broken config: naming — config layer with name_deny=["aws"]
// ---------------------------------------------------------------------------

/// config/deployment.yaml contains `aws_region` key and `"aws-east-1"` value.
/// Setting name_deny=["aws"] on the config layer must trigger a NamingViolation.
const YAML_BROKEN_NAMING_TOML: &str = r#"
[project]
name = "yaml-sample"
root = "."
languages = ["yaml"]

[[layers]]
name = "config"
paths = ["config/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws"]

[[layers]]
name = "manifests"
paths = ["manifests/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
"#;

#[test]
fn test_yaml_broken_naming_exits_one() {
    use std::fs;
    let config_path = yaml_fixture_dir().join("mille_e2e_naming.toml");
    fs::write(&config_path, YAML_BROKEN_NAMING_TOML).expect("failed to write config");

    let out = mille_in_yaml_fixture(&["check", "--config", "mille_e2e_naming.toml"]);
    let _ = fs::remove_file(&config_path);

    assert_eq!(
        exit_code(&out),
        1,
        "config layer with name_deny=[aws] must trigger NamingViolation\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_yaml_broken_naming_mentions_layer() {
    use std::fs;
    let config_path = yaml_fixture_dir().join("mille_e2e_naming2.toml");
    fs::write(&config_path, YAML_BROKEN_NAMING_TOML).expect("failed to write config");

    let out = mille_in_yaml_fixture(&["check", "--config", "mille_e2e_naming2.toml"]);
    let _ = fs::remove_file(&config_path);

    let s = stdout(&out);
    assert!(
        s.contains("config"),
        "NamingViolation output must mention 'config' layer\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Clean layer: manifests has no name_deny — aws keyword is OK
// ---------------------------------------------------------------------------

#[test]
fn test_yaml_no_naming_violation_for_clean_layer() {
    use std::fs;
    let config_path = yaml_fixture_dir().join("mille_e2e_clean_layer.toml");
    fs::write(&config_path, YAML_BROKEN_NAMING_TOML).expect("failed to write config");

    let out = mille_in_yaml_fixture(&["check", "--config", "mille_e2e_clean_layer.toml"]);
    let _ = fs::remove_file(&config_path);

    let s = stdout(&out);
    // manifests layer should NOT have violations since it has no name_deny
    assert!(
        !s.contains("manifests") || !s.contains("NamingViolation"),
        "manifests layer without name_deny should not have NamingViolation\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// name_targets: symbol only — only keys (Symbol) should be checked
// ---------------------------------------------------------------------------

/// With name_targets=["symbol"], only mapping keys should trigger violations.
/// `aws_region` key → Symbol → violation.
/// `"aws-east-1"` value → StringLiteral → NOT checked.
const YAML_NAME_TARGETS_SYMBOL_ONLY_TOML: &str = r#"
[project]
name = "yaml-sample"
root = "."
languages = ["yaml"]

[[layers]]
name = "config"
paths = ["config/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws"]
name_targets = ["symbol"]

[[layers]]
name = "manifests"
paths = ["manifests/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
"#;

#[test]
fn test_yaml_name_targets_symbol_only() {
    use std::fs;
    let config_path = yaml_fixture_dir().join("mille_e2e_symbol_only.toml");
    fs::write(&config_path, YAML_NAME_TARGETS_SYMBOL_ONLY_TOML).expect("failed to write config");

    let out = mille_in_yaml_fixture(&["check", "--config", "mille_e2e_symbol_only.toml"]);
    let _ = fs::remove_file(&config_path);

    // Should still exit 1 because aws_region key matches
    assert_eq!(
        exit_code(&out),
        1,
        "name_targets=[symbol] should still catch aws_region key\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

// ---------------------------------------------------------------------------
// name_targets: string_literal only — only values should be checked
// ---------------------------------------------------------------------------

/// With name_targets=["string_literal"], only scalar values should trigger violations.
/// `"aws-east-1"` value → StringLiteral → violation.
/// `aws_region` key → Symbol → NOT checked.
const YAML_NAME_TARGETS_STRING_LITERAL_ONLY_TOML: &str = r#"
[project]
name = "yaml-sample"
root = "."
languages = ["yaml"]

[[layers]]
name = "config"
paths = ["config/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws"]
name_targets = ["string_literal"]

[[layers]]
name = "manifests"
paths = ["manifests/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
"#;

#[test]
fn test_yaml_name_targets_string_literal_only() {
    use std::fs;
    let config_path = yaml_fixture_dir().join("mille_e2e_string_only.toml");
    fs::write(&config_path, YAML_NAME_TARGETS_STRING_LITERAL_ONLY_TOML)
        .expect("failed to write config");

    let out = mille_in_yaml_fixture(&["check", "--config", "mille_e2e_string_only.toml"]);
    let _ = fs::remove_file(&config_path);

    // Should exit 1 because "aws-east-1" value matches
    assert_eq!(
        exit_code(&out),
        1,
        "name_targets=[string_literal] should catch aws-east-1 value\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}
