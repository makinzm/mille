use crate::domain::entity::violation::{Severity, Violation, ViolationKind};
use crate::usecase::check_architecture::LayerStat;

/// Format a single violation as a human-readable string.
pub fn format_violation(v: &Violation) -> String {
    let marker = match v.severity {
        Severity::Error => "❌ [ERROR]",
        Severity::Warning => "⚠️  [WARN] ",
        Severity::Info => "ℹ️  [INFO] ",
    };
    match v.kind {
        ViolationKind::DependencyViolation => format!(
            "{} Dependency violation\n   {}:{}\n   import: {}\n   '{}' → '{}' is not allowed\n\n",
            marker, v.file, v.line, v.import_path, v.from_layer, v.to_layer
        ),
        ViolationKind::ExternalViolation => format!(
            "{} External violation\n   {}:{}\n   import: {}\n   '{}' is not allowed in '{}'\n\n",
            marker, v.file, v.line, v.import_path, v.to_layer, v.from_layer
        ),
        ViolationKind::CallPatternViolation => format!(
            "{} Call pattern violation\n   {}:{}\n   call: {}\n   '{}' is not in allow_methods\n\n",
            marker, v.file, v.line, v.import_path, v.to_layer
        ),
    }
}

/// Format per-layer file/violation statistics.
pub fn format_layer_stats(stats: &[LayerStat]) -> String {
    let mut out = String::new();
    for stat in stats {
        let marker = if stat.file_count == 0 {
            "⚠️ " // 0 files likely means the paths pattern matched nothing
        } else if stat.violation_count == 0 {
            "✅"
        } else {
            "❌"
        };
        out.push_str(&format!(
            "{} {:<20} ({:>3} file(s), {:>2} violation(s))\n",
            marker, stat.name, stat.file_count, stat.violation_count
        ));
    }
    out
}

/// Format the overall summary line (error/warning counts).
pub fn format_summary(violations: &[Violation]) -> String {
    let errors = violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warnings = violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();
    format!("Summary: {} error(s), {} warning(s)\n", errors, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::violation::{Severity, Violation, ViolationKind};
    use crate::usecase::check_architecture::LayerStat;

    fn error_violation(from: &str, to: &str, file: &str, line: usize) -> Violation {
        Violation {
            file: file.to_string(),
            line,
            from_layer: from.to_string(),
            to_layer: to.to_string(),
            import_path: format!("crate::{}::something", to),
            kind: ViolationKind::DependencyViolation,
            severity: Severity::Error,
        }
    }

    fn external_violation(layer: &str, pkg: &str, file: &str, line: usize) -> Violation {
        Violation {
            file: file.to_string(),
            line,
            from_layer: layer.to_string(),
            to_layer: pkg.to_string(),
            import_path: pkg.to_string(),
            kind: ViolationKind::ExternalViolation,
            severity: Severity::Error,
        }
    }

    // ------------------------------------------------------------------
    // format_violation
    // ------------------------------------------------------------------

    #[test]
    fn test_format_violation_contains_error_marker() {
        let v = error_violation("domain", "infrastructure", "src/domain/service/foo.rs", 5);
        let out = format_violation(&v);
        assert!(out.contains("❌"), "should contain error marker");
        assert!(out.contains("domain"), "should contain from_layer");
        assert!(out.contains("infrastructure"), "should contain to_layer");
        assert!(
            out.contains("src/domain/service/foo.rs"),
            "should contain file path"
        );
        assert!(
            out.contains('5'.to_string().as_str()),
            "should contain line number"
        );
    }

    #[test]
    fn test_format_external_violation_is_english() {
        let v = external_violation("main", "path/filepath", "main_test.go", 6);
        let out = format_violation(&v);
        assert!(out.contains("External violation"), "should contain kind");
        assert!(
            out.contains("is not allowed in"),
            "message must be in English, not Japanese\nout: {}",
            out
        );
        assert!(out.contains("path/filepath"), "should contain package name");
        assert!(out.contains("main"), "should contain layer name");
    }

    #[test]
    fn test_format_violation_contains_import_path() {
        let v = error_violation("domain", "infrastructure", "src/domain/foo.rs", 1);
        let out = format_violation(&v);
        assert!(
            out.contains("crate::infrastructure::something"),
            "should contain the import path"
        );
    }

    // ------------------------------------------------------------------
    // format_layer_stats
    // ------------------------------------------------------------------

    #[test]
    fn test_format_layer_stats_clean_layer() {
        let stats = vec![LayerStat {
            name: "domain".to_string(),
            file_count: 4,
            violation_count: 0,
        }];
        let out = format_layer_stats(&stats);
        assert!(out.contains("✅"), "clean layer should have ✅");
        assert!(out.contains("domain"));
        assert!(out.contains('4'.to_string().as_str()));
    }

    #[test]
    fn test_format_layer_stats_dirty_layer() {
        let stats = vec![LayerStat {
            name: "usecase".to_string(),
            file_count: 2,
            violation_count: 3,
        }];
        let out = format_layer_stats(&stats);
        assert!(out.contains("❌"), "layer with violations should have ❌");
        assert!(out.contains("usecase"));
    }

    #[test]
    fn test_format_layer_stats_zero_files_shows_warning() {
        let stats = vec![LayerStat {
            name: "main".to_string(),
            file_count: 0,
            violation_count: 0,
        }];
        let out = format_layer_stats(&stats);
        assert!(
            out.contains('⚠'),
            "layer with 0 files should show ⚠ (possible misconfigured paths)\nout: {}",
            out
        );
        assert!(
            !out.contains("✅"),
            "layer with 0 files must NOT show ✅\nout: {}",
            out
        );
    }

    // ------------------------------------------------------------------
    // format_summary
    // ------------------------------------------------------------------

    #[test]
    fn test_format_summary_zero_violations() {
        let out = format_summary(&[]);
        assert!(out.contains("0 error"), "should report 0 errors");
        assert!(out.contains("0 warning"), "should report 0 warnings");
    }

    #[test]
    fn test_format_summary_with_errors() {
        let violations = vec![
            error_violation("domain", "infrastructure", "src/domain/foo.rs", 1),
            error_violation("domain", "infrastructure", "src/domain/bar.rs", 2),
        ];
        let out = format_summary(&violations);
        assert!(out.contains("2 error"), "should report 2 errors");
    }
}
