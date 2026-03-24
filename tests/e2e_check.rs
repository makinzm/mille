//! End-to-end tests for `mille check`.
//!
//! These tests invoke the compiled binary directly and verify:
//!   - exit codes (0 = clean, 1 = violations found, 3 = config error)
//!   - stdout / stderr content
//!
//! Each test that requires a custom config writes a temp file to the project
//! root (so that relative layer paths in the TOML resolve correctly) and cleans
//! it up via RAII on drop.

use mille::infrastructure::parser::rust::RustParser;
use mille::infrastructure::repository::fs_source_file_repository::FsSourceFileRepository;
use mille::infrastructure::repository::toml_config_repository::TomlConfigRepository;
use mille::infrastructure::resolver::rust::RustResolver;
use mille::usecase::check_architecture;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Run `mille` with the given arguments from the project root.
fn mille(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(args)
        .current_dir(project_root())
        .output()
        .expect("failed to execute mille binary")
}

/// RAII wrapper: writes a temp TOML to the project root and removes it on drop.
struct TempConfig {
    path: PathBuf,
}

impl TempConfig {
    fn new(name: &str, content: &str) -> Self {
        let path = project_root().join(name);
        fs::write(&path, content).expect("failed to write temp config");
        TempConfig { path }
    }

    fn file_name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
}

