//! End-to-end tests for `mille check` and `mille init` with Kotlin projects.

use std::fs;
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
// Flat Kotlin layout — valid
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

// ---------------------------------------------------------------------------
// Flat Kotlin layout — violation detection
// ---------------------------------------------------------------------------

const KOTLIN_FLAT_USECASE_BLOCKS_DOMAIN_TOML: &str = r#"
[project]
name = "kotlinsample"
root = "."
languages = ["kotlin"]

[resolve.java]
module_name = "com.example.kotlinsample"

[[layers]]
name = "domain"
paths = ["**/domain/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"

[[layers]]
name = "usecase"
paths = ["**/usecase/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"

[[layers]]
name = "infrastructure"
paths = ["**/infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"

[[layers]]
name = "main"
paths = ["**/main/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
"#;

#[test]
fn test_kotlin_flat_violation_detected() {
    let config_path = kotlin_fixture_dir().join("mille_e2e_kotlin_broken.toml");
    fs::write(&config_path, KOTLIN_FLAT_USECASE_BLOCKS_DOMAIN_TOML)
        .expect("failed to write config");

    let out = mille_in_kotlin_fixture(&["check", "--config", "mille_e2e_kotlin_broken.toml"]);
    let _ = fs::remove_file(&config_path);

    assert_eq!(
        exit_code(&out),
        1,
        "usecase importing domain with allow=[] must trigger violation\nstdout:\n{}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// Gradle Kotlin layout — valid
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

// ---------------------------------------------------------------------------
// Gradle Kotlin layout — violation detection
// ---------------------------------------------------------------------------

const KOTLIN_GRADLE_USECASE_BLOCKS_DOMAIN_TOML: &str = r#"
[project]
name = "kotlinapp"
root = "."
languages = ["kotlin"]

[resolve.java]
build_gradle = "build.gradle"

[[layers]]
name = "domain"
paths = ["**/domain/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"

[[layers]]
name = "usecase"
paths = ["**/usecase/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"

[[layers]]
name = "infrastructure"
paths = ["**/infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"

[[layers]]
name = "main"
paths = ["**/kotlinapp/main/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
"#;

#[test]
fn test_kotlin_gradle_violation_detected() {
    let config_path = kotlin_gradle_fixture_dir().join("mille_e2e_kotlin_gradle_broken.toml");
    fs::write(&config_path, KOTLIN_GRADLE_USECASE_BLOCKS_DOMAIN_TOML)
        .expect("failed to write config");

    let out = mille_in_kotlin_gradle_fixture(&[
        "check",
        "--config",
        "mille_e2e_kotlin_gradle_broken.toml",
    ]);
    let _ = fs::remove_file(&config_path);

    assert_eq!(
        exit_code(&out),
        1,
        "usecase importing domain with allow=[] must trigger violation\nstdout:\n{}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// mille init — Kotlin layer detection
// ---------------------------------------------------------------------------

fn make_kt_file(dir: &std::path::Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn test_kotlin_init_detects_layers() {
    let tmp = tempfile::TempDir::new().unwrap();

    make_kt_file(
        tmp.path(),
        "src/domain/User.kt",
        "package com.example.kotlinsample.domain\ndata class User(val id: Int, val name: String)",
    );
    make_kt_file(
        tmp.path(),
        "src/usecase/UserService.kt",
        "package com.example.kotlinsample.usecase\nimport com.example.kotlinsample.domain.User\nclass UserService",
    );
    make_kt_file(
        tmp.path(),
        "src/infrastructure/UserRepo.kt",
        "package com.example.kotlinsample.infrastructure\nimport com.example.kotlinsample.domain.User\nclass UserRepo",
    );

    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["init"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute mille binary");

    assert_eq!(
        exit_code(&out),
        0,
        "mille init should exit 0\nstdout:\n{}",
        stdout(&out)
    );

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("domain")
            && content.contains("usecase")
            && content.contains("infrastructure"),
        "mille init should detect domain/usecase/infrastructure layers\ncontent:\n{}",
        content
    );
}

#[test]
fn test_kotlin_init_output_has_resolve_java() {
    let tmp = tempfile::TempDir::new().unwrap();

    // Add build.gradle so module_name can be auto-detected → [resolve.java] is emitted
    fs::write(
        tmp.path().join("build.gradle"),
        "group = 'com.example'\nversion = '1.0.0'\n",
    )
    .unwrap();
    fs::write(
        tmp.path().join("settings.gradle"),
        "rootProject.name = 'kotlinsample'\n",
    )
    .unwrap();

    make_kt_file(
        tmp.path(),
        "src/domain/User.kt",
        "package com.example.kotlinsample.domain\ndata class User(val id: Int, val name: String)",
    );
    make_kt_file(
        tmp.path(),
        "src/usecase/UserService.kt",
        "package com.example.kotlinsample.usecase\nimport com.example.kotlinsample.domain.User\nclass UserService",
    );

    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["init"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute mille binary");

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("[resolve.java]"),
        "mille init for Kotlin with build.gradle should emit [resolve.java] section\ncontent:\n{}",
        content
    );
}

#[test]
fn test_kotlin_init_paths_use_glob_prefix() {
    let tmp = tempfile::TempDir::new().unwrap();

    make_kt_file(
        tmp.path(),
        "src/domain/User.kt",
        "package com.example.kotlinsample.domain\ndata class User(val id: Int, val name: String)",
    );
    make_kt_file(
        tmp.path(),
        "src/usecase/UserService.kt",
        "package com.example.kotlinsample.usecase\nimport com.example.kotlinsample.domain.User\nclass UserService",
    );

    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["init"])
        .current_dir(tmp.path())
        .output()
        .expect("failed to execute mille binary");

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("**/domain/**"),
        "paths should use glob prefix\ncontent:\n{}",
        content
    );
    assert!(
        !content.contains("\"src/domain/"),
        "paths should NOT contain src/domain/\ncontent:\n{}",
        content
    );
}
