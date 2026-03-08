use crate::domain::entity::violation::{Severity, Violation, ViolationKind};

/// A single violation in JSON-serialisable form.
#[derive(Debug)]
struct JsonViolation<'a> {
    severity: &'static str,
    rule: &'static str,
    file: &'a str,
    line: usize,
    from_layer: &'a str,
    to_layer: &'a str,
    import_path: &'a str,
}

impl<'a> JsonViolation<'a> {
    fn from_violation(v: &'a Violation) -> Self {
        let severity = match v.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        let rule = match v.kind {
            ViolationKind::DependencyViolation => "dependency",
            ViolationKind::ExternalViolation => "external",
            ViolationKind::CallPatternViolation => "call_pattern",
            ViolationKind::UnknownImport => "unknown_import",
        };
        Self {
            severity,
            rule,
            file: &v.file,
            line: v.line,
            from_layer: &v.from_layer,
            to_layer: &v.to_layer,
            import_path: &v.import_path,
        }
    }

    fn to_json(&self) -> String {
        format!(
            r#"    {{"severity":"{sev}","rule":"{rule}","file":"{file}","line":{line},"from_layer":"{from}","to_layer":"{to}","import":"{imp}"}}"#,
            sev = self.severity,
            rule = self.rule,
            file = self.file,
            line = self.line,
            from = self.from_layer,
            to = self.to_layer,
            imp = self.import_path,
        )
    }
}

/// Format all violations as a JSON document.
///
/// ```json
/// {"summary":{"errors":N,"warnings":N},"violations":[...]}
/// ```
pub fn format_json(violations: &[Violation]) -> String {
    let errors = violations
        .iter()
        .filter(|v| v.severity == Severity::Error)
        .count();
    let warnings = violations
        .iter()
        .filter(|v| v.severity == Severity::Warning)
        .count();

    let items: Vec<String> = violations
        .iter()
        .map(|v| JsonViolation::from_violation(v).to_json())
        .collect();

    format!(
        "{{\n  \"summary\":{{\"errors\":{errors},\"warnings\":{warnings}}},\n  \"violations\":[\n{items}\n  ]\n}}\n",
        errors = errors,
        warnings = warnings,
        items = items.join(",\n"),
    )
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
            severity: Severity::Warning,
        }
    }

    #[test]
    fn test_json_empty_violations() {
        let out = format_json(&[]);
        assert!(out.contains(r#""errors":0"#), "out: {out}");
        assert!(out.contains(r#""warnings":0"#), "out: {out}");
        assert!(out.contains(r#""violations":["#), "out: {out}");
    }

    #[test]
    fn test_json_error_count() {
        let violations = vec![dep_violation(), dep_violation()];
        let out = format_json(&violations);
        assert!(out.contains(r#""errors":2"#), "out: {out}");
        assert!(out.contains(r#""warnings":0"#), "out: {out}");
    }

    #[test]
    fn test_json_warning_count() {
        let violations = vec![ext_violation()];
        let out = format_json(&violations);
        assert!(out.contains(r#""errors":0"#), "out: {out}");
        assert!(out.contains(r#""warnings":1"#), "out: {out}");
    }

    #[test]
    fn test_json_violation_fields() {
        let out = format_json(&[dep_violation()]);
        assert!(out.contains(r#""severity":"error""#), "out: {out}");
        assert!(out.contains(r#""rule":"dependency""#), "out: {out}");
        assert!(
            out.contains(r#""file":"src/domain/service/foo.rs""#),
            "out: {out}"
        );
        assert!(out.contains(r#""line":5"#), "out: {out}");
        assert!(out.contains(r#""from_layer":"domain""#), "out: {out}");
        assert!(out.contains(r#""to_layer":"infrastructure""#), "out: {out}");
    }

    #[test]
    fn test_json_external_rule_name() {
        let out = format_json(&[ext_violation()]);
        assert!(out.contains(r#""rule":"external""#), "out: {out}");
        assert!(out.contains(r#""severity":"warning""#), "out: {out}");
    }

    #[test]
    fn test_json_output_is_valid_structure() {
        let violations = vec![dep_violation(), ext_violation()];
        let out = format_json(&violations);
        // Starts and ends with braces
        assert!(out.trim().starts_with('{'), "out: {out}");
        assert!(out.trim().ends_with('}'), "out: {out}");
        // Contains both violations
        assert!(out.contains("domain"), "out: {out}");
        assert!(out.contains("sqlx"), "out: {out}");
    }
}