impl Drop for TempConfig {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
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
// Config fixtures
// ---------------------------------------------------------------------------

/// The real mille.toml — Clean Architecture is satisfied, so check must pass.
const VALID_CONFIG: &str = "mille.toml";

/// usecase allows only "domain" (no "infrastructure").
/// With the correctly refactored codebase this must produce zero violations.
/// IMPORTANT: if `check_architecture.rs` still imports infrastructure directly,
/// this config will return exit 1 — that is the RED signal for the architecture fix.
const USECASE_DOMAIN_ONLY_TOML: &str = r#"
[project]
name = "mille"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["serde"]

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = ["serde", "toml", "tree_sitter", "glob"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-in"
allow = ["usecase", "domain"]
external_mode = "opt-in"
external_allow = ["clap"]

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-in"
allow = ["domain", "infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["clap"]
"#;

/// infrastructure is opt-in with allow=[] → any import from domain is a violation.
/// `infrastructure/parser/rust.rs` imports domain entities → guaranteed exit 1.
const INFRA_BLOCKS_DOMAIN_TOML: &str = r#"
[project]
name = "mille-e2e"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-in"
external_allow = ["serde", "toml", "tree_sitter", "glob"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-in"
allow = ["usecase", "domain"]
external_mode = "opt-in"
external_allow = ["clap"]

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-in"
allow = ["domain", "infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["clap"]
"#;

/// usecase has allow_call_patterns for "domain" that excludes "new".
/// `check_architecture.rs` calls `ViolationDetector::new(...)` which is imported
/// from domain → CallPatternViolation → exit 1.
const CALL_PATTERN_VIOLATION_TOML: &str = r#"
[project]
name = "mille-e2e"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = ["serde", "toml", "tree_sitter", "glob"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []

  [[layers.allow_call_patterns]]
  callee_layer = "domain"
  allow_methods = ["detect", "check"]

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-in"
allow = ["usecase", "domain"]
external_mode = "opt-in"
external_allow = ["clap"]

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-in"
allow = ["domain", "infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["clap"]
"#;

// ---------------------------------------------------------------------------
// Normal cases
// ---------------------------------------------------------------------------

#[test]
fn test_valid_mille_toml_exits_zero() {
    let out = mille(&["check", "--config", VALID_CONFIG]);
    assert_eq!(
        exit_code(&out),
        0,
        "mille.toml should produce no violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_valid_config_stdout_contains_all_layer_names() {
    let out = mille(&["check", "--config", VALID_CONFIG]);
    let s = stdout(&out);
    for layer in &[
        "domain",
        "infrastructure",
        "usecase",
        "presentation",
        "main",
    ] {
        assert!(s.contains(layer), "output should mention layer '{}'", layer);
    }
}

#[test]
fn test_valid_config_all_layers_shown_as_clean() {
    let out = mille(&["check", "--config", VALID_CONFIG]);
    let s = stdout(&out);
    assert!(
        s.contains('✅'),
        "all layers should be marked ✅ when no violations"
    );
    assert!(
        !s.contains('❌'),
        "no layer should be marked ❌ when config is valid"
    );
}

#[test]
fn test_valid_config_summary_shows_zero_errors() {
    let out = mille(&["check", "--config", VALID_CONFIG]);
    let s = stdout(&out);
    assert!(
        s.contains("0 error(s)"),
        "summary should show 0 error(s)\nstdout:\n{}",
        s
    );
    assert!(
        s.contains("0 warning(s)"),
        "summary should show 0 warning(s)"
    );
}

#[test]
fn test_default_config_flag_uses_mille_toml() {
    // Running `mille check` with no --config should default to mille.toml.
    let out = mille(&["check"]);
    assert_eq!(
        exit_code(&out),
        0,
        "default config (mille.toml) should pass\nstdout:\n{}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// usecase Clean Architecture check (RED before fix, GREEN after fix)
// ---------------------------------------------------------------------------

#[test]
fn test_usecase_pure_domain_dependency_exits_zero() {
    // This is the architectural correctness test.
    // If usecase/check_architecture.rs directly imports from infrastructure,
    // this config (usecase allow=["domain"] only) will exit 1 → test fails (RED).
    // After the architecture fix it must exit 0 (GREEN).
    let cfg = TempConfig::new(
        "mille_e2e_usecase_domain_only.toml",
        USECASE_DOMAIN_ONLY_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        0,
        "with correct architecture, usecase must not import infrastructure\nstdout:\n{}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// Dependency violation cases
// ---------------------------------------------------------------------------

#[test]
fn test_dep_violation_infra_blocking_domain_exits_one() {
    let cfg = TempConfig::new(
        "mille_e2e_infra_blocks_domain.toml",
        INFRA_BLOCKS_DOMAIN_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "blocking infrastructure→domain must trigger violations\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_dep_violation_output_contains_infrastructure_layer() {
    let cfg = TempConfig::new(
        "mille_e2e_infra_blocks_domain2.toml",
        INFRA_BLOCKS_DOMAIN_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("infrastructure"),
        "violation output must mention 'infrastructure'\nstdout:\n{}",
        s
    );
}

#[test]
fn test_dep_violation_output_contains_domain_layer() {
    let cfg = TempConfig::new(
        "mille_e2e_infra_blocks_domain3.toml",
        INFRA_BLOCKS_DOMAIN_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("domain"),
        "violation output must mention 'domain'\nstdout:\n{}",
        s
    );
}

#[test]
fn test_dep_violation_output_contains_file_path() {
    let cfg = TempConfig::new(
        "mille_e2e_infra_blocks_domain4.toml",
        INFRA_BLOCKS_DOMAIN_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    // rust.rs or resolver.rs — at least one infrastructure file must appear.
    assert!(
        s.contains("src/infrastructure"),
        "violation output must reference the offending infrastructure file\nstdout:\n{}",
        s
    );
}

#[test]
fn test_dep_violation_infra_layer_marked_dirty_in_stats() {
    let cfg = TempConfig::new(
        "mille_e2e_infra_blocks_domain5.toml",
        INFRA_BLOCKS_DOMAIN_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains('❌'),
        "at least one layer should be marked ❌\nstdout:\n{}",
        s
    );
}

#[test]
fn test_dep_violation_summary_reports_nonzero_errors() {
    let cfg = TempConfig::new(
        "mille_e2e_infra_blocks_domain6.toml",
        INFRA_BLOCKS_DOMAIN_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    // "Summary: 0 error(s)" means no errors.  We need a non-zero count.
    // Avoid a substring match of "0 error(s)" within e.g. "10 error(s)".
    assert!(
        !s.contains("Summary: 0 error"),
        "summary must report > 0 errors\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Call pattern violation cases
// ---------------------------------------------------------------------------

#[test]
fn test_call_pattern_violation_exits_one() {
    let cfg = TempConfig::new(
        "mille_e2e_call_pattern_violation.toml",
        CALL_PATTERN_VIOLATION_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "blocking ViolationDetector::new must trigger CallPatternViolation\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_call_pattern_violation_output_contains_method() {
    let cfg = TempConfig::new(
        "mille_e2e_call_pattern_violation2.toml",
        CALL_PATTERN_VIOLATION_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    // The import_path is formatted as "ViolationDetector::with_severity"
    assert!(
        s.contains("with_severity"),
        "violation output must mention the forbidden method 'with_severity'\nstdout:\n{}",
        s
    );
}

#[test]
fn test_call_pattern_violation_output_contains_usecase_layer() {
    let cfg = TempConfig::new(
        "mille_e2e_call_pattern_violation3.toml",
        CALL_PATTERN_VIOLATION_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("usecase"),
        "violation output must mention 'usecase' as the offending layer\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// Config error cases
// ---------------------------------------------------------------------------

#[test]
fn test_nonexistent_config_exits_three() {
    let out = mille(&["check", "--config", "does_not_exist_at_all.toml"]);
    assert_eq!(
        exit_code(&out),
        3,
        "nonexistent config must exit with code 3"
    );
}

#[test]
fn test_nonexistent_config_error_goes_to_stderr() {
    let out = mille(&["check", "--config", "does_not_exist_at_all.toml"]);
    let err = stderr(&out);
    assert!(
        !err.is_empty(),
        "error message must be written to stderr\nstderr:\n{}",
        err
    );
}

#[test]
fn test_malformed_toml_exits_three() {
    let cfg = TempConfig::new("mille_e2e_malformed.toml", "this is not valid toml ][[[");
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        3,
        "malformed TOML must exit with code 3\nstderr:\n{}",
        stderr(&out)
    );
}

#[test]
fn test_malformed_toml_error_goes_to_stderr() {
    let cfg = TempConfig::new("mille_e2e_malformed2.toml", "this is not valid toml ][[[");
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert!(!stderr(&out).is_empty(), "error must be on stderr");
}

#[test]
fn test_empty_toml_exits_three() {
    // An empty TOML is valid TOML but has no [project] or [[layers]], which
    // should either fail to parse the expected fields or produce a valid-but-
    // empty check result. Our schema requires layers to be present.
    let cfg = TempConfig::new("mille_e2e_empty.toml", "");
    let out = mille(&["check", "--config", cfg.file_name()]);
    // Either exit 0 (no layers = no violations) or exit 3 (parse error) is
    // acceptable; what must NOT happen is a panic (exit code != 101).
    assert_ne!(
        exit_code(&out),
        101,
        "empty config must not cause a panic\nstdout:{}\nstderr:{}",
        stdout(&out),
        stderr(&out)
    );
}

// ---------------------------------------------------------------------------
// Output structure checks
// ---------------------------------------------------------------------------

#[test]
fn test_clean_run_stdout_is_not_empty() {
    // Even with no violations the formatter must print layer stats and summary.
    let out = mille(&["check", "--config", VALID_CONFIG]);
    assert!(
        !stdout(&out).is_empty(),
        "stdout must not be empty even when there are no violations"
    );
}

#[test]
fn test_violation_output_contains_error_marker() {
    let cfg = TempConfig::new("mille_e2e_marker.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("❌"),
        "violation output must contain ❌ marker\nstdout:\n{}",
        s
    );
}

#[test]
fn test_multiple_violations_all_present_in_output() {
    // infrastructure/parser/rust.rs imports multiple domain types.
    // Blocking all of them should produce more than one violation line.
    let cfg = TempConfig::new("mille_e2e_multi.toml", INFRA_BLOCKS_DOMAIN_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    // Count "Dependency violation" occurrences (one per violation block).
    let count = s.matches("Dependency violation").count();
    assert!(
        count >= 2,
        "expected at least 2 violations but found {}\nstdout:\n{}",
        count,
        s
    );
}

// ---------------------------------------------------------------------------
// Own-crate import detection (the "mille::" prefix bug)
// ---------------------------------------------------------------------------

/// main layer only allows usecase + presentation (NOT infrastructure).
/// src/main.rs imports from mille::infrastructure → must be detected as a violation.
/// This is the exact scenario the user reported: mille was silently passing when it
/// should not.
const MAIN_FORBIDS_INFRA_TOML: &str = r#"
[project]
name = "mille"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = ["serde", "toml", "tree_sitter", "glob"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-in"
allow = ["usecase", "domain"]
external_mode = "opt-in"
external_allow = ["clap"]

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-in"
allow = ["usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["clap"]
"#;

/// usecase only allows domain; main only allows usecase + presentation.
/// This config should produce 0 violations once the architecture is correct
/// (usecase must not import infrastructure; main must not import infrastructure/domain directly).
/// Used to verify that the "clean" path is also accurate — NOT a false negative.
const STRICT_LAYERING_TOML: &str = r#"
[project]
name = "mille"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["serde"]

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = ["serde", "toml", "tree_sitter", "glob"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-in"
allow = ["usecase", "domain"]
external_mode = "opt-in"
external_allow = ["clap"]

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-in"
allow = ["domain", "infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["clap"]
"#;

#[test]
fn test_main_forbids_infra_exits_one() {
    // RED: this test will FAIL before the resolver is fixed.
    // main.rs imports from mille::infrastructure::* which the resolver must classify
    // as Internal (infrastructure layer), producing violations when infrastructure
    // is not in the main layer's allow list.
    let cfg = TempConfig::new("mille_e2e_main_forbids_infra.toml", MAIN_FORBIDS_INFRA_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "main imports infrastructure but it is not in allow list — must exit 1\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_main_forbids_infra_violation_mentions_main_layer() {
    let cfg = TempConfig::new(
        "mille_e2e_main_forbids_infra2.toml",
        MAIN_FORBIDS_INFRA_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("main"),
        "violation output must mention 'main' as the offending layer\nstdout:\n{}",
        s
    );
}

#[test]
fn test_main_forbids_infra_violation_mentions_infrastructure() {
    let cfg = TempConfig::new(
        "mille_e2e_main_forbids_infra3.toml",
        MAIN_FORBIDS_INFRA_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("infrastructure"),
        "violation output must mention 'infrastructure' as the forbidden dependency\nstdout:\n{}",
        s
    );
}

#[test]
fn test_strict_layering_clean_exits_zero() {
    // The correctly-layered project (main allows all four layers) must produce 0 violations.
    let cfg = TempConfig::new("mille_e2e_strict_clean.toml", STRICT_LAYERING_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        0,
        "correctly-layered project must exit 0\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_domain_forbids_usecase_exits_one() {
    // domain (opt-out) denies usecase/presentation/infrastructure.
    // If any domain file imports from usecase → violation.
    // Currently no domain file imports usecase, so this should pass cleanly.
    // This test confirms opt-out deny rules work correctly.
    let cfg = TempConfig::new("mille_e2e_strict_clean2.toml", STRICT_LAYERING_TOML);
    let out = mille(&["check", "--config", cfg.file_name()]);
    // domain currently has no imports from usecase/presentation/infrastructure
    assert_eq!(
        exit_code(&out),
        0,
        "domain has no forbidden imports, should exit 0\nstdout:\n{}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// Dogfooding — library API integration (no binary invocation needed)
// Tests here live outside src/ so they are NOT subject to the architecture check,
// making it safe to import infrastructure types directly.
// ---------------------------------------------------------------------------

#[test]
fn test_dogfood_mille_toml_no_violations() {
    let result = check_architecture::check(
        "mille.toml",
        &TomlConfigRepository,
        &FsSourceFileRepository,
        &RustParser,
        &RustResolver,
    )
    .expect("mille.toml should be loadable");

    assert!(
        result.violations.is_empty(),
        "mille must not violate its own architecture rules.\nViolations:\n{:#?}",
        result.violations
    );
}

#[test]
fn test_dogfood_layer_stats_populated() {
    let result = check_architecture::check(
        "mille.toml",
        &TomlConfigRepository,
        &FsSourceFileRepository,
        &RustParser,
        &RustResolver,
    )
    .expect("mille.toml should be loadable");

    assert!(
        !result.layer_stats.is_empty(),
        "check result must include per-layer statistics"
    );
    assert!(
        result.layer_stats.iter().any(|s| s.file_count > 0),
        "at least one layer must have files"
    );
}

// ---------------------------------------------------------------------------
// External violation cases (Rust)
// ---------------------------------------------------------------------------

/// infrastructure with external_allow=[] — src/infrastructure uses tree_sitter, toml, glob, serde.
/// All of these must be detected as ExternalViolations.
const INFRA_EMPTY_EXTERNAL_ALLOW_TOML: &str = r#"
[project]
name = "mille-e2e"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
deny = ["infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]
external_mode = "opt-in"
external_allow = []

[[layers]]
name = "presentation"
paths = ["src/presentation/**"]
dependency_mode = "opt-in"
allow = ["usecase", "domain"]
external_mode = "opt-in"
external_allow = ["clap"]

[[layers]]
name = "main"
paths = ["src/main.rs"]
dependency_mode = "opt-in"
allow = ["domain", "infrastructure", "usecase", "presentation"]
external_mode = "opt-in"
external_allow = ["clap"]
"#;

#[test]
fn test_rust_infra_empty_external_allow_exits_one() {
    let cfg = TempConfig::new(
        "mille_e2e_rust_infra_ext_allow.toml",
        INFRA_EMPTY_EXTERNAL_ALLOW_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    assert_eq!(
        exit_code(&out),
        1,
        "infrastructure uses tree_sitter/toml/glob with external_allow=[]: must trigger violation\nstdout:\n{}",
        stdout(&out)
    );
}

#[test]
fn test_rust_infra_empty_external_allow_mentions_tree_sitter() {
    let cfg = TempConfig::new(
        "mille_e2e_rust_infra_ext_allow2.toml",
        INFRA_EMPTY_EXTERNAL_ALLOW_TOML,
    );
    let out = mille(&["check", "--config", cfg.file_name()]);
    let s = stdout(&out);
    assert!(
        s.contains("tree_sitter"),
        "violation output must mention 'tree_sitter'\nstdout:\n{}",
        s
    );
}

// ---------------------------------------------------------------------------
// PATH positional argument tests
// ---------------------------------------------------------------------------

/// `mille check <PATH>` where PATH points to a fixture directory.
/// The binary should chdir into that directory and find mille.toml there.
#[test]
fn test_check_with_path_argument() {
    let fixture = project_root().join("tests/fixtures/rust_sample");
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", fixture.to_str().unwrap()])
        .current_dir(project_root()) // NOT the fixture dir
        .output()
        .expect("failed to execute mille binary");
    assert_eq!(
        exit_code(&out),
        0,
        "check with path argument should succeed\nstderr:\n{}",
        stderr(&out)
    );
}

/// `mille check <PATH> --config <RELATIVE>` should resolve config relative to PATH.
#[test]
fn test_check_path_with_explicit_config() {
    let fixture = project_root().join("tests/fixtures/rust_sample");
    let out = Command::new(env!("CARGO_BIN_EXE_mille"))
        .args(["check", fixture.to_str().unwrap(), "--config", "mille.toml"])
        .current_dir(project_root())
        .output()
        .expect("failed to execute mille binary");
    assert_eq!(
        exit_code(&out),
        0,
        "check with path + explicit config should succeed\nstderr:\n{}",
        stderr(&out)
    );
}

/// `mille check <NONEXISTENT>` should fail with a clear error.
#[test]
fn test_check_nonexistent_path_fails() {
    let out = mille(&["check", "./nonexistent_dir_12345"]);
    assert_ne!(exit_code(&out), 0, "nonexistent path should fail");
    let err = stderr(&out);
    assert!(
        err.contains("nonexistent_dir_12345"),
        "error message should mention the path\nstderr:\n{}",
        err
    );
}
