use crate::domain::entity::layer::{DependencyMode, LayerConfig};
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::entity::violation::{Severity, Violation, ViolationKind};

pub struct ViolationDetector<'a> {
    layers: &'a [LayerConfig],
}

impl<'a> ViolationDetector<'a> {
    pub fn new(layers: &'a [LayerConfig]) -> Self {
        Self { layers }
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
            severity: Severity::Error,
        })
    }
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
}
