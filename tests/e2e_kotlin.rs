//! End-to-end tests for `mille check` and `mille init` with Kotlin projects.

use std::path::PathBuf;
use std::process::{Command, Output};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn kotlin_fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/kotlin_sample")
}

fn kotlin_gradle_fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/kotlin_gradle_sample")
}

fn mille_in_kotlin_fixture(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(kotlin_fixture_dir())
        .output()
        .expect("failed to execute mille binary")
}

fn mille_in_kotlin_gradle_fixture(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(kotlin_gradle_fixture_dir())
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
// Flat Kotlin layout
// ---------------------------------------------------------------------------

#[test]
fn test_kotlin_flat_valid_exits_zero() {
    let out = mille_in_kotlin_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "kotlin_sample should produce no violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_kotlin_flat_violation_detected() {
    // Inline config where usecase denies domain (reversed: domain blocks usecase)
    let broken_toml = r#"
[project]
name = "kotlinsample"
root = "."
languages = ["kotlin"]

[resolve.java]
module_name = "com.example.kotlinsample"

[[layers]]
name = "domain"
paths = ["**/domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"

[[layers]]
name = "usecase"
paths = ["**/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"

[[layers]]
name = "infrastructure"
paths = ["**/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"

[[layers]]
name = "main"
paths = ["**/main/**"]
dependency_mode = "opt-in"
allow = ["domain", "usecase", "infrastructure"]
external_mode = "opt-out"
"#;
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), broken_toml).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config"])
        .arg(tmp.path())
        .current_dir(kotlin_fixture_dir())
        .output()
        .expect("failed to execute mille binary");

    assert_ne!(
        exit_code(&out),
        0,
        "broken config should detect violations\nstdout:\n{}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// Gradle Kotlin layout
// ---------------------------------------------------------------------------

#[test]
fn test_kotlin_gradle_valid_exits_zero() {
    let out = mille_in_kotlin_gradle_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "kotlin_gradle_sample should produce no violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_kotlin_gradle_violation_detected() {
    let broken_toml = r#"
[project]
name = "kotlinapp"
root = "."
languages = ["kotlin"]

[resolve.java]
build_gradle = "build.gradle"

[[layers]]
name = "domain"
paths = ["**/domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"

[[layers]]
name = "usecase"
paths = ["**/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"

[[layers]]
name = "infrastructure"
paths = ["**/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"

[[layers]]
name = "main"
paths = ["**/kotlinapp/main/**"]
dependency_mode = "opt-in"
allow = ["domain", "usecase", "infrastructure"]
external_mode = "opt-out"
"#;
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), broken_toml).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config"])
        .arg(tmp.path())
        .current_dir(kotlin_gradle_fixture_dir())
        .output()
        .expect("failed to execute mille binary");

    assert_ne!(
        exit_code(&out),
        0,
        "broken gradle config should detect violations\nstdout:\n{}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// mille init — Kotlin layer detection
// ---------------------------------------------------------------------------

#[test]
fn test_kotlin_init_detects_layers() {
    let out = mille_in_kotlin_fixture(&["init", "--dry-run"]);
    let s = stdout(&out);
    assert!(
        s.contains("domain") && s.contains("usecase") && s.contains("infrastructure"),
        "mille init should detect domain/usecase/infrastructure layers\nstdout:\n{}",
        s
    );
}

#[test]
fn test_kotlin_init_output_has_resolve_java() {
    let out = mille_in_kotlin_fixture(&["init", "--dry-run"]);
    let s = stdout(&out);
    assert!(
        s.contains("[resolve.java]"),
        "mille init for Kotlin should emit [resolve.java] section\nstdout:\n{}",
        s
    );
}

#[test]
fn test_kotlin_init_paths_use_glob_prefix() {
    let out = mille_in_kotlin_fixture(&["init", "--dry-run"]);
    let s = stdout(&out);
    // paths should be **/layer/** not src/layer/**
    assert!(
        s.contains("**/domain/**"),
        "paths should use glob prefix\nstdout:\n{}",
        s
    );
    assert!(
        !s.contains("src/domain/"),
        "paths should NOT contain src/domain/\nstdout:\n{}",
        s
    );
}
