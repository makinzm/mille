use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::config::SeverityConfig;
use crate::domain::entity::import::ImportKind;
use crate::domain::entity::layer::{DependencyMode, LayerConfig};
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::entity::violation::{Severity, Violation, ViolationKind};

pub struct ViolationDetector<'a> {
    layers: &'a [LayerConfig],
    severity: SeverityConfig,
}

impl<'a> ViolationDetector<'a> {
    /// Create a detector with default severity (all violations = Error, unknown_import = Warning).
    pub fn new(layers: &'a [LayerConfig]) -> Self {
        Self {
            layers,
            severity: SeverityConfig::default(),
        }
    }

    /// Create a detector with explicit severity configuration.
    pub fn with_severity(layers: &'a [LayerConfig], severity: SeverityConfig) -> Self {
        Self { layers, severity }
    }

    /// Inspect a list of resolved imports and return all dependency violations.
    pub fn detect(&self, imports: &[ResolvedImport]) -> Vec<Violation> {
        let mut violations = Vec::new();
        for import in imports {
            if import.category != ImportCategory::Internal {
                continue;
            }
            let Some(from_layer) = self.find_layer_for_file(&import.raw.file) else {
                continue;
            };
            let Some(resolved) = &import.resolved_path else {
                continue;
            };
            let Some(to_layer) = self.find_layer_for_file(resolved) else {
                continue;
            };
            if from_layer.name == to_layer.name {
                continue;
            }
            if let Some(v) = self.check_violation(import, from_layer, to_layer) {
                violations.push(v);
            }
        }
        violations
    }

