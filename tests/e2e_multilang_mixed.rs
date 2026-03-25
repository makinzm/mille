//! End-to-end tests for `mille check` with a multi-language project
//! where different languages coexist in the same layer directories.
//!
//! Fixture: `tests/fixtures/multilang_mixed_sample/`
//! Languages: TypeScript + Python + Go (mixed in domain/, usecase/, infrastructure/, main/)

use std::path::PathBuf;
use std::process::{Command, Output};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/multilang_mixed_sample")
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
// 1. Happy path: valid multi-language fixture
// ---------------------------------------------------------------------------

#[test]
fn test_multilang_mixed_valid_config_exits_zero() {
    let out = mille_in_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "multilang_mixed_sample should produce no violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_multilang_mixed_valid_config_summary_shows_zero_errors() {
    let out = mille_in_fixture(&["check"]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary must show 0 errors, got:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// 2. dep opt-in broken: usecase.allow = [] -> domain dependency violation
// ---------------------------------------------------------------------------

const MIXED_BROKEN_DEP_OPTIN_TOML: &str = r#"
[project]
name = "multilang-mixed"
root = "."
languages = ["typescript", "python", "go"]

[resolve.go]
module_name = "github.com/example/multilang-mixed"

[resolve.python]
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

[[layers]]
name = "main"
paths = ["main/**"]
dependency_mode = "opt-in"
allow = ["domain", "usecase", "infrastructure"]
external_mode = "opt-out"
external_deny = []

  [[layers.allow_call_patterns]]
  callee_layer = "domain"
  allow_methods = ["create", "NewUser"]
"#;

#[test]
fn test_multilang_mixed_broken_dep_optin_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_dep_optin.toml");
    std::fs::write(&config, MIXED_BROKEN_DEP_OPTIN_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_dep_optin.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    assert_eq!(
        exit_code(&out),
        1,
        "usecase.allow=[] should produce violations across TS/PY/Go\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_multilang_mixed_broken_dep_optin_mentions_usecase() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_dep_optin2.toml");
    std::fs::write(&config, MIXED_BROKEN_DEP_OPTIN_TOML).unwrap();
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
// 3. dep opt-out broken: infrastructure.deny = ["domain"] -> violation
// ---------------------------------------------------------------------------

const MIXED_BROKEN_DEP_OPTOUT_TOML: &str = r#"
[project]
name = "multilang-mixed"
root = "."
languages = ["typescript", "python", "go"]

[resolve.go]
module_name = "github.com/example/multilang-mixed"

[resolve.python]
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
external_mode = "opt-in"
external_allow = ["some-lib"]

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = ["domain"]
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
  allow_methods = ["create", "NewUser"]
"#;

#[test]
fn test_multilang_mixed_broken_dep_optout_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_dep_optout.toml");
    std::fs::write(&config, MIXED_BROKEN_DEP_OPTOUT_TOML).unwrap();
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
fn test_multilang_mixed_broken_dep_optout_mentions_infrastructure() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_dep_optout2.toml");
    std::fs::write(&config, MIXED_BROKEN_DEP_OPTOUT_TOML).unwrap();
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
// 4. external opt-in broken: domain.external_allow = [] -> violation (Python's `import os`)
// ---------------------------------------------------------------------------

const MIXED_BROKEN_EXT_OPTIN_TOML: &str = r#"
[project]
name = "multilang-mixed"
root = "."
languages = ["typescript", "python", "go"]

[resolve.go]
module_name = "github.com/example/multilang-mixed"

[resolve.python]
package_names = ["domain", "usecase", "infrastructure", "main"]

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-in"
external_allow = []

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
  allow_methods = ["create", "NewUser"]
"#;

#[test]
fn test_multilang_mixed_broken_external_optin_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_ext_optin.toml");
    std::fs::write(&config, MIXED_BROKEN_EXT_OPTIN_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_ext_optin.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    assert_eq!(
        exit_code(&out),
        1,
        "domain.external_allow=[] should catch Python's `import os`\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_multilang_mixed_broken_external_optin_mentions_domain() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_ext_optin2.toml");
    std::fs::write(&config, MIXED_BROKEN_EXT_OPTIN_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_ext_optin2.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    let s = stdout(&out);
    assert!(
        s.contains("domain"),
        "output must mention 'domain', got:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// 5. external opt-out broken: infrastructure.external_deny blocks all external libs
// ---------------------------------------------------------------------------

const MIXED_BROKEN_EXT_OPTOUT_TOML: &str = r#"
[project]
name = "multilang-mixed"
root = "."
languages = ["typescript", "python", "go"]

[resolve.go]
module_name = "github.com/example/multilang-mixed"

[resolve.python]
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
external_mode = "opt-in"
external_allow = ["some-lib"]

[[layers]]
name = "infrastructure"
paths = ["infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = ["node:fs", "os", "database/sql"]

[[layers]]
name = "main"
paths = ["main/**"]
dependency_mode = "opt-in"
allow = ["domain", "usecase", "infrastructure"]
external_mode = "opt-out"
external_deny = []

  [[layers.allow_call_patterns]]
  callee_layer = "domain"
  allow_methods = ["create", "NewUser"]
"#;

#[test]
fn test_multilang_mixed_broken_external_optout_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_ext_optout.toml");
    std::fs::write(&config, MIXED_BROKEN_EXT_OPTOUT_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_ext_optout.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    assert_eq!(
        exit_code(&out),
        1,
        "infrastructure.external_deny should catch node:fs/os/database/sql\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_multilang_mixed_broken_external_optout_mentions_infrastructure() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_ext_optout2.toml");
    std::fs::write(&config, MIXED_BROKEN_EXT_OPTOUT_TOML).unwrap();
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
// 6. allow_call_patterns broken: main.allow_methods = [] -> CallPatternViolation
// ---------------------------------------------------------------------------

const MIXED_BROKEN_CALL_PATTERN_TOML: &str = r#"
[project]
name = "multilang-mixed"
root = "."
languages = ["typescript", "python", "go"]

[resolve.go]
module_name = "github.com/example/multilang-mixed"

[resolve.python]
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
fn test_multilang_mixed_broken_call_pattern_exits_one() {
    let dir = fixture_dir();
    let config = dir.join("mille_broken_call_pattern.toml");
    std::fs::write(&config, MIXED_BROKEN_CALL_PATTERN_TOML).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", "--config", "mille_broken_call_pattern.toml"])
        .current_dir(&dir)
        .output()
        .expect("failed to execute mille binary");
    std::fs::remove_file(&config).ok();
    assert_eq!(
        exit_code(&out),
        1,
        "allow_methods=[] should trigger CallPatternViolation for create/NewUser\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}
