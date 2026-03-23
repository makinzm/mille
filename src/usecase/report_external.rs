use std::collections::BTreeSet;

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
    let config = config_repo.load(config_path).map_err(|e| e.to_string())?;

    let mut all_resolved: Vec<crate::domain::entity::resolved_import::ResolvedImport> = Vec::new();

    for layer in &config.layers {
        let files = file_repo.collect(&layer.paths);
        for file_path in &files {
            let source = std::fs::read_to_string(file_path)
                .map_err(|e| format!("failed to read {}: {}", file_path, e))?;
            let raw_imports = parser.parse_imports(&source, file_path);
            for raw in &raw_imports {
                all_resolved.push(resolver.resolve_for_project(raw, &config.project.name));
            }
        }
    }

    let layers = compute_external_report(&all_resolved, &config.layers);
    Ok(ReportExternalResult { layers })
}

/// Pure computation: given resolved imports and layer configs, return external packages per layer.
///
/// Separated from the I/O pipeline to make it independently testable.
pub(crate) fn compute_external_report(
    imports: &[crate::domain::entity::resolved_import::ResolvedImport],
    layers: &[crate::domain::entity::layer::LayerConfig],
) -> Vec<LayerExternalReport> {
    let mut layer_packages: Vec<BTreeSet<String>> =
        (0..layers.len()).map(|_| BTreeSet::new()).collect();

    for import in imports {
        if import.raw.kind == ImportKind::Mod {
            continue;
        }
        if import.category != ImportCategory::External {
            continue;
        }
        let Some(layer_idx) = find_layer_idx_for_file(layers, &import.raw.file) else {
            continue;
        };
        let pkg = extract_package_name(&import.raw.path).to_string();
        layer_packages[layer_idx].insert(pkg);
    }

    layers
        .iter()
        .zip(layer_packages)
        .map(|(layer, pkgs)| LayerExternalReport {
            layer_name: layer.name.clone(),
            packages: pkgs.into_iter().collect(),
        })
        .collect()
}

fn find_layer_idx_for_file(
    layers: &[crate::domain::entity::layer::LayerConfig],
    file_path: &str,
) -> Option<usize> {
    layers.iter().position(|layer| {
        layer.paths.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .ok()
                .map(|p| p.matches(file_path))
                .unwrap_or(false)
        })
    })
}

/// Extract the top-level package name from a raw import path.
///
/// Uses the same strategy as `ViolationDetector::detect_external`:
/// split on `::` and take the first segment. This works for all import styles:
/// - Colon-separated: `serde::Deserialize` -> `serde`
/// - Slash-separated: `database/sql` -> `database/sql` (no `::`, whole string returned)
/// - Plain: `lodash` -> `lodash`, `sqlalchemy` -> `sqlalchemy`
fn extract_package_name(path: &str) -> &str {
    path.split("::").next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};
    use crate::domain::entity::layer::{DependencyMode, LayerConfig, NameTarget};
    use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};

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
            name_deny: vec![],
            name_allow: vec![],
            name_targets: NameTarget::all(),
            name_deny_ignore: vec![],
        }
    }

    fn make_resolved(file: &str, path: &str, category: ImportCategory) -> ResolvedImport {
        ResolvedImport {
            raw: RawImport {
                path: path.to_string(),
                line: 1,
                file: file.to_string(),
                kind: ImportKind::Use,
                named_imports: vec![],
            },
            category,
            resolved_path: None,
            package_name: None,
        }
    }

    // ------------------------------------------------------------------
    // extract_package_name
    // ------------------------------------------------------------------

    #[test]
    fn test_extract_package_name_colon_separated() {
        assert_eq!(extract_package_name("serde::Deserialize"), "serde");
    }

    #[test]
    fn test_extract_package_name_full_path() {
        assert_eq!(extract_package_name("database/sql"), "database/sql");
    }

    #[test]
    fn test_extract_package_name_plain() {
        assert_eq!(extract_package_name("sqlalchemy"), "sqlalchemy");
    }

    // ------------------------------------------------------------------
    // compute_external_report — pure unit tests
    // ------------------------------------------------------------------

    #[test]
    fn test_groups_packages_by_layer() {
        let layers = vec![
            make_layer("domain", &["src/domain/**"]),
            make_layer("infra", &["src/infra/**"]),
        ];
        let imports = vec![
            make_resolved(
                "src/domain/entity/foo.rs",
                "serde::Deserialize",
                ImportCategory::External,
            ),
            make_resolved("src/infra/repo.rs", "toml::Value", ImportCategory::External),
        ];

        let result = compute_external_report(&imports, &layers);

        let domain = result.iter().find(|l| l.layer_name == "domain").unwrap();
        let infra = result.iter().find(|l| l.layer_name == "infra").unwrap();
        assert_eq!(domain.packages, vec!["serde"]);
        assert_eq!(infra.packages, vec!["toml"]);
    }

    #[test]
    fn test_deduplicates_same_package() {
        let layers = vec![make_layer("domain", &["src/domain/**"])];
        let imports = vec![
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

        let result = compute_external_report(&imports, &layers);
        assert_eq!(result[0].packages, vec!["serde"]);
    }

    #[test]
    fn test_skips_non_external_imports() {
        let layers = vec![make_layer("domain", &["src/domain/**"])];
        let imports = vec![
            make_resolved(
                "src/domain/entity/foo.rs",
                "crate::usecase::Service",
                ImportCategory::Internal,
            ),
            make_resolved(
                "src/domain/entity/foo.rs",
                "std::fmt",
                ImportCategory::Stdlib,
            ),
            make_resolved(
                "src/domain/entity/foo.rs",
                "mystery",
                ImportCategory::Unknown,
            ),
        ];

        let result = compute_external_report(&imports, &layers);
        assert!(
            result[0].packages.is_empty(),
            "only External imports should be included"
        );
    }

    #[test]
    fn test_packages_are_sorted() {
        let layers = vec![make_layer("infra", &["src/infra/**"])];
        let imports = vec![
            make_resolved("src/infra/a.rs", "toml::Value", ImportCategory::External),
            make_resolved("src/infra/a.rs", "clap::Parser", ImportCategory::External),
            make_resolved(
                "src/infra/a.rs",
                "serde::Serialize",
                ImportCategory::External,
            ),
        ];

        let result = compute_external_report(&imports, &layers);
        assert_eq!(result[0].packages, vec!["clap", "serde", "toml"]);
    }

    #[test]
    fn test_skips_files_not_in_any_layer() {
        let layers = vec![make_layer("domain", &["src/domain/**"])];
        // file is in src/other — matches no layer glob
        let imports = vec![make_resolved(
            "src/other/helper.rs",
            "serde::Deserialize",
            ImportCategory::External,
        )];

        let result = compute_external_report(&imports, &layers);
        assert!(result[0].packages.is_empty());
    }

    #[test]
    fn test_skips_mod_declarations() {
        let layers = vec![make_layer("domain", &["src/domain/**"])];
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

        let result = compute_external_report(&imports, &layers);
        assert!(
            result[0].packages.is_empty(),
            "pub mod X; must not appear in report"
        );
    }

    #[test]
    fn test_empty_imports_all_layers_empty() {
        let layers = vec![
            make_layer("domain", &["src/domain/**"]),
            make_layer("infra", &["src/infra/**"]),
        ];

        let result = compute_external_report(&[], &layers);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|l| l.packages.is_empty()));
    }
}