    /// Return the first layer whose `paths` glob patterns match `file_path`.
    fn find_layer_for_file(&self, file_path: &str) -> Option<&LayerConfig> {
        self.layers.iter().find(|layer| {
            layer.paths.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .ok()
                    .map(|p| p.matches(file_path))
                    .unwrap_or(false)
            })
        })
    }

    /// Inspect a list of resolved imports and return all external-dependency violations.
    ///
    /// For each `ImportCategory::External` import, the crate name (first `::` component) is
    /// matched against `external_allow` (opt-in) or `external_deny` (opt-out) using the patterns
    /// as full-string regular expressions.
    pub fn detect_external(&self, imports: &[ResolvedImport]) -> Vec<Violation> {
        let mut violations = Vec::new();
        for import in imports {
            if import.category != ImportCategory::External {
                continue;
            }
            // `mod X;` declarations are internal module structure declarations, not external
            // library imports. They must not be checked against external_allow/external_deny.
            if import.raw.kind == ImportKind::Mod {
                continue;
            }
            let Some(from_layer) = self.find_layer_for_file(&import.raw.file) else {
                continue;
            };
            let crate_name = import
                .raw
                .path
                .split("::")
                .next()
                .unwrap_or(&import.raw.path);
            let allowed = match from_layer.external_mode {
                DependencyMode::OptIn => from_layer
                    .external_allow
                    .iter()
                    .any(|p| matches_external_pattern(p, crate_name)),
                DependencyMode::OptOut => !from_layer
                    .external_deny
                    .iter()
                    .any(|p| matches_external_pattern(p, crate_name)),
            };
            if !allowed {
                violations.push(Violation {
                    file: import.raw.file.clone(),
                    line: import.raw.line,
                    from_layer: from_layer.name.clone(),
                    to_layer: crate_name.to_string(),
                    import_path: import.raw.path.clone(),
                    kind: ViolationKind::ExternalViolation,
                    severity: parse_severity(&self.severity.external_violation),
                });
            }
        }
        violations
    }

    /// Check `allow_call_patterns` rules: for each static call (`Type::method()`) in `call_exprs`,
    /// resolve the receiver type to its layer via `resolved_imports` and flag calls whose method
    /// is not in the caller layer's `allow_methods` list for that `callee_layer`.
    ///
    /// Instance method calls (`var.method()`, `receiver_type == None`) are skipped because their
    /// type cannot be determined without type inference.
    pub fn detect_call_patterns(
        &self,
        call_exprs: &[RawCallExpr],
        resolved_imports: &[ResolvedImport],
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        for call in call_exprs {
            // Only static calls with a known receiver type can be checked.
            let Some(receiver_type) = &call.receiver_type else {
                continue;
            };

            let Some(from_layer) = self.find_layer_for_file(&call.file) else {
                continue;
            };

            if from_layer.allow_call_patterns.is_empty() {
                continue;
            }

            // Collect imports in this file that resolve to each callee layer.
            for pattern in &from_layer.allow_call_patterns {
                let type_is_from_callee = resolved_imports.iter().any(|imp| {
                    imp.raw.file == call.file
                        && imp.category == ImportCategory::Internal
                        && imp
                            .resolved_path
                            .as_deref()
                            .and_then(|rp| self.find_layer_for_file(rp))
                            .map(|l| l.name == pattern.callee_layer)
                            .unwrap_or(false)
                        && (
                            // Rust / Go: type name embedded in import path
                            type_name_from_import(&imp.raw.path)
                                .map(|n| n == receiver_type.as_str())
                                .unwrap_or(false)
                            // Python / TypeScript: named imports tracked explicitly
                            || imp.raw.named_imports.iter().any(|n| n == receiver_type.as_str())
                        )
                });

                if !type_is_from_callee {
                    continue;
                }

                if !pattern.allow_methods.contains(&call.method) {
                    violations.push(Violation {
                        file: call.file.clone(),
                        line: call.line,
                        from_layer: from_layer.name.clone(),
                        to_layer: pattern.callee_layer.clone(),
                        import_path: format!("{}::{}", receiver_type, call.method),
                        kind: ViolationKind::CallPatternViolation,
                        severity: parse_severity(&self.severity.call_pattern_violation),
                    });
                }
            }
        }

        violations
    }

    /// Report all `ImportCategory::Unknown` imports from files that belong to a known layer.
    ///
    /// These are imports the resolver could not classify (e.g. ambiguous module paths).
    /// Severity is controlled by `severity.unknown_import` in mille.toml.
    pub fn detect_unknown(&self, imports: &[ResolvedImport]) -> Vec<Violation> {
        let sev = parse_severity(&self.severity.unknown_import);
        let mut violations = Vec::new();
        for import in imports {
            if import.category != ImportCategory::Unknown {
                continue;
            }
            let Some(from_layer) = self.find_layer_for_file(&import.raw.file) else {
                continue;
            };
            violations.push(Violation {
                file: import.raw.file.clone(),
                line: import.raw.line,
                from_layer: from_layer.name.clone(),
                to_layer: String::new(),
                import_path: import.raw.path.clone(),
                kind: ViolationKind::UnknownImport,
                severity: sev.clone(),
            });
        }
        violations
    }

    /// Check whether the dependency `from → to` is permitted.
    /// Returns `Some(Violation)` if it is not.
    fn check_violation(
        &self,
        import: &ResolvedImport,
        from: &LayerConfig,
        to: &LayerConfig,
    ) -> Option<Violation> {
        // Imports within the same layer are always allowed.
        if from.name == to.name {
            return None;
        }
        let allowed = match from.dependency_mode {
            DependencyMode::OptIn => from.allow.contains(&to.name),
            DependencyMode::OptOut => !from.deny.contains(&to.name),
        };
        if allowed {
            return None;
        }
        Some(Violation {
            file: import.raw.file.clone(),
            line: import.raw.line,
            from_layer: from.name.clone(),
            to_layer: to.name.clone(),
            import_path: import.raw.path.clone(),
            kind: ViolationKind::DependencyViolation,
            severity: parse_severity(&self.severity.dependency_violation),
        })
    }
}

/// Parse a severity string from config into a `Severity` enum value.
///
/// Invalid or unknown strings default to `Severity::Error` (safe default).
fn parse_severity(s: &str) -> Severity {
    match s {
        "warning" => Severity::Warning,
        "info" => Severity::Info,
        _ => Severity::Error,
    }
}

/// Match `crate_name` against a pattern using exact string equality.
/// Users write patterns as plain strings (e.g. `"github.com/foo/bar"`), no regex escaping needed.
fn matches_external_pattern(pattern: &str, crate_name: &str) -> bool {
    pattern == crate_name
}

