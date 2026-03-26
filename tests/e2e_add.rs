//! End-to-end tests for `mille add`.
//!
//! Each test creates a temporary directory with a realistic project structure,
//! invokes the compiled binary, and verifies exit code + file contents.

use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run `mille add` (plus any extra args) with `current_dir` set to `dir`.
fn mille_add(dir: &std::path::Path, target: &str, extra_args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mille"));
    cmd.arg("add");
    cmd.arg(target);
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

fn make_file(base: &std::path::Path, rel: &str, content: &str) {
    let path = base.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

/// Create a minimal mille.toml with given content.
fn write_config(dir: &std::path::Path, content: &str) {
    fs::write(dir.join("mille.toml"), content).unwrap();
}

/// Minimal mille.toml with one layer (domain).
fn base_config() -> &'static str {
    r#"[project]
name = "test_project"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#
}

/// Config with [resolve] section.
fn config_with_resolve() -> &'static str {
    r#"[project]
name = "test_project"
root = "."
languages = ["rust"]

[resolve.rust]
package_names = ["domain"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_add_new_layer_to_existing_toml() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");
    make_file(
        tmp.path(),
        "src/newlayer/service.rs",
        "use crate::domain::entity::User;",
    );

    let out = mille_add(tmp.path(), "src/newlayer", &[]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille add should exit 0\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    // New layer should be appended
    assert!(
        content.contains("name = \"newlayer\""),
        "new layer should be added\n{}",
        content
    );
    assert!(
        content.contains("src/newlayer/**"),
        "new layer path should use glob\n{}",
        content
    );
}

#[test]
fn test_add_preserves_existing_layers() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");
    make_file(tmp.path(), "src/newlayer/service.rs", "pub fn run() {}");

    let out = mille_add(tmp.path(), "src/newlayer", &[]);
    assert_eq!(exit_code(&out), 0, "stderr:\n{}", stderr(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    // Original domain layer must still be present
    assert!(
        content.contains("name = \"domain\""),
        "existing domain layer should be preserved\n{}",
        content
    );
    assert!(
        content.contains("src/domain/**"),
        "existing domain path should be preserved\n{}",
        content
    );
}

#[test]
fn test_add_preserves_resolve_section() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), config_with_resolve());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");
    make_file(tmp.path(), "src/newlayer/service.rs", "pub fn run() {}");

    let out = mille_add(tmp.path(), "src/newlayer", &[]);
    assert_eq!(exit_code(&out), 0, "stderr:\n{}", stderr(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("[resolve.rust]"),
        "[resolve.rust] section should be preserved\n{}",
        content
    );
    assert!(
        content.contains("package_names"),
        "package_names should be preserved\n{}",
        content
    );
}

#[test]
fn test_add_conflict_without_force_exits_error() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");

    // Try to add src/domain which already exists
    let out = mille_add(tmp.path(), "src/domain", &[]);
    assert_ne!(
        exit_code(&out),
        0,
        "should exit non-zero on conflict\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_add_conflict_with_force_replaces() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");

    let out = mille_add(tmp.path(), "src/domain", &["--force"]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille add --force should exit 0\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    // Layer should still exist (replaced, not duplicated)
    let count = content.matches("name = \"domain\"").count();
    assert_eq!(
        count, 1,
        "domain layer should appear exactly once after replacement\n{}",
        content
    );
}

#[test]
fn test_add_custom_name() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");
    make_file(tmp.path(), "src/newlayer/service.rs", "pub fn run() {}");

    let out = mille_add(tmp.path(), "src/newlayer", &["--name", "my_custom"]);
    assert_eq!(exit_code(&out), 0, "stderr:\n{}", stderr(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("name = \"my_custom\""),
        "custom name should be used\n{}",
        content
    );
}

#[test]
fn test_add_config_not_found() {
    let tmp = TempDir::new().unwrap();
    // No mille.toml created
    fs::create_dir_all(tmp.path().join("src/newlayer")).unwrap();

    let out = mille_add(tmp.path(), "src/newlayer", &[]);
    assert_eq!(
        exit_code(&out),
        3,
        "should exit 3 when config not found\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let err = stderr(&out);
    assert!(
        err.contains("not found"),
        "error should mention 'not found'\n{}",
        err
    );
}

#[test]
fn test_add_target_not_directory() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/afile.rs", "fn main() {}");

    let out = mille_add(tmp.path(), "src/afile.rs", &[]);
    assert_eq!(
        exit_code(&out),
        3,
        "should exit 3 when target is not a directory\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let err = stderr(&out);
    assert!(
        err.contains("not a directory"),
        "error should mention 'not a directory'\n{}",
        err
    );
}

// ---------------------------------------------------------------------------
// --depth tests
// ---------------------------------------------------------------------------

#[test]
fn test_add_depth_creates_multiple_layers() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");
    // Target dir with 3 subdirectories, each with source files
    make_file(tmp.path(), "lib/alpha/mod.rs", "pub fn a() {}");
    make_file(tmp.path(), "lib/beta/mod.rs", "use crate::alpha::a;");
    make_file(tmp.path(), "lib/gamma/mod.rs", "pub fn g() {}");

    let out = mille_add(tmp.path(), "lib", &["--depth", "1"]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille add --depth should exit 0\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    // Each subdirectory should be a separate layer
    assert!(
        content.contains("name = \"alpha\""),
        "alpha layer should be added\n{}",
        content
    );
    assert!(
        content.contains("name = \"beta\""),
        "beta layer should be added\n{}",
        content
    );
    assert!(
        content.contains("name = \"gamma\""),
        "gamma layer should be added\n{}",
        content
    );
    // Paths should include the target prefix
    assert!(
        content.contains("lib/alpha/**"),
        "alpha path should include target prefix\n{}",
        content
    );
}

#[test]
fn test_add_depth_preserves_existing_layers() {
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");
    make_file(tmp.path(), "lib/alpha/mod.rs", "pub fn a() {}");
    make_file(tmp.path(), "lib/beta/mod.rs", "pub fn b() {}");

    let out = mille_add(tmp.path(), "lib", &["--depth", "1"]);
    assert_eq!(exit_code(&out), 0, "stderr:\n{}", stderr(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("name = \"domain\""),
        "existing domain layer should be preserved\n{}",
        content
    );
    assert!(
        content.contains("src/domain/**"),
        "existing domain path should be preserved\n{}",
        content
    );
}

#[test]
fn test_add_depth_with_conflict_skips_overlapping() {
    let tmp = TempDir::new().unwrap();
    // Config already has lib/alpha as a layer
    let config = r#"[project]
name = "test_project"
root = "."
languages = ["lang_a"]

[[layers]]
name = "alpha"
paths = ["lib/alpha/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#;
    write_config(tmp.path(), config);
    make_file(tmp.path(), "lib/alpha/mod.rs", "pub fn a() {}");
    make_file(tmp.path(), "lib/beta/mod.rs", "pub fn b() {}");

    let out = mille_add(tmp.path(), "lib", &["--depth", "1"]);
    // Should exit 1 because of skipped overlapping layer
    assert_eq!(
        exit_code(&out),
        1,
        "should exit 1 when layers are skipped\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    // beta should be added
    assert!(
        content.contains("name = \"beta\""),
        "beta layer should be added\n{}",
        content
    );
    // alpha should appear only once (not duplicated)
    let alpha_count = content.matches("name = \"alpha\"").count();
    assert_eq!(
        alpha_count, 1,
        "alpha should appear exactly once (original, not duplicated)\n{}",
        content
    );
}

#[test]
fn test_add_depth_with_force_replaces_overlapping() {
    let tmp = TempDir::new().unwrap();
    let config = r#"[project]
name = "test_project"
root = "."
languages = ["lang_a"]

[[layers]]
name = "alpha"
paths = ["lib/alpha/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#;
    write_config(tmp.path(), config);
    make_file(tmp.path(), "lib/alpha/mod.rs", "pub fn a() {}");
    make_file(tmp.path(), "lib/beta/mod.rs", "pub fn b() {}");

    let out = mille_add(tmp.path(), "lib", &["--depth", "1", "--force"]);
    assert_eq!(
        exit_code(&out),
        0,
        "should exit 0 with --force\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("name = \"alpha\""),
        "alpha should exist\n{}",
        content
    );
    assert!(
        content.contains("name = \"beta\""),
        "beta should exist\n{}",
        content
    );
    let alpha_count = content.matches("name = \"alpha\"").count();
    assert_eq!(
        alpha_count, 1,
        "alpha should appear exactly once\n{}",
        content
    );
}

#[test]
fn test_add_without_depth_unchanged() {
    // Verify the original single-layer behavior still works
    let tmp = TempDir::new().unwrap();
    write_config(tmp.path(), base_config());
    make_file(tmp.path(), "src/domain/entity.rs", "pub struct User;");
    make_file(tmp.path(), "lib/alpha/mod.rs", "pub fn a() {}");
    make_file(tmp.path(), "lib/beta/mod.rs", "pub fn b() {}");

    // Without --depth: the whole lib dir is one layer
    let out = mille_add(tmp.path(), "lib", &[]);
    assert_eq!(exit_code(&out), 0, "stderr:\n{}", stderr(&out));

    let content = fs::read_to_string(tmp.path().join("mille.toml")).unwrap();
    assert!(
        content.contains("name = \"lib\""),
        "should add 'lib' as a single layer\n{}",
        content
    );
    assert!(
        content.contains("lib/**"),
        "path should be lib/**\n{}",
        content
    );
    // Should NOT have alpha/beta as separate layers
    assert!(
        !content.contains("name = \"alpha\""),
        "alpha should NOT be a separate layer without --depth\n{}",
        content
    );
}
