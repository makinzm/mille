use crate::domain::entity::violation::{Severity, Violation};
use crate::usecase::check_architecture::LayerStat;

/// Format a single violation as a human-readable string.
pub fn format_violation(v: &Violation) -> String {
    todo!()
}

/// Format per-layer file/violation statistics.
pub fn format_layer_stats(stats: &[LayerStat]) -> String {
    todo!()
}

/// Format the overall summary line (error/warning counts).
pub fn format_summary(violations: &[Violation]) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};
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
        assert!(out.contains("src/domain/service/foo.rs"), "should contain file path");
        assert!(out.contains('5'.to_string().as_str()), "should contain line number");
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