/// Extract the type/package name brought into scope by an import path.
///
/// - Rust:  `"crate::infrastructure::Repo"` → `Some("Repo")`  (split by `::`)
/// - Go:    `"github.com/example/gosample/domain"` → `Some("domain")`  (split by `/`)
/// - Returns `None` for wildcards (`*`) and grouped imports (`{…}`).
///
/// Python and TypeScript named imports are checked via `named_imports` field directly.
fn type_name_from_import(path: &str) -> Option<&str> {
    // Rust-style paths use "::" separator.
    if path.contains("::") {
        let last = path.split("::").last()?;
        if last.starts_with('{') || last == "*" {
            return None;
        }
        return Some(last);
    }

    // Go-style paths use "/" separator (e.g. "github.com/foo/bar/domain").
    // The last segment is the package name used as the call receiver.
    if path.contains('/') {
        return path.split('/').last().filter(|s| !s.is_empty());
    }

    // Plain single-segment paths (e.g. "fmt", "os" in Go stdlib).
    if path.starts_with('{') || path == "*" {
        return None;
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};
    use crate::domain::entity::layer::DependencyMode;
    use crate::domain::entity::resolved_import::ImportCategory;

    fn make_layer(
        name: &str,
        paths: &[&str],
        mode: DependencyMode,
        allow: &[&str],
        deny: &[&str],
    ) -> LayerConfig {
        LayerConfig {
            name: name.to_string(),
            paths: paths.iter().map(|s| s.to_string()).collect(),
            dependency_mode: mode,
            allow: allow.iter().map(|s| s.to_string()).collect(),
            deny: deny.iter().map(|s| s.to_string()).collect(),
            external_mode: DependencyMode::OptIn,
            external_allow: vec![],
            external_deny: vec![],
            allow_call_patterns: vec![],
        }
    }

    fn make_internal(file: &str, line: usize, path: &str, resolved: &str) -> ResolvedImport {
        ResolvedImport {
            raw: RawImport {
                path: path.to_string(),
                line,
                file: file.to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::Internal,
            resolved_path: Some(resolved.to_string()),
        }
    }

    // ------------------------------------------------------------------
    // find_layer_for_file
    // ------------------------------------------------------------------

    #[test]
    fn test_find_layer_exact_glob() {
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);

        let found = detector.find_layer_for_file("src/domain/entity/config.rs");
        assert_eq!(found.map(|l| l.name.as_str()), Some("domain"));

        let found2 = detector.find_layer_for_file("src/infrastructure/parser/rust.rs");
        assert_eq!(found2.map(|l| l.name.as_str()), Some("infrastructure"));
    }

    #[test]
    fn test_find_layer_no_match_returns_none() {
        let layers = vec![make_layer(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);

        assert!(detector
            .find_layer_for_file("src/presentation/cli.rs")
            .is_none());
    }

    // ------------------------------------------------------------------
    // check_violation
    // ------------------------------------------------------------------

    #[test]
    fn test_opt_in_allowed_dependency_no_violation() {
        // infrastructure opt-in, allow = ["domain"]  →  infra→domain OK
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &["domain"],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let import = make_internal(
            "src/infrastructure/repo.rs",
            1,
            "crate::domain::entity::config",
            "src/domain/entity/config",
        );
        let from = &layers[1];
        let to = &layers[0];
        assert!(detector.check_violation(&import, from, to).is_none());
    }

    #[test]
    fn test_opt_in_disallowed_dependency_is_violation() {
        // domain opt-in, allow = []  →  domain→infrastructure VIOLATION
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &["domain"],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let import = make_internal(
            "src/domain/service/foo.rs",
            5,
            "crate::infrastructure::repo",
            "src/infrastructure/repo",
        );
        let from = &layers[0]; // domain
        let to = &layers[1]; // infrastructure
        let v = detector.check_violation(&import, from, to);
        assert!(v.is_some());
        let v = v.unwrap();
        assert_eq!(v.from_layer, "domain");
        assert_eq!(v.to_layer, "infrastructure");
        assert_eq!(v.kind, ViolationKind::DependencyViolation);
        assert_eq!(v.severity, Severity::Error);
    }

    #[test]
    fn test_opt_out_denied_dependency_is_violation() {
        // domain opt-out, deny = ["infrastructure"]  →  domain→infrastructure VIOLATION
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptOut,
                &[],
                &["infrastructure"],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &["domain"],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let import = make_internal(
            "src/domain/service/foo.rs",
            3,
            "crate::infrastructure::repo",
            "src/infrastructure/repo",
        );
        let from = &layers[0];
        let to = &layers[1];
        let v = detector.check_violation(&import, from, to);
        assert!(v.is_some());
        assert_eq!(v.unwrap().from_layer, "domain");
    }

    #[test]
    fn test_opt_out_allowed_dependency_no_violation() {
        // infrastructure opt-out, deny = []  →  infra→domain OK
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptOut,
                &[],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let import = make_internal(
            "src/infrastructure/repo.rs",
            1,
            "crate::domain::entity::config",
            "src/domain/entity/config",
        );
        let from = &layers[1];
        let to = &layers[0];
        assert!(detector.check_violation(&import, from, to).is_none());
    }

    #[test]
    fn test_same_layer_no_violation() {
        let layers = vec![make_layer(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let import = make_internal(
            "src/domain/service/foo.rs",
            1,
            "crate::domain::entity::config",
            "src/domain/entity/config",
        );
        let from = &layers[0];
        let to = &layers[0];
        assert!(detector.check_violation(&import, from, to).is_none());
    }

    // ------------------------------------------------------------------
    // detect (end-to-end)
    // ------------------------------------------------------------------

    #[test]
    fn test_detect_returns_only_internal_violations() {
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &["domain"],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);

        let imports = vec![
            // infra → domain: allowed
            make_internal(
                "src/infrastructure/repo.rs",
                1,
                "crate::domain::entity::config",
                "src/domain/entity/config",
            ),
            // domain → infra: VIOLATION
            make_internal(
                "src/domain/service/foo.rs",
                5,
                "crate::infrastructure::repo",
                "src/infrastructure/repo",
            ),
        ];

        let violations = detector.detect(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].from_layer, "domain");
        assert_eq!(violations[0].to_layer, "infrastructure");
    }

    #[test]
    fn test_detect_skips_non_internal_imports() {
        let layers = vec![make_layer(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);

        // Stdlib and External imports should never generate violations.
        let imports = vec![
            ResolvedImport {
                raw: RawImport {
                    path: "std::fs".to_string(),
                    line: 1,
                    file: "src/domain/entity/foo.rs".to_string(),
                    kind: ImportKind::Use,
                    named_imports: vec![],
                },
                category: ImportCategory::Stdlib,
                resolved_path: None,
            },
            ResolvedImport {
                raw: RawImport {
                    path: "serde::Deserialize".to_string(),
                    line: 2,
                    file: "src/domain/entity/foo.rs".to_string(),
                    kind: ImportKind::Use,
                    named_imports: vec![],
                },
                category: ImportCategory::External,
                resolved_path: None,
            },
        ];

        assert!(detector.detect(&imports).is_empty());
    }

    // ------------------------------------------------------------------
    // Dogfooding: mille.toml layer rules
    // ------------------------------------------------------------------

    #[test]
    fn test_dogfood_mille_toml_infra_to_domain_allowed() {
        // As per mille.toml: infrastructure opt-in allow=["domain"]
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptOut,
                &[],
                &["infrastructure"],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &["domain"],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_internal(
            "src/infrastructure/repository/toml_config_repository.rs",
            3,
            "crate::domain::entity::config::MilleConfig",
            "src/domain/entity/config",
        )];
        assert!(detector.detect(&imports).is_empty());
    }

    #[test]
    fn test_dogfood_mille_toml_domain_to_infra_violation() {
        // As per mille.toml: domain opt-out deny=["infrastructure"]
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptOut,
                &[],
                &["infrastructure"],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &["domain"],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_internal(
            "src/domain/service/foo.rs",
            1,
            "crate::infrastructure::parser::rust",
            "src/infrastructure/parser/rust",
        )];
        let violations = detector.detect(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].from_layer, "domain");
    }

    // ------------------------------------------------------------------
    // detect_external
    // ------------------------------------------------------------------

    fn make_layer_with_external(
        name: &str,
        paths: &[&str],
        mode: DependencyMode,
        external_allow: &[&str],
        external_deny: &[&str],
    ) -> LayerConfig {
        LayerConfig {
            name: name.to_string(),
            paths: paths.iter().map(|s| s.to_string()).collect(),
            dependency_mode: DependencyMode::OptIn,
            allow: vec![],
            deny: vec![],
            external_mode: mode,
            external_allow: external_allow.iter().map(|s| s.to_string()).collect(),
            external_deny: external_deny.iter().map(|s| s.to_string()).collect(),
            allow_call_patterns: vec![],
        }
    }

    fn make_external(file: &str, line: usize, path: &str) -> ResolvedImport {
        ResolvedImport {
            raw: RawImport {
                path: path.to_string(),
                line,
                file: file.to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::External,
            resolved_path: None,
        }
    }

    #[test]
    fn test_detect_external_opt_in_allowed_crate_no_violation() {
        // domain opt-in, external_allow=["serde"] → serde import OK
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &["serde"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_external(
            "src/domain/entity/config.rs",
            1,
            "serde::Deserialize",
        )];
        assert!(detector.detect_external(&imports).is_empty());
    }

    #[test]
    fn test_detect_external_opt_in_disallowed_crate_is_violation() {
        // domain opt-in, external_allow=[] → any external import is a violation
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_external(
            "src/domain/entity/config.rs",
            1,
            "serde::Deserialize",
        )];
        let violations = detector.detect_external(&imports);
        assert_eq!(violations.len(), 1);
        let v = &violations[0];
        assert_eq!(v.from_layer, "domain");
        assert_eq!(v.to_layer, "serde"); // crate name
        assert_eq!(v.kind, ViolationKind::ExternalViolation);
        assert_eq!(v.severity, Severity::Error);
        assert_eq!(v.line, 1);
    }

    #[test]
    fn test_detect_external_opt_out_allowed_no_violation() {
        // infrastructure opt-out, external_deny=[] → any external import is OK
        let layers = vec![make_layer_with_external(
            "infrastructure",
            &["src/infrastructure/**"],
            DependencyMode::OptOut,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_external(
            "src/infrastructure/parser/rust.rs",
            1,
            "tree_sitter::Node",
        )];
        assert!(detector.detect_external(&imports).is_empty());
    }

    #[test]
    fn test_detect_external_opt_out_denied_crate_is_violation() {
        // infrastructure opt-out, external_deny=["sqlx"] → sqlx import is a violation
        let layers = vec![make_layer_with_external(
            "infrastructure",
            &["src/infrastructure/**"],
            DependencyMode::OptOut,
            &[],
            &["sqlx"],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_external("src/infrastructure/db.rs", 5, "sqlx::query")];
        let violations = detector.detect_external(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].from_layer, "infrastructure");
        assert_eq!(violations[0].to_layer, "sqlx");
        assert_eq!(violations[0].kind, ViolationKind::ExternalViolation);
    }

    #[test]
    fn test_detect_external_each_crate_listed_separately() {
        // external_allow=["sqlx", "sea_orm"] → each crate needs its own exact entry
        let layers = vec![make_layer_with_external(
            "infra",
            &["src/infra/**"],
            DependencyMode::OptIn,
            &["sqlx", "sea_orm"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![
            make_external("src/infra/db.rs", 1, "sqlx::query"),
            make_external("src/infra/orm.rs", 2, "sea_orm::DatabaseConnection"),
        ];
        assert!(detector.detect_external(&imports).is_empty());
    }

    #[test]
    fn test_detect_external_pattern_is_exact_not_regex() {
        // "sqlx|sea_orm" as a single entry must NOT match "sqlx" — patterns are not regex
        let layers = vec![make_layer_with_external(
            "infra",
            &["src/infra/**"],
            DependencyMode::OptIn,
            &["sqlx|sea_orm"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_external("src/infra/db.rs", 1, "sqlx::query")];
        // "sqlx|sea_orm" is not "sqlx", so this must be a violation
        assert_eq!(detector.detect_external(&imports).len(), 1);
    }

    #[test]
    fn test_detect_external_skips_mod_declarations() {
        // `pub mod X;` declarations are module structure, not external library imports.
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "entity".to_string(),
                line: 1,
                file: "src/domain/mod.rs".to_string(),
                kind: ImportKind::Mod, // pub mod entity;
                named_imports: vec![],
            },
            category: ImportCategory::External,
            resolved_path: None,
        }];
        assert!(
            detector.detect_external(&imports).is_empty(),
            "pub mod X; must not be treated as an external library import"
        );
    }

    #[test]
    fn test_detect_external_skips_internal_and_stdlib() {
        // Internal and Stdlib imports must not generate external violations.
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![
            ResolvedImport {
                raw: RawImport {
                    path: "crate::domain::entity::config".to_string(),
                    line: 1,
                    file: "src/domain/service/foo.rs".to_string(),
                    kind: ImportKind::Use,
                    named_imports: vec![],
                },
                category: ImportCategory::Internal,
                resolved_path: Some("src/domain/entity/config".to_string()),
            },
            ResolvedImport {
                raw: RawImport {
                    path: "std::fmt".to_string(),
                    line: 2,
                    file: "src/domain/service/foo.rs".to_string(),
                    kind: ImportKind::Use,
                    named_imports: vec![],
                },
                category: ImportCategory::Stdlib,
                resolved_path: None,
            },
        ];
        assert!(detector.detect_external(&imports).is_empty());
    }

    #[test]
    fn test_detect_external_crate_name_extracted_from_path() {
        // "tree_sitter::Node" → crate name is "tree_sitter"
        let layers = vec![make_layer_with_external(
            "infra",
            &["src/infra/**"],
            DependencyMode::OptIn,
            &["tree_sitter"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_external("src/infra/parser.rs", 3, "tree_sitter::Node")];
        assert!(detector.detect_external(&imports).is_empty());
    }

    // ------------------------------------------------------------------
    // detect_call_patterns
    // ------------------------------------------------------------------

    fn make_layer_with_call_pattern(
        name: &str,
        paths: &[&str],
        callee_layer: &str,
        allow_methods: &[&str],
    ) -> LayerConfig {
        use crate::domain::entity::layer::CallPattern;
        LayerConfig {
            name: name.to_string(),
            paths: paths.iter().map(|s| s.to_string()).collect(),
            dependency_mode: DependencyMode::OptIn,
            allow: vec![callee_layer.to_string()],
            deny: vec![],
            external_mode: DependencyMode::OptIn,
            external_allow: vec![],
            external_deny: vec![],
            allow_call_patterns: vec![CallPattern {
                callee_layer: callee_layer.to_string(),
                allow_methods: allow_methods.iter().map(|s| s.to_string()).collect(),
            }],
        }
    }

    fn make_resolved_internal(file: &str, import_path: &str, resolved: &str) -> ResolvedImport {
        ResolvedImport {
            raw: RawImport {
                path: import_path.to_string(),
                line: 1,
                file: file.to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::Internal,
            resolved_path: Some(resolved.to_string()),
        }
    }

    fn make_static_call(file: &str, line: usize, receiver: &str, method: &str) -> RawCallExpr {
        RawCallExpr {
            file: file.to_string(),
            line,
            receiver_type: Some(receiver.to_string()),
            method: method.to_string(),
        }
    }

    fn make_instance_call(file: &str, line: usize, method: &str) -> RawCallExpr {
        RawCallExpr {
            file: file.to_string(),
            line,
            receiver_type: None,
            method: method.to_string(),
        }
    }

    #[test]
    fn test_no_allow_call_patterns_no_violations() {
        // A layer without allow_call_patterns should never emit CallPatternViolation.
        let layers = vec![
            make_layer("main", &["src/main.rs"], DependencyMode::OptIn, &[], &[]),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let calls = vec![make_static_call("src/main.rs", 5, "Repo", "find_user")];
        let imports = vec![make_resolved_internal(
            "src/main.rs",
            "crate::infrastructure::Repo",
            "src/infrastructure/Repo",
        )];
        assert!(detector.detect_call_patterns(&calls, &imports).is_empty());
    }

    #[test]
    fn test_allowed_method_no_violation() {
        // Repo::new() where "new" is in allow_methods → no violation.
        let layers = vec![
            make_layer_with_call_pattern(
                "main",
                &["src/main.rs"],
                "infrastructure",
                &["new", "build"],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let calls = vec![make_static_call("src/main.rs", 3, "Repo", "new")];
        let imports = vec![make_resolved_internal(
            "src/main.rs",
            "crate::infrastructure::Repo",
            "src/infrastructure/Repo",
        )];
        assert!(
            detector.detect_call_patterns(&calls, &imports).is_empty(),
            "Repo::new() should be allowed"
        );
    }

    #[test]
    fn test_disallowed_method_is_violation() {
        // Repo::find_user() where only "new" is in allow_methods → violation.
        let layers = vec![
            make_layer_with_call_pattern("main", &["src/main.rs"], "infrastructure", &["new"]),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let calls = vec![make_static_call("src/main.rs", 10, "Repo", "find_user")];
        let imports = vec![make_resolved_internal(
            "src/main.rs",
            "crate::infrastructure::Repo",
            "src/infrastructure/Repo",
        )];
        let violations = detector.detect_call_patterns(&calls, &imports);
        assert_eq!(violations.len(), 1);
        let v = &violations[0];
        assert_eq!(v.from_layer, "main");
        assert_eq!(v.to_layer, "infrastructure");
        assert_eq!(v.kind, ViolationKind::CallPatternViolation);
        assert_eq!(v.severity, Severity::Error);
        assert_eq!(v.line, 10);
    }

    #[test]
    fn test_instance_call_skipped_no_violation() {
        // Instance calls (receiver_type = None) cannot be type-checked; must not emit violations.
        let layers = vec![
            make_layer_with_call_pattern("main", &["src/main.rs"], "infrastructure", &["new"]),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let calls = vec![make_instance_call("src/main.rs", 12, "find_user")];
        let imports = vec![make_resolved_internal(
            "src/main.rs",
            "crate::infrastructure::Repo",
            "src/infrastructure/Repo",
        )];
        assert!(
            detector.detect_call_patterns(&calls, &imports).is_empty(),
            "instance calls should be skipped"
        );
    }

    #[test]
    fn test_receiver_type_not_from_callee_layer_no_violation() {
        // Repo::new() but Repo is from "domain", not "infrastructure" → no violation.
        let layers = vec![
            make_layer_with_call_pattern("main", &["src/main.rs"], "infrastructure", &["new"]),
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers);
        let calls = vec![make_static_call("src/main.rs", 5, "Repo", "find_user")];
        // import resolves to domain, not infrastructure
        let imports = vec![make_resolved_internal(
            "src/main.rs",
            "crate::domain::Repo",
            "src/domain/Repo",
        )];
        assert!(
            detector.detect_call_patterns(&calls, &imports).is_empty(),
            "Repo from domain should not be checked against infrastructure allow_call_patterns"
        );
    }

    // ------------------------------------------------------------------
    // severity configuration
    // ------------------------------------------------------------------

    #[test]
    fn test_detect_dependency_violation_uses_configured_warning_severity() {
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &["domain"],
                &[],
            ),
        ];
        let severity = SeverityConfig {
            dependency_violation: "warning".to_string(),
            ..SeverityConfig::default()
        };
        let detector = ViolationDetector::with_severity(&layers, severity);
        // domain → infra: violation, but configured as warning
        let imports = vec![make_internal(
            "src/domain/service/foo.rs",
            5,
            "crate::infrastructure::repo",
            "src/infrastructure/repo",
        )];
        let violations = detector.detect(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning);
        assert_eq!(violations[0].kind, ViolationKind::DependencyViolation);
    }

    #[test]
    fn test_detect_external_violation_uses_configured_warning_severity() {
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let severity = SeverityConfig {
            external_violation: "warning".to_string(),
            ..SeverityConfig::default()
        };
        let detector = ViolationDetector::with_severity(&layers, severity);
        let imports = vec![make_external(
            "src/domain/entity/config.rs",
            1,
            "serde::Deserialize",
        )];
        let violations = detector.detect_external(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning);
        assert_eq!(violations[0].kind, ViolationKind::ExternalViolation);
    }

    #[test]
    fn test_detect_call_pattern_violation_uses_configured_warning_severity() {
        let layers = vec![
            make_layer_with_call_pattern("main", &["src/main.rs"], "infrastructure", &["new"]),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
        ];
        let severity = SeverityConfig {
            call_pattern_violation: "warning".to_string(),
            ..SeverityConfig::default()
        };
        let detector = ViolationDetector::with_severity(&layers, severity);
        let calls = vec![make_static_call("src/main.rs", 10, "Repo", "find_user")];
        let imports = vec![make_resolved_internal(
            "src/main.rs",
            "crate::infrastructure::Repo",
            "src/infrastructure/Repo",
        )];
        let violations = detector.detect_call_patterns(&calls, &imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning);
        assert_eq!(violations[0].kind, ViolationKind::CallPatternViolation);
    }

    #[test]
    fn test_detect_unknown_reports_unknown_import_with_default_warning_severity() {
        let layers = vec![make_layer(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "some-unresolvable-module".to_string(),
                line: 3,
                file: "src/domain/service/foo.rs".to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::Unknown,
            resolved_path: None,
        }];
        let violations = detector.detect_unknown(&imports);
        assert_eq!(violations.len(), 1, "unknown import must be reported");
        assert_eq!(violations[0].severity, Severity::Warning);
        assert_eq!(violations[0].kind, ViolationKind::UnknownImport);
        assert_eq!(violations[0].from_layer, "domain");
        assert_eq!(violations[0].import_path, "some-unresolvable-module");
    }

    #[test]
    fn test_detect_unknown_uses_configured_error_severity() {
        let layers = vec![make_layer(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let severity = SeverityConfig {
            unknown_import: "error".to_string(),
            ..SeverityConfig::default()
        };
        let detector = ViolationDetector::with_severity(&layers, severity);
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "mystery-import".to_string(),
                line: 7,
                file: "src/domain/entity/config.rs".to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::Unknown,
            resolved_path: None,
        }];
        let violations = detector.detect_unknown(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Error);
    }

    #[test]
    fn test_detect_unknown_skips_files_not_in_any_layer() {
        let layers = vec![make_layer(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        // File is in "src/other/" which matches no layer
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "unknown-thing".to_string(),
                line: 1,
                file: "src/other/helper.rs".to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::Unknown,
            resolved_path: None,
        }];
        assert!(
            detector.detect_unknown(&imports).is_empty(),
            "unknown imports from files outside any layer must be skipped"
        );
    }

    #[test]
    fn test_detect_unknown_skips_non_unknown_categories() {
        let layers = vec![make_layer(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &[],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![
            make_internal(
                "src/domain/service/foo.rs",
                1,
                "crate::domain::entity::config",
                "src/domain/entity/config",
            ),
            make_external("src/domain/entity/config.rs", 2, "serde::Deserialize"),
        ];
        assert!(
            detector.detect_unknown(&imports).is_empty(),
            "Internal and External imports must not be reported by detect_unknown"
        );
    }

    #[test]
    fn test_detect_violation_default_severity_is_error() {
        // Regression: without explicit severity config, violations must still be Error.
        let layers = vec![
            make_layer(
                "domain",
                &["src/domain/**"],
                DependencyMode::OptIn,
                &[],
                &[],
            ),
            make_layer(
                "infrastructure",
                &["src/infrastructure/**"],
                DependencyMode::OptIn,
                &["domain"],
                &[],
            ),
        ];
        let detector = ViolationDetector::new(&layers); // default severity
        let imports = vec![make_internal(
            "src/domain/service/foo.rs",
            5,
            "crate::infrastructure::repo",
            "src/infrastructure/repo",
        )];
        let violations = detector.detect(&imports);
        assert_eq!(violations.len(), 1);
        assert_eq!(
            violations[0].severity,
            Severity::Error,
            "default severity must be Error"
        );
    }
}
