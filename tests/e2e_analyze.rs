//! End-to-end tests for `mille analyze`.
//!
//! Verifies that the analyze subcommand outputs correct dependency graphs
//! in terminal, json, dot, and svg formats without applying violation rules.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn mille_analyze(dir: &Path, extra_args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mille"));
    cmd.arg("analyze");
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

fn make_file(base: &Path, rel: &str, content: &str) {
    let path = base.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// ---------------------------------------------------------------------------
// Fixture builder
//
// Creates a 3-layer Rust project:
//   src/domain/user.rs          → no internal imports
//   src/usecase/service.rs      → use crate::domain::user::User;
//   src/infrastructure/repo.rs  → use crate::domain::user::User;
//
// mille.toml defines the 3 layers with proper dependency rules.
// ---------------------------------------------------------------------------

fn setup_fixture(tmp: &TempDir) {
    make_file(
        tmp.path(),
        "src/domain/user.rs",
        "pub struct User { pub id: u64 }",
    );
    make_file(
        tmp.path(),
        "src/usecase/service.rs",
        "use crate::domain::user::User;\npub fn get_user() -> User { todo!() }",
    );
    make_file(
        tmp.path(),
        "src/infrastructure/repo.rs",
        "use crate::domain::user::User;\npub fn find() -> User { todo!() }",
    );
    make_file(
        tmp.path(),
        "mille.toml",
        r#"
[project]
name = "test-analyze"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
allow = []
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
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#,
    );
}

// ---------------------------------------------------------------------------
// JSON format
// ---------------------------------------------------------------------------

#[test]
fn test_analyze_json_valid_shape() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &["--format", "json"]);
    let s = stdout(&out);

    assert!(
        s.trim().starts_with('{'),
        "json output must start with '{{\nstdout:\n{s}\nstderr:\n{}",
        stderr(&out)
    );
    assert!(
        s.trim().ends_with('}'),
        "json output must end with '}}\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"nodes\""),
        "json output must contain 'nodes' key\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"edges\""),
        "json output must contain 'edges' key\nstdout:\n{s}"
    );
}

#[test]
fn test_analyze_json_has_layer_names() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &["--format", "json"]);
    let s = stdout(&out);

    assert!(
        s.contains("\"domain\""),
        "nodes must include 'domain'\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"usecase\""),
        "nodes must include 'usecase'\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"infrastructure\""),
        "nodes must include 'infrastructure'\nstdout:\n{s}"
    );
}

#[test]
fn test_analyze_json_has_edge() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &["--format", "json"]);
    let s = stdout(&out);

    // usecase and infrastructure both import from domain
    assert!(
        s.contains("\"from\""),
        "edges must include 'from' key\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"to\""),
        "edges must include 'to' key\nstdout:\n{s}"
    );
    assert!(
        s.contains("\"domain\""),
        "some edge must reference 'domain' as a target\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// DOT format
// ---------------------------------------------------------------------------

#[test]
fn test_analyze_dot_starts_with_digraph() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &["--format", "dot"]);
    let s = stdout(&out);

    assert!(
        s.trim().starts_with("digraph"),
        "dot output must start with 'digraph'\nstdout:\n{s}"
    );
}

#[test]
fn test_analyze_dot_has_node_and_edge() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &["--format", "dot"]);
    let s = stdout(&out);

    assert!(
        s.contains("\"domain\""),
        "dot output must define a 'domain' node\nstdout:\n{s}"
    );
    assert!(
        s.contains("->"),
        "dot output must contain at least one edge (->)\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// SVG format
// ---------------------------------------------------------------------------

#[test]
fn test_analyze_svg_is_valid_xml() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &["--format", "svg"]);
    let s = stdout(&out);

    assert!(
        s.contains("<svg"),
        "svg output must contain '<svg'\nstdout:\n{s}\nstderr:\n{}",
        stderr(&out)
    );
    assert!(
        s.contains("</svg>"),
        "svg output must contain '</svg>'\nstdout:\n{s}"
    );
}

#[test]
fn test_analyze_svg_has_layer_text() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &["--format", "svg"]);
    let s = stdout(&out);

    assert!(
        s.contains("domain"),
        "svg must contain layer name 'domain'\nstdout:\n{s}"
    );
    assert!(
        s.contains("usecase"),
        "svg must contain layer name 'usecase'\nstdout:\n{s}"
    );
}

#[test]
fn test_analyze_svg_has_edge_line() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &["--format", "svg"]);
    let s = stdout(&out);

    // SVG edges are rendered as <line> or <path> elements
    let has_edge = s.contains("<line") || s.contains("<path");
    assert!(
        has_edge,
        "svg must contain edge lines (<line> or <path>)\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// Terminal format (default)
// ---------------------------------------------------------------------------

#[test]
fn test_analyze_terminal_shows_layers() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    let out = mille_analyze(tmp.path(), &[]);
    let s = stdout(&out);

    assert!(
        s.contains("domain"),
        "terminal output must show 'domain'\nstdout:\n{s}"
    );
    assert!(
        s.contains("usecase"),
        "terminal output must show 'usecase'\nstdout:\n{s}"
    );
    assert!(
        s.contains("infrastructure"),
        "terminal output must show 'infrastructure'\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// Exit code
// ---------------------------------------------------------------------------

#[test]
fn test_analyze_exits_zero_always() {
    let tmp = TempDir::new().unwrap();
    setup_fixture(&tmp);

    // analyze never exits 1 — it only visualizes, does not enforce rules
    for format in &["terminal", "json", "dot", "svg"] {
        let out = mille_analyze(tmp.path(), &["--format", format]);
        assert_eq!(
            exit_code(&out),
            0,
            "mille analyze --format {format} must exit 0\nstdout:\n{}\nstderr:\n{}",
            stdout(&out),
            stderr(&out)
        );
    }
}
