//! End-to-end tests for `mille check` with TypeScript projects.
//!
//! Tests invoke the compiled binary against the `tests/fixtures/typescript_sample/` fixture
//! to verify TypeScript language support works correctly.

use std::path::PathBuf;
use std::process::{Command, Output};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/typescript_sample")
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
// 1. Happy path: valid TypeScript fixture
// ---------------------------------------------------------------------------

#[test]
fn test_ts_valid_config_exits_zero() {
    let out = mille_in_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "typescript_sample should produce no violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_ts_valid_config_summary_shows_zero_errors() {
    let out = mille_in_fixture(&["check"]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary must show 0 errors, got:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// 2. dep opt-in broken: usecase.allow = [] → usecase → domain 違反
// ---------------------------------------------------------------------------

const BROKEN_DEP_OPTIN_TOML: &str = r#"
[project]
name = "typescript-sample"
root = "."
languages = ["typescript"]

[resolve.typescript]
tsconfig = "./tsconfig.json"

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
external_mode = "opt-in"
external_allow = ["some-lib"]

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;

#[test]
fn test_ts_broken_usecase_allow_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_dep_optin.toml");
    std::fs::write(&config, BROKEN_DEP_OPTIN_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_dep_optin.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    assert_eq!(
        exit_code(&out),
        1,
        "usecase.allow=[] should produce violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_ts_broken_usecase_allow_mentions_usecase() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_dep_optin2.toml");
    std::fs::write(&config, BROKEN_DEP_OPTIN_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_dep_optin2.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    let s = stdout(&out);
    assert!(
        s.contains("usecase"),
        "output must mention 'usecase', got:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// 3. dep opt-out broken: infrastructure.deny = ["domain"] → violation
// ---------------------------------------------------------------------------

const BROKEN_DEP_OPTOUT_TOML: &str = r#"
[project]
name = "typescript-sample"
root = "."
languages = ["typescript"]

[resolve.typescript]
tsconfig = "./tsconfig.json"

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
external_mode = "opt-in"
external_allow = ["some-lib"]

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = ["domain"]
external_mode = "opt-out"
external_deny = []
"#;

#[test]
fn test_ts_broken_infra_deny_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_dep_optout.toml");
    std::fs::write(&config, BROKEN_DEP_OPTOUT_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_dep_optout.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    assert_eq!(
        exit_code(&out),
        1,
        "infrastructure.deny=[domain] should produce violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_ts_broken_infra_deny_mentions_infrastructure() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_dep_optout2.toml");
    std::fs::write(&config, BROKEN_DEP_OPTOUT_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_dep_optout2.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    let s = stdout(&out);
    assert!(
        s.contains("infrastructure"),
        "output must mention 'infrastructure', got:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// 4. external opt-in broken: usecase.external_allow = [] → "some-lib" 違反
// ---------------------------------------------------------------------------

const BROKEN_EXT_OPTIN_TOML: &str = r#"
[project]
name = "typescript-sample"
root = "."
languages = ["typescript"]

[resolve.typescript]
tsconfig = "./tsconfig.json"

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
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;

#[test]
fn test_ts_broken_external_optin_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_ext_optin.toml");
    std::fs::write(&config, BROKEN_EXT_OPTIN_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_ext_optin.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    assert_eq!(
        exit_code(&out),
        1,
        "usecase.external_allow=[] should produce violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_ts_broken_external_optin_mentions_usecase() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_ext_optin2.toml");
    std::fs::write(&config, BROKEN_EXT_OPTIN_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_ext_optin2.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    let s = stdout(&out);
    assert!(
        s.contains("usecase"),
        "output must mention 'usecase', got:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// 5. external opt-out broken: infrastructure.external_deny = ["node:fs"] → violation
// ---------------------------------------------------------------------------

const BROKEN_EXT_OPTOUT_TOML: &str = r#"
[project]
name = "typescript-sample"
root = "."
languages = ["typescript"]

[resolve.typescript]
tsconfig = "./tsconfig.json"

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
external_mode = "opt-in"
external_allow = ["some-lib"]

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = ["node:fs"]
"#;

#[test]
fn test_ts_broken_external_optout_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_ext_optout.toml");
    std::fs::write(&config, BROKEN_EXT_OPTOUT_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_ext_optout.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    assert_eq!(
        exit_code(&out),
        1,
        "infrastructure.external_deny=[node:fs] should produce violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_ts_broken_external_optout_mentions_infrastructure() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_ext_optout2.toml");
    std::fs::write(&config, BROKEN_EXT_OPTOUT_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_ext_optout2.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    let s = stdout(&out);
    assert!(
        s.contains("infrastructure"),
        "output must mention 'infrastructure', got:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// allow_call_patterns: forbidden static method call on domain class
// ---------------------------------------------------------------------------

/// main/app.ts calls User.create() but allow_methods=[] → CallPatternViolation.
const TS_BROKEN_CALL_PATTERN_TOML: &str = r#"
[project]
name = "typescript-sample"
root = "."
languages = ["typescript"]

[resolve.typescript]
tsconfig = "./tsconfig.json"

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
external_mode = "opt-in"
external_allow = ["some-lib"]

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
fn test_ts_broken_call_pattern_exits_one() {
    let config = fixture_dir().join("mille_e2e_ts_call_pattern.toml");
    std::fs::write(&config, TS_BROKEN_CALL_PATTERN_TOML).expect("failed to write config");

    let out = mille_in_fixture(&["check", "--config", "mille_e2e_ts_call_pattern.toml"]);
    std::fs::remove_file(&config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "User.create() is forbidden (allow_methods=[]): must trigger CallPatternViolation\nstdout:\n{}",
        stdout(&out)
    );
}
