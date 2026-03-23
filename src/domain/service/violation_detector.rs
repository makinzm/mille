use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::config::SeverityConfig;
use crate::domain::entity::import::ImportKind;
use crate::domain::entity::layer::{DependencyMode, LayerConfig};
use crate::domain::entity::name::RawName;
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
            // Use package_name set by the resolver (language-aware, no extension checks here).
            // Fallback to raw path for backwards compatibility with tests.
            let crate_name: &str = match &import.package_name {
                Some(name) => name.as_str(),
                None => &import.raw.path,
            };
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
                            // Colon-separated / slash-separated: type name embedded in import path
                            type_name_from_import(&imp.raw.path)
                                .map(|n| n == receiver_type.as_str())
                                .unwrap_or(false)
                            // Dot-separated / slash-separated: named imports tracked explicitly
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

    /// Check naming convention rules: for each name, match `name_deny` keywords against
    /// the name value (case-insensitive partial match).
    ///
    /// `raw_names` contains all names extracted from source files (symbols, variables, comments).
    /// File-level checks (NameKind::File) should be pre-computed by the caller and passed as RawName.
    pub fn detect_naming(&self, raw_names: &[RawName]) -> Vec<Violation> {
        let sev = parse_severity(&self.severity.naming_violation);
        let mut violations = Vec::new();

        for raw_name in raw_names {
            let Some(layer) = self.find_layer_for_file(&raw_name.file) else {
                continue;
            };
            if layer.name_deny.is_empty() {
                continue;
            }

            // Skip files matching name_deny_ignore glob patterns
            if !layer.name_deny_ignore.is_empty()
                && layer.name_deny_ignore.iter().any(|pat| {
                    glob::Pattern::new(pat)
                        .map(|p| p.matches(&raw_name.file))
                        .unwrap_or(false)
                })
            {
                continue;
            }

            // Check if this name's kind is in the layer's name_targets
            let target_kind = raw_name.kind;
            let is_targeted = layer
                .name_targets
                .iter()
                .any(|t| t.as_name_kind() == target_kind);
            if !is_targeted {
                continue;
            }

            // Case-insensitive partial match against each denied keyword.
            // Strip name_allow substrings first so composite words like "category"
            // don't cause false positives when a denied keyword appears inside them.
            let name_lower = raw_name.name.to_lowercase();
            let name_stripped = layer
                .name_allow
                .iter()
                .fold(name_lower.clone(), |acc, allow| {
                    acc.replace(&allow.to_lowercase() as &str, "")
                });
            for keyword in &layer.name_deny {
                if name_stripped.contains(keyword.to_lowercase().as_str()) {
                    let target_str = match raw_name.kind {
                        crate::domain::entity::name::NameKind::File => "file",
                        crate::domain::entity::name::NameKind::Symbol => "symbol",
                        crate::domain::entity::name::NameKind::Variable => "variable",
                        crate::domain::entity::name::NameKind::Comment => "comment",
                        crate::domain::entity::name::NameKind::StringLiteral => "string_literal",
                    };
                    violations.push(Violation {
                        file: raw_name.file.clone(),
                        line: raw_name.line,
                        from_layer: layer.name.clone(),
                        to_layer: target_str.to_string(),
                        import_path: keyword.clone(),
                        kind: ViolationKind::NamingViolation,
                        severity: sev.clone(),
                    });
                    // Report once per name per layer (first matching keyword)
                    break;
                }
            }
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
/// - Colon-separated: `"crate::infrastructure::Repo"` -> `Some("Repo")`
/// - Slash-separated: `"github.com/example/sample/domain"` -> `Some("domain")`
/// - Returns `None` for wildcards (`*`) and grouped imports (`{...}`).
///
/// Dot-separated and slash-separated named imports are checked via `named_imports` field directly.
fn type_name_from_import(path: &str) -> Option<&str> {
    // Colon-separated paths use "::" separator.
    if path.contains("::") {
        let last = path.split("::").last()?;
        if last.starts_with('{') || last == "*" {
            return None;
        }
        return Some(last);
    }

    // Backslash-separated paths use "\" separator (e.g. "App\Domain\User").
    // The last segment is the class name used as the call receiver.
    if path.contains('\\') {
        return path.split('\\').last().filter(|s| !s.is_empty());
    }

    // Slash-separated paths use "/" separator (e.g. "github.com/foo/bar/domain").
    // The last segment is the package name used as the call receiver.
    if path.contains('/') {
        return path.split('/').last().filter(|s| !s.is_empty());
    }

    // Plain single-segment paths (e.g. "fmt", "os" in stdlib).
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
        use crate::domain::entity::layer::NameTarget;
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
            name_deny: vec![],
            name_allow: vec![],
            name_targets: NameTarget::all(),
            name_deny_ignore: vec![],
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
            package_name: None,
        }
    }

    fn make_external(file: &str, line: usize, path: &str, package: &str) -> ResolvedImport {
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
            package_name: Some(package.to_string()),
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

        let found2 = detector.find_layer_for_file("src/infrastructure/parser/lang.rs");
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
                package_name: None,
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
                package_name: None,
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
            "crate::infrastructure::parser::lang",
            "src/infrastructure/parser/lang",
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
            name_deny: vec![],
            name_allow: vec![],
            name_targets: crate::domain::entity::layer::NameTarget::all(),
            name_deny_ignore: vec![],
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
            "serde",
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
            "serde",
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
            "src/infrastructure/parser/lang.rs",
            1,
            "tree_sitter::Node",
            "tree_sitter",
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
        let imports = vec![make_external(
            "src/infrastructure/db.rs",
            5,
            "sqlx::query",
            "sqlx",
        )];
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
            make_external("src/infra/db.rs", 1, "sqlx::query", "sqlx"),
            make_external(
                "src/infra/orm.rs",
                2,
                "sea_orm::DatabaseConnection",
                "sea_orm",
            ),
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
        let imports = vec![make_external("src/infra/db.rs", 1, "sqlx::query", "sqlx")];
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
            package_name: None,
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
                package_name: None,
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
                package_name: None,
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
        let imports = vec![make_external(
            "src/infra/parser.rs",
            3,
            "tree_sitter::Node",
            "tree_sitter",
        )];
        assert!(detector.detect_external(&imports).is_empty());
    }

    #[test]
    fn test_detect_external_dotted_import_allowed() {
        // "matplotlib.pyplot" import with external_allow=["matplotlib"] → no violation
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &["matplotlib"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "matplotlib.pyplot".to_string(),
                line: 1,
                file: "src/domain/chart.py".to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::External,
            resolved_path: None,
            package_name: Some("matplotlib".to_string()),
        }];
        assert!(
            detector.detect_external(&imports).is_empty(),
            "matplotlib.pyplot should be allowed when external_allow=[\"matplotlib\"]"
        );
    }

    #[test]
    fn test_detect_external_dotted_import_violation() {
        // "unknown.submodule" not in external_allow → violation
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &["matplotlib"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "unknown.submodule".to_string(),
                line: 2,
                file: "src/domain/chart.py".to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::External,
            resolved_path: None,
            package_name: Some("unknown".to_string()),
        }];
        let violations = detector.detect_external(&imports);
        assert_eq!(violations.len(), 1, "unknown.submodule must be a violation");
        assert_eq!(
            violations[0].to_layer, "unknown",
            "crate_name should be 'unknown'"
        );
    }

    #[test]
    fn test_detect_external_ts_subpath_allowed_by_package_name() {
        // Slash-separated: "vitest/config" should match external_allow = ["vitest"]
        // crate_name extraction uses first npm segment for .ts files
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &["vitest"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "vitest/config".to_string(),
                line: 1,
                file: "src/domain/test.ts".to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::External,
            resolved_path: None,
            package_name: Some("vitest".to_string()),
        }];
        assert!(
            detector.detect_external(&imports).is_empty(),
            "vitest/config should be allowed when external_allow=[\"vitest\"]"
        );
    }

    #[test]
    fn test_detect_external_ts_scoped_package_allowed() {
        // Slash-separated (scoped): "@vueuse/core/utilities" should match external_allow = ["@vueuse/core"]
        let layers = vec![make_layer_with_external(
            "domain",
            &["src/domain/**"],
            DependencyMode::OptIn,
            &["@vueuse/core"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "@vueuse/core/utilities".to_string(),
                line: 1,
                file: "src/domain/component.ts".to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::External,
            resolved_path: None,
            package_name: Some("@vueuse/core".to_string()),
        }];
        assert!(
            detector.detect_external(&imports).is_empty(),
            "@vueuse/core/utilities should be allowed when external_allow=[\"@vueuse/core\"]"
        );
    }

    #[test]
    fn test_detect_external_full_path_allowed() {
        // Full module path used as crate_name -- exact match required
        let layers = vec![make_layer_with_external(
            "infra",
            &["lang/infra/**"],
            DependencyMode::OptIn,
            &["github.com/cilium/ebpf"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![ResolvedImport {
            raw: RawImport {
                path: "github.com/cilium/ebpf".to_string(),
                line: 1,
                file: "lang/infra/ebpf.x".to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category: ImportCategory::External,
            resolved_path: None,
            package_name: None,
        }];
        assert!(
            detector.detect_external(&imports).is_empty(),
            "github.com/cilium/ebpf should be allowed when exact path is in external_allow"
        );
    }

    #[test]
    fn test_detect_external_stdlib_allowed() {
        // Stdlib packages ("fmt", "net/http") appear in external_allow with full path
        let layers = vec![make_layer_with_external(
            "domain",
            &["lang/domain/**"],
            DependencyMode::OptIn,
            &["fmt", "net/http"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![
            ResolvedImport {
                raw: RawImport {
                    path: "fmt".to_string(),
                    line: 1,
                    file: "lang/domain/user.x".to_string(),
                    kind: ImportKind::Use,
                    named_imports: vec![],
                },
                category: ImportCategory::External,
                resolved_path: None,
                package_name: None,
            },
            ResolvedImport {
                raw: RawImport {
                    path: "net/http".to_string(),
                    line: 2,
                    file: "lang/domain/user.x".to_string(),
                    kind: ImportKind::Use,
                    named_imports: vec![],
                },
                category: ImportCategory::External,
                resolved_path: None,
                package_name: None,
            },
        ];
        assert!(
            detector.detect_external(&imports).is_empty(),
            "fmt and net/http should be allowed when in external_allow"
        );
    }

    #[test]
    fn test_detect_external_colon_separator() {
        // Regression: "::" splitting still works after dotted-import fix
        let layers = vec![make_layer_with_external(
            "infra",
            &["src/infra/**"],
            DependencyMode::OptIn,
            &["serde"],
            &[],
        )];
        let detector = ViolationDetector::new(&layers);
        let imports = vec![make_external(
            "src/infra/repo.rs",
            1,
            "serde::Deserialize",
            "serde",
        )];
        assert!(
            detector.detect_external(&imports).is_empty(),
            "serde::Deserialize should be allowed for .rs files"
        );
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
            name_deny: vec![],
            name_allow: vec![],
            name_targets: crate::domain::entity::layer::NameTarget::all(),
            name_deny_ignore: vec![],
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
            package_name: None,
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
            "serde",
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
            package_name: None,
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
            package_name: None,
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
            package_name: None,
        }];
        assert!(
            detector.detect_unknown(&imports).is_empty(),
            "unknown imports from files outside any layer must be skipped"
        );
    }

    #[test]
    fn test_detect_unknown_skips_non_unknown_types() {
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
            make_external(
                "src/domain/entity/config.rs",
                2,
                "serde::Deserialize",
                "serde",
            ),
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

    // ------------------------------------------------------------------
    // detect_naming
    // ------------------------------------------------------------------

    use crate::domain::entity::layer::NameTarget;
    use crate::domain::entity::name::NameKind;

    fn make_layer_with_name_deny(
        name: &str,
        paths: &[&str],
        name_deny: &[&str],
        name_targets: Vec<NameTarget>,
    ) -> LayerConfig {
        LayerConfig {
            name: name.to_string(),
            paths: paths.iter().map(|s| s.to_string()).collect(),
            dependency_mode: DependencyMode::OptOut,
            allow: vec![],
            deny: vec![],
            external_mode: DependencyMode::OptOut,
            external_allow: vec![],
            external_deny: vec![],
            allow_call_patterns: vec![],
            name_deny: name_deny.iter().map(|s| s.to_string()).collect(),
            name_allow: vec![],
            name_targets,
            name_deny_ignore: vec![],
        }
    }

    fn make_raw_name(name: &str, kind: NameKind, line: usize, file: &str) -> RawName {
        RawName {
            name: name.to_string(),
            kind,
            line,
            file: file.to_string(),
        }
    }

    #[test]
    fn test_detect_naming_symbol_violation() {
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["aws"],
            NameTarget::all(),
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "AwsClient",
            NameKind::Symbol,
            10,
            "src/usecase/service.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            1,
            "AwsClient should violate name_deny=[\"aws\"]"
        );
        assert_eq!(violations[0].from_layer, "usecase");
        assert_eq!(violations[0].kind, ViolationKind::NamingViolation);
        // import_path holds the matched keyword
        assert_eq!(violations[0].import_path, "aws");
        // to_layer holds the target kind
        assert_eq!(violations[0].to_layer, "symbol");
    }

    #[test]
    fn test_detect_naming_case_insensitive() {
        // name_deny = ["AWS"] should match "aws_handler" (case-insensitive)
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["AWS"],
            NameTarget::all(),
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "aws_handler",
            NameKind::Symbol,
            5,
            "src/usecase/handler.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            1,
            "aws_handler should match AWS (case-insensitive)"
        );
    }

    #[test]
    fn test_detect_naming_partial_match() {
        // "ManageAws" should match name_deny = ["aws"] (partial match)
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["aws"],
            NameTarget::all(),
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "ManageAws",
            NameKind::Symbol,
            3,
            "src/usecase/manage.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            1,
            "ManageAws should match aws (partial match)"
        );
    }

    #[test]
    fn test_detect_naming_target_filter_symbol_only() {
        // name_targets = [Symbol] のとき Variable は対象外
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["aws"],
            vec![NameTarget::Symbol],
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "aws_url",
            NameKind::Variable,
            2,
            "src/usecase/service.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            0,
            "Variable should be ignored when name_targets=[Symbol]"
        );
    }

    #[test]
    fn test_detect_naming_no_violation_when_keyword_absent() {
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["aws"],
            NameTarget::all(),
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "UserService",
            NameKind::Symbol,
            1,
            "src/usecase/user_service.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            0,
            "UserService should not violate name_deny=[\"aws\"]"
        );
    }

    #[test]
    fn test_detect_naming_no_violation_when_name_deny_is_empty() {
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &[], // empty name_deny
            NameTarget::all(),
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "AwsClient",
            NameKind::Symbol,
            1,
            "src/usecase/service.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            0,
            "empty name_deny should produce no violations"
        );
    }

    #[test]
    fn test_detect_naming_file_not_in_any_layer_is_ignored() {
        // File not matching any layer's paths should be ignored
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["aws"],
            NameTarget::all(),
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "AwsClient",
            NameKind::Symbol,
            1,
            "src/domain/entity.rs", // not in usecase layer
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            0,
            "files not in any layer should be ignored"
        );
    }

    #[test]
    fn test_detect_naming_severity_from_config() {
        use crate::domain::entity::config::SeverityConfig;
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["aws"],
            NameTarget::all(),
        )];
        let severity = SeverityConfig {
            naming_violation: "warning".to_string(),
            ..SeverityConfig::default()
        };
        let detector = ViolationDetector::with_severity(&layers, severity);
        let names = vec![make_raw_name(
            "AwsClient",
            NameKind::Symbol,
            1,
            "src/usecase/service.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].severity, Severity::Warning);
    }

    #[test]
    fn test_detect_naming_name_allow_suppresses_false_positive() {
        // "category" contains a denied keyword but name_allow = ["category"] should suppress it
        let mut layer =
            make_layer_with_name_deny("domain", &["src/domain/**"], &["bad"], NameTarget::all());
        layer.name_allow = vec!["category".to_string()];
        let layers = vec![layer];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "ImportCategory",
            NameKind::Symbol,
            14,
            "src/domain/entity/resolved_import.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            0,
            "ImportCategory should not be flagged: denied keyword inside 'category' is allowed"
        );
    }

    #[test]
    fn test_detect_naming_name_allow_does_not_suppress_standalone_keyword() {
        // name_allow = ["category"] must NOT suppress names without "category" substring
        let mut layer =
            make_layer_with_name_deny("domain", &["src/domain/**"], &["bad"], NameTarget::all());
        layer.name_allow = vec!["category".to_string()];
        let layers = vec![layer];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "BadConfig",
            NameKind::Symbol,
            10,
            "src/domain/entity/config.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(violations.len(), 1, "BadConfig must still be flagged");
    }

    #[test]
    fn test_detect_naming_name_allow_partial_coverage_still_violations() {
        // Name still contains the denied keyword after stripping "category" -- still a violation
        let mut layer =
            make_layer_with_name_deny("domain", &["src/domain/**"], &["bad"], NameTarget::all());
        layer.name_allow = vec!["category".to_string()];
        let layers = vec![layer];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "BadCategory",
            NameKind::Symbol,
            5,
            "src/domain/entity/config.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            1,
            "BadCategory must still be flagged (standalone 'Bad' remains after stripping 'category')"
        );
    }

    #[test]
    fn test_detect_naming_ignore_skips_symbols_in_matching_file() {
        // Files matching name_deny_ignore should not produce violations
        let mut layer =
            make_layer_with_name_deny("domain", &["src/domain/**"], &["aws"], NameTarget::all());
        layer.name_deny_ignore = vec!["**/test_*.rs".to_string()];
        let layers = vec![layer];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "AwsClient",
            NameKind::Symbol,
            10,
            "src/domain/test_helpers.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            0,
            "test_helpers.rs matches **/test_*.rs so it should be ignored"
        );
    }

    #[test]
    fn test_detect_naming_ignore_does_not_affect_non_matching_files() {
        // Files NOT matching name_deny_ignore are still checked
        let mut layer =
            make_layer_with_name_deny("domain", &["src/domain/**"], &["aws"], NameTarget::all());
        layer.name_deny_ignore = vec!["**/test_*.rs".to_string()];
        let layers = vec![layer];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "AwsClient",
            NameKind::Symbol,
            10,
            "src/domain/entity/client.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            1,
            "client.rs does not match the ignore pattern — should still be flagged"
        );
    }

    #[test]
    fn test_detect_naming_ignore_empty_by_default_checks_all_files() {
        // Without name_deny_ignore, all files in the layer are checked (regression)
        let layer =
            make_layer_with_name_deny("domain", &["src/domain/**"], &["aws"], NameTarget::all());
        let layers = vec![layer];
        let detector = ViolationDetector::new(&layers);
        let names = vec![
            make_raw_name(
                "AwsClient",
                NameKind::Symbol,
                10,
                "src/domain/entity/foo.rs",
            ),
            make_raw_name(
                "AwsHelper",
                NameKind::Symbol,
                20,
                "src/domain/test_helper.rs",
            ),
        ];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            2,
            "both files should be checked when ignore is empty"
        );
    }

    #[test]
    fn test_detect_naming_string_literal_violation() {
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["aws"],
            NameTarget::all(),
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "aws-sdk-bucket",
            NameKind::StringLiteral,
            15,
            "src/usecase/service.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            1,
            "string literal containing 'aws' should violate name_deny"
        );
        assert_eq!(violations[0].from_layer, "usecase");
        assert_eq!(violations[0].to_layer, "string_literal");
        assert_eq!(violations[0].import_path, "aws");
        assert_eq!(violations[0].kind, ViolationKind::NamingViolation);
    }

    #[test]
    fn test_detect_naming_target_filter_excludes_string_literal() {
        // name_targets = [Symbol, Variable] のとき StringLiteral は対象外
        let layers = vec![make_layer_with_name_deny(
            "usecase",
            &["src/usecase/**"],
            &["aws"],
            vec![NameTarget::Symbol, NameTarget::Variable],
        )];
        let detector = ViolationDetector::new(&layers);
        let names = vec![make_raw_name(
            "aws-sdk-bucket",
            NameKind::StringLiteral,
            15,
            "src/usecase/service.rs",
        )];
        let violations = detector.detect_naming(&names);
        assert_eq!(
            violations.len(),
            0,
            "StringLiteral should be ignored when name_targets=[Symbol, Variable]"
        );
    }
}
