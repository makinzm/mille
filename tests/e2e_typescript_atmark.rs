//! End-to-end tests for TypeScript path alias (`@/`) support.
//!
//! Verifies that `@/*` aliases defined in `tsconfig.json` are resolved to
//! internal imports, not treated as external packages.

use std::path::PathBuf;
use std::process::{Command, Output};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/typescript_atmark_example")
}

fn mille_in_fixture(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(fixture_dir())
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
// 1. Happy path: @/ aliases are resolved to internal imports
//
// `external_allow = []` means no external packages are allowed in usecase.
// If @/domain/user were treated as external, this would produce an ExternalViolation.
// After the fix, @/domain/user → internal (src/domain/user) → allowed by dep allow=["domain"].
// ---------------------------------------------------------------------------

#[test]
fn test_atmark_valid_exits_zero() {
    let out = mille_in_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "@/domain/user should be resolved as internal (not external)\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_atmark_valid_summary_shows_zero_errors() {
    let out = mille_in_fixture(&["check"]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary must show 0 errors, got:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// 2. Broken dep: usecase.allow=[] should detect @/domain/user as a dep violation
//
// With external_mode=opt-out (no external violation), and allow=[]:
// - Before fix: @/domain/user → external → no dep check → exit 0 (BUG: undetected)
// - After fix:  @/domain/user → internal (domain) → dep violation → exit 1
// ---------------------------------------------------------------------------

const BROKEN_DEP_TOML: &str = r#"
[project]
name = "typescript-atmark-example"
root = "."
languages = ["typescript"]

[resolve.typescript]
tsconfig = "./tsconfig.json"

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
allow = []
external_mode = "opt-out"
external_deny = []
"#;

#[test]
fn test_atmark_broken_dep_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_e2e_atmark_broken_dep.toml");
    std::fs::write(&config, BROKEN_DEP_TOML).expect("failed to write config");

    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_e2e_atmark_broken_dep.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "@/domain/user must be detected as dep violation when usecase.allow=[]\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}
