use crate::domain::entity::violation::{Severity, Violation, ViolationKind};

/// Format a single violation as a GitHub Actions annotation.
///
/// Output format: `::error file=<path>,line=<n>::<message>`
pub fn format_violation_ga(v: &Violation) -> String {
    let level = match v.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "notice",
    };
    let message = match v.kind {
        ViolationKind::DependencyViolation => format!(
            "Dependency violation: '{}' → '{}' is not allowed (import: {})",
            v.from_layer, v.to_layer, v.import_path
        ),
        ViolationKind::ExternalViolation => format!(
            "External violation: '{}' is not allowed in '{}' (import: {})",
            v.to_layer, v.from_layer, v.import_path
        ),
        ViolationKind::CallPatternViolation => format!(
            "Call pattern violation: '{}' is not in allow_methods (call: {})",
            v.to_layer, v.import_path
        ),
        ViolationKind::UnknownImport => format!(
            "Unknown import: '{}' could not be classified",
            v.import_path
        ),
        ViolationKind::NamingViolation => format!(
            "Naming violation: forbidden keyword '{}' found in {} (layer: '{}')",
            v.import_path, v.to_layer, v.from_layer
        ),
    };
    format!("::{} file={},line={}::{}\n", level, v.file, v.line, message)
}

/// Format all violations as GitHub Actions annotations.
///
/// Each violation becomes a `::error` or `::warning` annotation.
/// A `::notice::` summary line is always appended so that users get
/// confirmation even when there are no violations.
pub fn format_all_ga(violations: &[Violation]) -> String {
    let mut out: String = violations.iter().map(format_violation_ga).collect();
    let errors = violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warnings = violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();
    if errors == 0 && warnings == 0 {
        out.push_str("::notice::Architecture check passed: 0 violations\n");
    } else {
        out.push_str(&format!(
            "::notice::Architecture check: {} error(s), {} warning(s)\n",
            errors, warnings
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::violation::{Severity, Violation, ViolationKind};

    fn dep_violation() -> Violation {
        Violation {
            file: "src/domain/service/foo.rs".to_string(),
            line: 5,
            from_layer: "domain".to_string(),
            to_layer: "infrastructure".to_string(),
            import_path: "crate::infrastructure::db".to_string(),
            kind: ViolationKind::DependencyViolation,
            severity: Severity::Error,
        }
    }

    fn ext_violation() -> Violation {
        Violation {
            file: "src/usecase/order.rs".to_string(),
            line: 3,
            from_layer: "usecase".to_string(),
            to_layer: "sqlx".to_string(),
            import_path: "sqlx".to_string(),
            kind: ViolationKind::ExternalViolation,
            severity: Severity::Error,
        }
    }

    fn call_violation() -> Violation {
        Violation {
            file: "src/main.rs".to_string(),
            line: 15,
            from_layer: "main".to_string(),
            to_layer: "find_user".to_string(),
            import_path: "repo.find_user".to_string(),
            kind: ViolationKind::CallPatternViolation,
            severity: Severity::Error,
        }
    }

    #[test]
    fn test_ga_dependency_violation_format() {
        let out = format_violation_ga(&dep_violation());
        assert!(
            out.starts_with("::error "),
            "should start with ::error\nout: {out}"
        );
        assert!(
            out.contains("file=src/domain/service/foo.rs"),
            "should contain file path\nout: {out}"
        );
        assert!(
            out.contains("line=5"),
            "should contain line number\nout: {out}"
        );
        assert!(
            out.contains("domain"),
            "should contain from_layer\nout: {out}"
        );
        assert!(
            out.contains("infrastructure"),
            "should contain to_layer\nout: {out}"
        );
    }

    #[test]
    fn test_ga_external_violation_format() {
        let out = format_violation_ga(&ext_violation());
        assert!(
            out.starts_with("::error "),
            "should start with ::error\nout: {out}"
        );
        assert!(out.contains("file=src/usecase/order.rs"), "out: {out}");
        assert!(out.contains("line=3"), "out: {out}");
        assert!(
            out.contains("sqlx"),
            "should contain package name\nout: {out}"
        );
        assert!(out.contains("External violation"), "out: {out}");
    }

    #[test]
    fn test_ga_call_pattern_violation_format() {
        let out = format_violation_ga(&call_violation());
        assert!(out.starts_with("::error "), "out: {out}");
        assert!(out.contains("file=src/main.rs"), "out: {out}");
        assert!(out.contains("line=15"), "out: {out}");
        assert!(out.contains("Call pattern violation"), "out: {out}");
    }

    #[test]
    fn test_ga_warning_uses_warning_level() {
        let mut v = dep_violation();
        v.severity = Severity::Warning;
        let out = format_violation_ga(&v);
        assert!(out.starts_with("::warning "), "out: {out}");
    }

    #[test]
    fn test_ga_info_uses_notice_level() {
        let mut v = dep_violation();
        v.severity = Severity::Info;
        let out = format_violation_ga(&v);
        assert!(out.starts_with("::notice "), "out: {out}");
    }

    #[test]
    fn test_format_all_ga_empty_shows_notice() {
        let out = format_all_ga(&[]);
        assert!(
            out.contains("::notice::"),
            "no violations → should show ::notice:: summary\nout: {out}"
        );
        assert!(
            out.contains("passed"),
            "should confirm check passed\nout: {out}"
        );
    }

    #[test]
    fn test_format_all_ga_multiple_violations() {
        let violations = vec![dep_violation(), ext_violation()];
        let out = format_all_ga(&violations);
        let lines: Vec<&str> = out.lines().collect();
        // 2 violations + 1 summary notice
        assert_eq!(
            lines.len(),
            3,
            "should have 2 violation lines + 1 notice\nout: {out}"
        );
    }

    #[test]
    fn test_format_all_ga_summary_notice_shows_counts() {
        let violations = vec![dep_violation(), ext_violation()];
        let out = format_all_ga(&violations);
        assert!(
            out.contains("::notice::Architecture check:"),
            "summary notice must show error/warning counts\nout: {out}"
        );
    }
}
