//! End-to-end tests for `mille check` with Elixir projects.
//!
//! Tests invoke the compiled binary against the `tests/fixtures/elixir_sample/` fixture
//! to verify Elixir language support works correctly.

use std::path::PathBuf;
use std::process::{Command, Output};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn elixir_fixture_dir() -> PathBuf {
    project_root().join("tests/fixtures/elixir_sample")
}

/// Run `mille check` from the Elixir fixture directory.
fn mille_in_elixir_fixture(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(elixir_fixture_dir())
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
// Happy path: valid Elixir fixture
// ---------------------------------------------------------------------------

#[test]
fn test_elixir_valid_config_exits_zero() {
    let out = mille_in_elixir_fixture(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "elixir_sample mille.toml should produce no violations\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_elixir_valid_config_summary_shows_zero_errors() {
    let out = mille_in_elixir_fixture(&["check"]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary must show 0 errors, got:\n{s}"
    );
}

#[test]
fn test_elixir_valid_config_all_layers_clean() {
    let out = mille_in_elixir_fixture(&["check"]);
    let s = stdout(&out);
    assert!(s.contains("domain"), "output must list domain layer");
    assert!(s.contains("usecase"), "output must list usecase layer");
    assert!(
        s.contains("infrastructure"),
        "output must list infrastructure layer"
    );
}

// ---------------------------------------------------------------------------
// Broken fixture: usecase has allow=[] (domain not allowed)
// ---------------------------------------------------------------------------

#[test]
fn test_elixir_broken_usecase_exits_one() {
    let broken_config = elixir_fixture_dir().join("mille_broken_usecase.toml");
    let config_content = r#"
[project]
name = "elixir-sample"
root = "."
languages = ["elixir"]

[resolve.elixir]
app_name = "MyApp"

[[layers]]
name = "domain"
paths = ["lib/domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["lib/usecase/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["lib/infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_elixir_fixture(&["check", "--config", "mille_broken_usecase.toml"]);
    std::fs::remove_file(&broken_config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "broken usecase config must exit 1\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

#[test]
fn test_elixir_broken_usecase_mentions_usecase() {
    let broken_config = elixir_fixture_dir().join("mille_broken_usecase2.toml");
    let config_content = r#"
[project]
name = "elixir-sample"
root = "."
languages = ["elixir"]

[resolve.elixir]
app_name = "MyApp"

[[layers]]
name = "domain"
paths = ["lib/domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["lib/usecase/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["lib/infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_elixir_fixture(&["check", "--config", "mille_broken_usecase2.toml"]);
    std::fs::remove_file(&broken_config).ok();

    let s = stdout(&out);
    assert!(
        s.contains("usecase"),
        "violation output must mention 'usecase' layer\nstdout:\n{s}"
    );
}

// ---------------------------------------------------------------------------
// Broken fixture: infrastructure denies domain
// ---------------------------------------------------------------------------

#[test]
fn test_elixir_broken_infra_deny_domain_exits_one() {
    let broken_config = elixir_fixture_dir().join("mille_broken_infra_deny.toml");
    let config_content = r#"
[project]
name = "elixir-sample"
root = "."
languages = ["elixir"]

[resolve.elixir]
app_name = "MyApp"

[[layers]]
name = "domain"
paths = ["lib/domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["lib/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["lib/infrastructure/**"]
dependency_mode = "opt-out"
deny = ["domain"]
external_mode = "opt-out"
external_deny = []
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_elixir_fixture(&["check", "--config", "mille_broken_infra_deny.toml"]);
    std::fs::remove_file(&broken_config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "infrastructure deny=domain must exit 1\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}

// ---------------------------------------------------------------------------
// Broken fixture: domain has external_deny=["Ecto"]
// ---------------------------------------------------------------------------

#[test]
fn test_elixir_broken_external_deny_exits_one() {
    let broken_config = elixir_fixture_dir().join("mille_broken_ext_deny.toml");
    // infrastructure/repo.ex has `alias Ecto.Repo` — this is an external import
    // Set external_deny=["Ecto"] on infrastructure to trigger a violation
    let config_content = r#"
[project]
name = "elixir-sample"
root = "."
languages = ["elixir"]

[resolve.elixir]
app_name = "MyApp"

[[layers]]
name = "domain"
paths = ["lib/domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "usecase"
paths = ["lib/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-out"
external_deny = []

[[layers]]
name = "infrastructure"
paths = ["lib/infrastructure/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = ["Ecto"]
"#;
    std::fs::write(&broken_config, config_content).expect("failed to write broken config");

    let out = mille_in_elixir_fixture(&["check", "--config", "mille_broken_ext_deny.toml"]);
    std::fs::remove_file(&broken_config).ok();

    assert_eq!(
        exit_code(&out),
        1,
        "external_deny=[Ecto] must exit 1\nstdout:\n{}\nstderr:\n{}",
        stdout(&out),
        stderr(&out)
    );
}
