/// E2E tests for `mille report external`.
///
/// Uses the existing `go_sample` fixture which has:
/// - infrastructure layer: imports `database/sql` (External)
/// - cmd layer: imports `fmt`, `os` (External in Go — stdlib is classified as External)
/// - domain, usecase layers: no external imports
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run_mille(args: &[&str], cwd: &PathBuf) -> std::process::Output {
    let binary = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/mille");
    std::process::Command::new(binary)
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("failed to run mille binary")
}

#[test]
fn test_e2e_report_external_terminal() {
    let cwd = fixture_path("go_sample");
    let output = run_mille(&["report", "external"], &cwd);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "exit code should be 0\nstdout: {stdout}"
    );
    // infrastructure layer should list database/sql
    assert!(
        stdout.contains("database/sql"),
        "expected database/sql in output\n{stdout}"
    );
    // cmd layer should list fmt and os
    assert!(stdout.contains("fmt"), "expected fmt in output\n{stdout}");
    assert!(stdout.contains("os"), "expected os in output\n{stdout}");
}

#[test]
fn test_e2e_report_external_json() {
    let cwd = fixture_path("go_sample");
    let output = run_mille(&["report", "external", "--format", "json"], &cwd);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "exit code should be 0\nstdout: {stdout}"
    );

    // Basic JSON structure check
    assert!(
        stdout.trim_start().starts_with('['),
        "JSON output should start with '['\n{stdout}"
    );
    assert!(
        stdout.contains("\"layer\""),
        "JSON should contain 'layer' key\n{stdout}"
    );
    assert!(
        stdout.contains("\"packages\""),
        "JSON should contain 'packages' key\n{stdout}"
    );
    assert!(
        stdout.contains("database/sql"),
        "JSON should include database/sql\n{stdout}"
    );
}

#[test]
fn test_e2e_report_external_no_external_layers() {
    let cwd = fixture_path("go_sample");
    let output = run_mille(&["report", "external"], &cwd);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "exit code should be 0\nstdout: {stdout}"
    );
    // domain and usecase layers have no external imports; (none) should appear
    assert!(
        stdout.contains("(none)"),
        "layers with no external imports should show (none)\n{stdout}"
    );
}

#[test]
fn test_e2e_report_external_output_file() {
    let cwd = fixture_path("go_sample");
    let out_file = cwd.join("_test_report_external_output.json");

    // Clean up in case a prior run left it
    let _ = std::fs::remove_file(&out_file);

    let output = run_mille(
        &[
            "report",
            "external",
            "--format",
            "json",
            "--output",
            "_test_report_external_output.json",
        ],
        &cwd,
    );
    assert!(output.status.success(), "exit code should be 0");
    assert!(out_file.exists(), "--output file should have been created");

    let content = std::fs::read_to_string(&out_file).unwrap();
    assert!(
        content.contains("database/sql"),
        "output file should contain database/sql"
    );

    std::fs::remove_file(&out_file).unwrap();
}
