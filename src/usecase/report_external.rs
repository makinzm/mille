use std::collections::{BTreeMap, BTreeSet};

use crate::domain::entity::import::ImportKind;
use crate::domain::entity::resolved_import::ImportCategory;
use crate::domain::repository::config_repository::ConfigRepository;
use crate::domain::repository::parser::Parser;
use crate::domain::repository::resolver::Resolver;
use crate::domain::repository::source_file_repository::SourceFileRepository;

pub struct ReportExternalResult {
    pub layers: Vec<LayerExternalReport>,
}

pub struct LayerExternalReport {
    pub layer_name: String,
    /// Sorted, deduplicated list of external package names used in this layer.
    pub packages: Vec<String>,
}

/// Run the `report external` pipeline using injected ports.
pub fn report_external(
    config_path: &str,
    config_repo: &dyn ConfigRepository,
    file_repo: &dyn SourceFileRepository,
    parser: &dyn Parser,
    resolver: &dyn Resolver,
) -> Result<ReportExternalResult, String> {
    todo!()
}

/// Extract the top-level package name from a raw import path.
///
/// Uses the same strategy as `ViolationDetector::detect_external`:
/// split on `::` and take the first segment. This works for all languages:
/// - Rust: `serde::Deserialize` → `serde`
/// - Go: `database/sql` → `database/sql` (no `::`, whole string returned)
/// - TypeScript: `lodash` → `lodash`
/// - Python: `sqlalchemy` → `sqlalchemy`
fn extract_package_name(path: &str) -> &str {
    path.split("::").next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::call_expr::RawCallExpr;
    use crate::domain::entity::config::{MilleConfig, ProjectConfig, SeverityConfig};
    use crate::domain::entity::import::{ImportKind, RawImport};
    use crate::domain::entity::layer::{DependencyMode, LayerConfig};
    use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};

    // ------------------------------------------------------------------
    // Test doubles
    // ------------------------------------------------------------------

    struct FixedConfigRepo(MilleConfig);
    impl ConfigRepository for FixedConfigRepo {
        fn load(&self, _: &str) -> std::io::Result<MilleConfig> {
            Ok(self.0.clone())
        }
    }

    struct EmptyFileRepo;
    impl SourceFileRepository for EmptyFileRepo {
        fn collect(&self, _: &[String]) -> Vec<String> {
            vec![]
        }
    }

    struct NoOpParser;
    impl Parser for NoOpParser {
        fn parse_imports(&self, _: &str, _: &str) -> Vec<RawImport> {
            vec![]
        }
        fn parse_call_exprs(&self, _: &str, _: &str) -> Vec<RawCallExpr> {
            vec![]
        }
    }

    struct NoOpResolver;
    impl Resolver for NoOpResolver {
        fn resolve(&self, import: &RawImport) -> ResolvedImport {
            ResolvedImport {
                raw: import.clone(),
                category: ImportCategory::Unknown,
                resolved_path: None,
            }
        }
    }

    /// A resolver that returns whatever category+path we pre-define per file.
    struct FixedResolver(Vec<ResolvedImport>);
    impl Resolver for FixedResolver {
        fn resolve(&self, import: &RawImport) -> ResolvedImport {
            self.0
                .iter()
                .find(|r| r.raw.path == import.path && r.raw.file == import.file)
                .cloned()
                .unwrap_or(ResolvedImport {
                    raw: import.clone(),
                    category: ImportCategory::Unknown,
                    resolved_path: None,
                })
        }
    }

    /// A file repo that returns a fixed list of files for any paths glob.
    struct FixedFileRepo(Vec<String>);
    impl SourceFileRepository for FixedFileRepo {
        fn collect(&self, _: &[String]) -> Vec<String> {
            self.0.clone()
        }
    }

    /// A parser that returns pre-defined imports for specific files.
    struct FixedParser(Vec<RawImport>);
    impl Parser for FixedParser {
        fn parse_imports(&self, _source: &str, file: &str) -> Vec<RawImport> {
            self.0
                .iter()
                .filter(|i| i.file == file)
                .cloned()
                .collect()
        }
        fn parse_call_exprs(&self, _: &str, _: &str) -> Vec<RawCallExpr> {
            vec![]
        }
    }

    fn make_config(layers: Vec<LayerConfig>) -> MilleConfig {
        MilleConfig {
            project: ProjectConfig {
                name: "test".to_string(),
                root: ".".to_string(),
                languages: vec![],
            },
            layers,
            ignore: None,
            resolve: None,
            severity: SeverityConfig::default(),
        }
    }

    fn make_layer(name: &str, paths: &[&str]) -> LayerConfig {
        LayerConfig {
            name: name.to_string(),
            paths: paths.iter().map(|s| s.to_string()).collect(),
            dependency_mode: DependencyMode::OptIn,
            allow: vec![],
            deny: vec![],
            external_mode: DependencyMode::OptIn,
            external_allow: vec![],
            external_deny: vec![],
            allow_call_patterns: vec![],
        }
    }

    fn make_raw(file: &str, path: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: file.to_string(),
            kind: ImportKind::Use,
            named_imports: vec![],
        }
    }

    fn make_resolved(file: &str, path: &str, category: ImportCategory) -> ResolvedImport {
        ResolvedImport {
            raw: make_raw(file, path),
            category,
            resolved_path: None,
        }
    }

    // ------------------------------------------------------------------
    // extract_package_name
    // ------------------------------------------------------------------

    #[test]
    fn test_extract_package_name_rust_path() {
        assert_eq!(extract_package_name("serde::Deserialize"), "serde");
    }

    #[test]
    fn test_extract_package_name_go_path() {
        assert_eq!(extract_package_name("database/sql"), "database/sql");
    }

    #[test]
    fn test_extract_package_name_plain() {
        assert_eq!(extract_package_name("sqlalchemy"), "sqlalchemy");
    }

    // ------------------------------------------------------------------
    // report_external — unit
    // ------------------------------------------------------------------

    #[test]
    fn test_groups_packages_by_layer() {
        let config = make_config(vec![
            make_layer("domain", &["src/domain/**"]),
            make_layer("infra", &["src/infra/**"]),
        ]);

        // domain file imports serde; infra file imports toml
        let raw_imports = vec![
            make_raw("src/domain/entity/foo.rs", "serde::Deserialize"),
            make_raw("src/infra/repo.rs", "toml::Value"),
        ];
        let resolved = vec![
            make_resolved(
                "src/domain/entity/foo.rs",
                "serde::Deserialize",
                ImportCategory::External,
            ),
            make_resolved("src/infra/repo.rs", "toml::Value", ImportCategory::External),
        ];

        let result = report_external(
            "any.toml",
            &FixedConfigRepo(config),
            &FixedFileRepo(vec![
                "src/domain/entity/foo.rs".to_string(),
                "src/infra/repo.rs".to_string(),
            ]),
            &FixedParser(raw_imports),
            &FixedResolver(resolved),
        )
        .unwrap();

        let domain = result.layers.iter().find(|l| l.layer_name == "domain").unwrap();
        let infra = result.layers.iter().find(|l| l.layer_name == "infra").unwrap();
        assert_eq!(domain.packages, vec!["serde"]);
        assert_eq!(infra.packages, vec!["toml"]);
    }

    #[test]
    fn test_deduplicates_same_package() {
        let config = make_config(vec![make_layer("domain", &["src/domain/**"])]);

        let raw_imports = vec![
            make_raw("src/domain/entity/a.rs", "serde::Deserialize"),
            make_raw("src/domain/entity/b.rs", "serde::Serialize"),
        ];
        let resolved = vec![
            make_resolved(
                "src/domain/entity/a.rs",
                "serde::Deserialize",
                ImportCategory::External,
            ),
            make_resolved(
                "src/domain/entity/b.rs",
                "serde::Serialize",
                ImportCategory::External,
            ),
        ];

        let result = report_external(
            "any.toml",
            &FixedConfigRepo(config),
            &FixedFileRepo(vec![
                "src/domain/entity/a.rs".to_string(),
                "src/domain/entity/b.rs".to_string(),
            ]),
            &FixedParser(raw_imports),
            &FixedResolver(resolved),
        )
        .unwrap();

        let domain = result.layers.iter().find(|l| l.layer_name == "domain").unwrap();
        assert_eq!(domain.packages, vec!["serde"]);
    }

    #[test]
    fn test_skips_non_external_imports() {
        let config = make_config(vec![make_layer("domain", &["src/domain/**"])]);

        let raw_imports = vec![
            make_raw("src/domain/entity/foo.rs", "crate::usecase::Service"),
            make_raw("src/domain/entity/foo.rs", "std::fmt"),
            make_raw("src/domain/entity/foo.rs", "mystery"),
        ];
        let resolved = vec![
            make_resolved(
                "src/domain/entity/foo.rs",
                "crate::usecase::Service",
                ImportCategory::Internal,
            ),
            make_resolved("src/domain/entity/foo.rs", "std::fmt", ImportCategory::Stdlib),
            make_resolved(
                "src/domain/entity/foo.rs",
                "mystery",
                ImportCategory::Unknown,
            ),
        ];

        let result = report_external(
            "any.toml",
            &FixedConfigRepo(config),
            &FixedFileRepo(vec!["src/domain/entity/foo.rs".to_string()]),
            &FixedParser(raw_imports),
            &FixedResolver(resolved),
        )
        .unwrap();

        let domain = result.layers.iter().find(|l| l.layer_name == "domain").unwrap();
        assert!(domain.packages.is_empty(), "only External imports should be included");
    }

    #[test]
    fn test_packages_are_sorted() {
        let config = make_config(vec![make_layer("infra", &["src/infra/**"])]);

        let raw_imports = vec![
            make_raw("src/infra/a.rs", "toml::Value"),
            make_raw("src/infra/a.rs", "clap::Parser"),
            make_raw("src/infra/a.rs", "serde::Serialize"),
        ];
        let resolved = vec![
            make_resolved("src/infra/a.rs", "toml::Value", ImportCategory::External),
            make_resolved("src/infra/a.rs", "clap::Parser", ImportCategory::External),
            make_resolved("src/infra/a.rs", "serde::Serialize", ImportCategory::External),
        ];

        let result = report_external(
            "any.toml",
            &FixedConfigRepo(config),
            &FixedFileRepo(vec!["src/infra/a.rs".to_string()]),
            &FixedParser(raw_imports),
            &FixedResolver(resolved),
        )
        .unwrap();

        let infra = result.layers.iter().find(|l| l.layer_name == "infra").unwrap();
        assert_eq!(infra.packages, vec!["clap", "serde", "toml"]);
    }

    #[test]
    fn test_skips_files_not_in_any_layer() {
        // The file repo returns files outside any layer glob; they should be ignored.
        // (In practice SourceFileRepository only returns files matching the layer's own globs,
        //  so this tests the layer-assignment logic inside report_external.)
        let config = make_config(vec![make_layer("domain", &["src/domain/**"])]);

        let raw_imports = vec![make_raw("src/other/helper.rs", "serde::Deserialize")];
        let resolved = vec![make_resolved(
            "src/other/helper.rs",
            "serde::Deserialize",
            ImportCategory::External,
        )];

        let result = report_external(
            "any.toml",
            &FixedConfigRepo(config),
            // Return files outside domain/**
            &FixedFileRepo(vec!["src/other/helper.rs".to_string()]),
            &FixedParser(raw_imports),
            &FixedResolver(resolved),
        )
        .unwrap();

        // domain layer should have no packages — helper.rs is not in its glob
        let domain = result.layers.iter().find(|l| l.layer_name == "domain").unwrap();
        assert!(domain.packages.is_empty());
    }

    #[test]
    fn test_skips_mod_declarations() {
        let config = make_config(vec![make_layer("domain", &["src/domain/**"])]);

        let mod_import = RawImport {
            path: "entity".to_string(),
            line: 1,
            file: "src/domain/mod.rs".to_string(),
            kind: ImportKind::Mod, // pub mod entity;
            named_imports: vec![],
        };
        let resolved = vec![ResolvedImport {
            raw: mod_import.clone(),
            category: ImportCategory::External,
            resolved_path: None,
        }];

        struct ModParser(RawImport);
        impl Parser for ModParser {
            fn parse_imports(&self, _: &str, _: &str) -> Vec<RawImport> {
                vec![self.0.clone()]
            }
            fn parse_call_exprs(&self, _: &str, _: &str) -> Vec<RawCallExpr> {
                vec![]
            }
        }

        let result = report_external(
            "any.toml",
            &FixedConfigRepo(config),
            &FixedFileRepo(vec!["src/domain/mod.rs".to_string()]),
            &ModParser(mod_import),
            &FixedResolver(resolved),
        )
        .unwrap();

        let domain = result.layers.iter().find(|l| l.layer_name == "domain").unwrap();
        assert!(domain.packages.is_empty(), "pub mod X; must not appear in report");
    }

    #[test]
    fn test_empty_layers_return_empty_packages() {
        let config = make_config(vec![
            make_layer("domain", &["src/domain/**"]),
            make_layer("infra", &["src/infra/**"]),
        ]);

        let result = report_external(
            "any.toml",
            &FixedConfigRepo(config),
            &EmptyFileRepo,
            &NoOpParser,
            &NoOpResolver,
        )
        .unwrap();

        assert_eq!(result.layers.len(), 2);
        assert!(result.layers.iter().all(|l| l.packages.is_empty()));
    }
}
