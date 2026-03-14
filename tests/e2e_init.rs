//! End-to-end tests for `mille init`.
//!
//! Each test creates a temporary directory with a realistic project structure,
//! invokes the compiled binary, and verifies exit code + file contents.

use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run `mille init` (plus any extra args) with `current_dir` set to `dir`.
fn mille_init(dir: &std::path::Path, extra_args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mille"));
    cmd.arg("init");
    cmd.args(extra_args);
    cmd.current_dir(dir);
    cmd.output().expect("failed to execute mille binary")
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

fn make_dir(base: &std::path::Path, rel: &str) {
    fs::create_dir_all(base.join(rel)).unwrap();
}

fn make_file(base: &std::path::Path, rel: &str, content: &str) {
    let path = base.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_init_creates_toml_from_layer_dirs() {
    let tmp = TempDir::new().unwrap();
    // domain has no internal imports
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");
    // usecase imports from domain
    make_file(
        tmp.path(),
        "src/usecase/check.rs",
        "use crate::domain::entity::User;",
    );
    // infrastructure also imports from domain
    make_file(
        tmp.path(),
        "src/infrastructure/repo.rs",
        "use crate::domain::entity::User;",
    );

    let out = mille_init(tmp.path(), &[]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille init should exit 0\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let toml_path = tmp.path().join("mille.toml");
    assert!(toml_path.exists(), "mille.toml should be created");

    let content = fs::read_to_string(&toml_path).unwrap();
    assert!(
        content.contains("[project]"),
        "generated TOML must contain [project]\n{}",
        content
    );
    assert!(
        content.contains("[[layers]]"),
        "generated TOML must contain [[layers]]\n{}",
        content
    );
    // domain should have no allow (no internal deps)
    // usecase and infrastructure should reference domain in allow
    assert!(
        content.contains("\"domain\""),
        "generated TOML must reference the domain layer\n{}",
        content
    );
    // external_mode is a required field — must always be present
    assert!(
        content.contains("external_mode"),
        "generated TOML must contain external_mode for every layer\n{}",
        content
    );
}

#[test]
fn test_init_with_output_flag() {
    let tmp = TempDir::new().unwrap();
    make_dir(tmp.path(), "src/domain");
    make_file(tmp.path(), "src/domain/mod.rs", "");

    let out = mille_init(tmp.path(), &["--output", "custom.toml"]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille init --output custom.toml should exit 0\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let toml_path = tmp.path().join("custom.toml");
    assert!(toml_path.exists(), "custom.toml should be created");
    assert!(
        !tmp.path().join("mille.toml").exists(),
        "default mille.toml must NOT be created when --output is set"
    );
}

#[test]
fn test_init_existing_file_without_force_exits_error() {
    let tmp = TempDir::new().unwrap();
    let existing = tmp.path().join("mille.toml");
    fs::write(&existing, "# existing content").unwrap();

    let out = mille_init(tmp.path(), &[]);
    assert_ne!(
        exit_code(&out),
        0,
        "should fail when mille.toml already exists and --force is not set\nstdout:\n{}",
        stdout(&out)
    );

    // File must not be modified
    let content = fs::read_to_string(&existing).unwrap();
    assert_eq!(
        content, "# existing content",
        "existing file must not be modified"
    );
}

#[test]
fn test_init_with_depth_flag() {
    let tmp = TempDir::new().unwrap();
    // Nested project: src/domain/entity and src/domain/repository rolled up to src/domain
    make_file(
        tmp.path(),
        "src/domain/entity/user.rs",
        "pub struct User { pub id: u64 }",
    );
    make_file(
        tmp.path(),
        "src/domain/repository/repo.rs",
        "pub trait UserRepo {}",
    );
    // usecase imports from domain
    make_file(
        tmp.path(),
        "src/usecase/check.rs",
        "use crate::domain::entity::User;",
    );

    let out = mille_init(tmp.path(), &["--depth", "2"]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille init --depth 2 should exit 0\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let toml_path = tmp.path().join("mille.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    // Should have domain and usecase layers but NOT entity or repository
    assert!(
        content.contains("\"domain\""),
        "should have domain layer\n{}",
        content
    );
    assert!(
        content.contains("\"usecase\""),
        "should have usecase layer\n{}",
        content
    );
    assert!(
        !content.contains("\"entity\""),
        "entity should be rolled up into domain, not a separate layer\n{}",
        content
    );
    assert!(
        !content.contains("\"repository\""),
        "repository should be rolled up into domain, not a separate layer\n{}",
        content
    );
    // usecase should depend on domain
    assert!(
        content.contains("allow = [\"domain\"]"),
        "usecase should allow domain\n{}",
        content
    );
}

#[test]
fn test_init_depth3_disambiguates_entity_layers() {
    let tmp = TempDir::new().unwrap();
    // Two "entity" dirs with different parents
    make_file(
        tmp.path(),
        "src/domain/entity/user.rs",
        "pub struct User { pub id: u64 }",
    );
    make_file(
        tmp.path(),
        "src/infrastructure/entity/model.rs",
        "use crate::domain::entity::User;",
    );

    let out = mille_init(tmp.path(), &["--depth", "3"]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille init --depth 3 should exit 0\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let toml_path = tmp.path().join("mille.toml");
    let content = fs::read_to_string(&toml_path).unwrap();

    assert!(
        content.contains("\"domain_entity\""),
        "should have domain_entity layer\n{}",
        content
    );
    assert!(
        content.contains("\"infrastructure_entity\""),
        "should have infrastructure_entity layer\n{}",
        content
    );
    assert!(
        !content.contains("name = \"entity\""),
        "plain 'entity' layer should not exist when parents differ\n{}",
        content
    );
}

#[test]
fn test_init_existing_file_with_force_overwrites() {
    let tmp = TempDir::new().unwrap();
    let existing = tmp.path().join("mille.toml");
    fs::write(&existing, "# old content").unwrap();

    make_dir(tmp.path(), "src/domain");
    make_file(tmp.path(), "src/domain/mod.rs", "");

    let out = mille_init(tmp.path(), &["--force"]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille init --force should overwrite existing file\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let content = fs::read_to_string(&existing).unwrap();
    assert_ne!(
        content, "# old content",
        "file should have been overwritten by --force"
    );
    assert!(
        content.contains("[project]"),
        "overwritten file must be valid config\n{}",
        content
    );
}

// ---------------------------------------------------------------------------
// Java: flat layout (src/domain/, src/usecase/, ...)
// ---------------------------------------------------------------------------

#[test]
fn test_java_init_flat_detects_layers() {
    let tmp = TempDir::new().unwrap();
    // domain — no imports
    make_file(
        tmp.path(),
        "src/domain/User.java",
        "package com.example.myapp.domain;\npublic class User {}",
    );
    // usecase — imports domain
    make_file(
        tmp.path(),
        "src/usecase/UserService.java",
        "package com.example.myapp.usecase;\nimport com.example.myapp.domain.User;\npublic class UserService {}",
    );
    // infrastructure — imports domain
    make_file(
        tmp.path(),
        "src/infrastructure/UserRepo.java",
        "package com.example.myapp.infrastructure;\nimport com.example.myapp.domain.User;\nimport java.util.List;\npublic class UserRepo {}",
    );

    let out = mille_init(tmp.path(), &[]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille init should exit 0 for flat Java layout\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("\"domain\""),
        "domain layer must be detected\n{}",
        content
    );
    assert!(
        content.contains("\"usecase\""),
        "usecase layer must be detected\n{}",
        content
    );
    assert!(
        content.contains("\"infrastructure\""),
        "infrastructure layer must be detected\n{}",
        content
    );
}

#[test]
fn test_java_init_flat_usecase_allows_domain() {
    let tmp = TempDir::new().unwrap();
    make_file(
        tmp.path(),
        "src/domain/User.java",
        "package com.example.myapp.domain;\npublic class User {}",
    );
    make_file(
        tmp.path(),
        "src/usecase/UserService.java",
        "package com.example.myapp.usecase;\nimport com.example.myapp.domain.User;\npublic class UserService {}",
    );

    let out = mille_init(tmp.path(), &[]);
    assert_eq!(exit_code(&out), 0, "stdout:\n{}", stdout(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("allow = [\"domain\"]"),
        "usecase should allow domain\n{}",
        content
    );
}

#[test]
fn test_java_init_flat_paths_use_glob_prefix() {
    // Java layer paths must be "**/domain/**" form, not "src/domain/**"
    let tmp = TempDir::new().unwrap();
    make_file(
        tmp.path(),
        "src/domain/User.java",
        "package com.example.myapp.domain;\npublic class User {}",
    );
    make_file(
        tmp.path(),
        "src/usecase/UserService.java",
        "package com.example.myapp.usecase;\nimport com.example.myapp.domain.User;\npublic class UserService {}",
    );

    let out = mille_init(tmp.path(), &[]);
    assert_eq!(exit_code(&out), 0, "stdout:\n{}", stdout(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("**/domain/**"),
        "domain path should use **/domain/** glob\n{}",
        content
    );
    assert!(
        content.contains("**/usecase/**"),
        "usecase path should use **/usecase/** glob\n{}",
        content
    );
    assert!(
        !content.contains("src/domain"),
        "should NOT use src/domain (depth-based) path\n{}",
        content
    );
}

// ---------------------------------------------------------------------------
// Java: Maven standard layout (src/main/java/com/example/myapp/...)
// ---------------------------------------------------------------------------

#[test]
fn test_java_init_maven_detects_layers() {
    let tmp = TempDir::new().unwrap();

    make_file(
        tmp.path(),
        "pom.xml",
        r#"<?xml version="1.0"?><project><groupId>com.example</groupId><artifactId>myapp</artifactId></project>"#,
    );
    make_file(
        tmp.path(),
        "src/main/java/com/example/myapp/domain/User.java",
        "package com.example.myapp.domain;\npublic class User {}",
    );
    make_file(
        tmp.path(),
        "src/main/java/com/example/myapp/usecase/UserService.java",
        "package com.example.myapp.usecase;\nimport com.example.myapp.domain.User;\npublic class UserService {}",
    );
    make_file(
        tmp.path(),
        "src/main/java/com/example/myapp/infrastructure/UserRepo.java",
        "package com.example.myapp.infrastructure;\nimport com.example.myapp.domain.User;\npublic class UserRepo {}",
    );

    let out = mille_init(tmp.path(), &[]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille init should exit 0 for Maven layout\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("\"domain\""),
        "domain layer must be detected in Maven layout\n{}",
        content
    );
    assert!(
        content.contains("\"usecase\""),
        "usecase layer must be detected\n{}",
        content
    );
    assert!(
        content.contains("\"infrastructure\""),
        "infrastructure layer must be detected\n{}",
        content
    );
}

#[test]
fn test_java_init_maven_output_has_resolve_java() {
    let tmp = TempDir::new().unwrap();

    make_file(
        tmp.path(),
        "pom.xml",
        r#"<?xml version="1.0"?><project><groupId>com.example</groupId><artifactId>myapp</artifactId></project>"#,
    );
    make_file(
        tmp.path(),
        "src/main/java/com/example/myapp/domain/User.java",
        "package com.example.myapp.domain;\npublic class User {}",
    );
    make_file(
        tmp.path(),
        "src/main/java/com/example/myapp/usecase/UserService.java",
        "package com.example.myapp.usecase;\nimport com.example.myapp.domain.User;\npublic class UserService {}",
    );

    let out = mille_init(tmp.path(), &[]);
    assert_eq!(exit_code(&out), 0, "stdout:\n{}", stdout(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("[resolve.java]"),
        "Maven project must have [resolve.java] section\n{}",
        content
    );
    assert!(
        content.contains("module_name = \"com.example.myapp\""),
        "module_name must be auto-detected from pom.xml\n{}",
        content
    );
}

#[test]
fn test_java_init_maven_paths_use_glob_prefix() {
    // Maven layout: paths must be **/domain/** not src/main/java/com/example/myapp/domain/**
    let tmp = TempDir::new().unwrap();

    make_file(
        tmp.path(),
        "pom.xml",
        r#"<?xml version="1.0"?><project><groupId>com.example</groupId><artifactId>myapp</artifactId></project>"#,
    );
    make_file(
        tmp.path(),
        "src/main/java/com/example/myapp/domain/User.java",
        "package com.example.myapp.domain;\npublic class User {}",
    );
    make_file(
        tmp.path(),
        "src/main/java/com/example/myapp/usecase/UserService.java",
        "package com.example.myapp.usecase;\nimport com.example.myapp.domain.User;\npublic class UserService {}",
    );

    let out = mille_init(tmp.path(), &[]);
    assert_eq!(exit_code(&out), 0, "stdout:\n{}", stdout(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("**/domain/**"),
        "domain path must use **/domain/** glob\n{}",
        content
    );
    assert!(
        !content.contains("src/main/java"),
        "path must NOT include Maven source root prefix\n{}",
        content
    );
}
