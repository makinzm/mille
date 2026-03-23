use std::collections::BTreeSet;

use crate::domain::entity::layer::LayerConfig;
use crate::domain::repository::resolve_config_generator::ResolveConfigGenerator;

/// Default implementation of `ResolveConfigGenerator` that knows about
/// specific language resolve configurations (module paths, package prefixes,
/// import path packages, etc.).
pub struct DefaultResolveConfigGenerator {
    /// Module path name for full-module-path languages (e.g. from go.mod).
    pub module_path_name: Option<String>,
    /// Package prefix name for dot-separated-package languages (e.g. from pom.xml).
    pub package_prefix_name: Option<String>,
}

/// Return true if the string is a valid dotted-import-path identifier.
/// Identifiers start with a letter or underscore and contain only letters, digits,
/// and underscores. Directory names like "2026-03-01-ml-knowledge-base" are excluded.
fn is_valid_import_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

impl ResolveConfigGenerator for DefaultResolveConfigGenerator {
    fn generate_resolve_toml(&self, languages: &[String], layers: &[LayerConfig]) -> String {
        let mut out = String::new();

        // Full-module-path language (e.g. "go"): emit [resolve.go] with module_name
        let has_module_path_lang = languages.iter().any(|l| l == "go");
        if has_module_path_lang {
            if let Some(mn) = self.module_path_name.as_deref().filter(|m| !m.is_empty()) {
                out.push('\n');
                out.push_str("[resolve.go]\n");
                out.push_str(&format!("module_name = \"{}\"\n", mn));
            }
        }

        // Dot-separated-package language (e.g. "java", "kotlin"): emit [resolve.java]
        let has_package_prefix_lang = languages.iter().any(|l| l == "java" || l == "kotlin");
        if has_package_prefix_lang {
            if let Some(mn) = self
                .package_prefix_name
                .as_deref()
                .filter(|m| !m.is_empty())
            {
                out.push('\n');
                out.push_str("[resolve.java]\n");
                out.push_str(&format!("module_name = \"{}\"\n", mn));
            }
        }

        // Dot-separated-import-path language (e.g. "python"): emit [resolve.python]
        let has_import_path_lang = languages.iter().any(|l| l == "python");
        if has_import_path_lang {
            let pkg_names = self.internal_package_names(languages, layers);
            if !pkg_names.is_empty() {
                out.push('\n');
                out.push_str("[resolve.python]\n");
                let names_str = pkg_names
                    .iter()
                    .map(|n| format!("\"{}\"", n))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("package_names = [{}]\n", names_str));
            }
        }

        out
    }

    fn internal_package_names(
        &self,
        languages: &[String],
        layers: &[LayerConfig],
    ) -> BTreeSet<String> {
        let has_import_path_lang = languages.iter().any(|l| l == "python");
        if !has_import_path_lang {
            return BTreeSet::new();
        }

        // Base: last path component of each layer (e.g. "domain" from "src/domain/**").
        let base: BTreeSet<String> = layers
            .iter()
            .flat_map(|layer| layer.paths.iter())
            .filter_map(|path| {
                let p = path.trim_end_matches("/**").trim_end_matches('/');
                p.split('/').next_back().map(|s| s.to_string())
            })
            .filter(|s| is_valid_import_identifier(s))
            .collect();

        // All path directory components -- candidates for namespace package prefixes.
        let all_components: BTreeSet<String> = layers
            .iter()
            .flat_map(|layer| layer.paths.iter())
            .flat_map(|path| {
                let p = path.trim_end_matches("/**").trim_end_matches('/');
                p.split('/').map(|s| s.to_string()).collect::<Vec<_>>()
            })
            .filter(|s| is_valid_import_identifier(s))
            .collect();

        // If a path component appears in external_allow of any layer, real imports use it
        // as a top-level prefix (e.g. `from src.domain...`). Promote it to package_names
        // so it is classified as Internal and filtered out of external_allow.
        let namespace_pkgs: BTreeSet<String> = layers
            .iter()
            .flat_map(|layer| layer.external_allow.iter())
            .filter(|pkg| all_components.contains(pkg.as_str()))
            .cloned()
            .collect();

        base.into_iter().chain(namespace_pkgs).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::layer::{DependencyMode, NameTarget};

    fn make_layer(name: &str, paths: Vec<&str>) -> LayerConfig {
        LayerConfig {
            name: name.to_string(),
            paths: paths.into_iter().map(|p| p.to_string()).collect(),
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

    #[test]
    fn test_generate_resolve_toml_for_module_path_language() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: Some("github.com/example/myproject".to_string()),
            package_prefix_name: None,
        };
        let layers = vec![make_layer("domain", vec!["go/domain/**"])];
        let toml = gen.generate_resolve_toml(&["go".to_string()], &layers);
        assert!(toml.contains("[resolve.go]"));
        assert!(toml.contains("module_name = \"github.com/example/myproject\""));
    }

    #[test]
    fn test_generate_resolve_toml_for_package_prefix_language() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: Some("com.example.myapp".to_string()),
        };
        let layers = vec![make_layer("domain", vec!["**/domain/**"])];
        let toml = gen.generate_resolve_toml(&["java".to_string()], &layers);
        assert!(toml.contains("[resolve.java]"));
        assert!(toml.contains("module_name = \"com.example.myapp\""));
    }

    #[test]
    fn test_generate_resolve_toml_for_import_path_language() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: None,
        };
        let layers = vec![
            make_layer("domain", vec!["src/domain/**"]),
            make_layer("usecase", vec!["src/usecase/**"]),
        ];
        let toml = gen.generate_resolve_toml(&["python".to_string()], &layers);
        assert!(toml.contains("[resolve.python]"));
        assert!(toml.contains("package_names"));
        assert!(toml.contains("\"domain\""));
        assert!(toml.contains("\"usecase\""));
    }

    #[test]
    fn test_no_resolve_section_for_colon_path_language() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: None,
        };
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let toml = gen.generate_resolve_toml(&["rust".to_string()], &layers);
        assert!(toml.is_empty() || !toml.contains("[resolve"));
    }

    #[test]
    fn test_internal_package_names_from_layer_paths() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: None,
        };
        let layers = vec![
            make_layer("domain", vec!["src/domain/**"]),
            make_layer("usecase", vec!["src/usecase/**"]),
            make_layer("infrastructure", vec!["src/infrastructure/**"]),
        ];
        let names = gen.internal_package_names(&["python".to_string()], &layers);
        assert!(names.contains("domain"));
        assert!(names.contains("usecase"));
        assert!(names.contains("infrastructure"));
    }

    #[test]
    fn test_internal_package_names_empty_for_non_import_path_language() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: None,
        };
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let names = gen.internal_package_names(&["rust".to_string()], &layers);
        assert!(names.is_empty());
    }

    #[test]
    fn test_module_path_language_no_resolve_without_module_name() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: None,
        };
        let layers = vec![make_layer("domain", vec!["go/domain/**"])];
        let toml = gen.generate_resolve_toml(&["go".to_string()], &layers);
        assert!(!toml.contains("[resolve.go]"));
    }

    #[test]
    fn test_namespace_promotion_from_external_allow() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: None,
        };
        let mut layer = make_layer("src_domain", vec!["src/domain/**"]);
        layer.external_allow = vec!["src".to_string(), "dataclasses".to_string()];
        let layers = vec![layer];
        let names = gen.internal_package_names(&["python".to_string()], &layers);
        // "src" is a path component AND appears in external_allow, so it's promoted
        assert!(names.contains("src"));
        assert!(names.contains("domain"));
        // "dataclasses" is NOT a path component, so it stays in external_allow
        assert!(!names.contains("dataclasses"));
    }

    #[test]
    fn test_module_path_ignored_for_non_go_language() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: Some("github.com/example/ignored".to_string()),
            package_prefix_name: None,
        };
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let toml = gen.generate_resolve_toml(&["rust".to_string()], &layers);
        assert!(!toml.contains("[resolve.go]"));
    }

    #[test]
    fn test_package_prefix_ignored_for_non_java_language() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: Some("com.example.ignored".to_string()),
        };
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let toml = gen.generate_resolve_toml(&["rust".to_string()], &layers);
        assert!(!toml.contains("[resolve.java]"));
    }

    #[test]
    fn test_kotlin_triggers_package_prefix_resolve() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: Some("com.example.myapp".to_string()),
        };
        let layers = vec![make_layer("domain", vec!["**/domain/**"])];
        let toml = gen.generate_resolve_toml(&["kotlin".to_string()], &layers);
        assert!(toml.contains("[resolve.java]"));
        assert!(toml.contains("module_name = \"com.example.myapp\""));
    }

    #[test]
    fn test_monorepo_package_names_deduplicated() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: None,
        };
        let layers = vec![
            make_layer("crawler_domain", vec!["crawler/src/domain/**"]),
            make_layer("server_domain", vec!["server/src/domain/**"]),
            make_layer("crawler_usecase", vec!["crawler/src/usecase/**"]),
        ];
        let names = gen.internal_package_names(&["python".to_string()], &layers);
        // "domain" appears in two layers but should be deduplicated in BTreeSet
        assert!(names.contains("domain"));
        assert!(names.contains("usecase"));
    }

    #[test]
    fn test_package_prefix_without_module_name_produces_nothing() {
        let gen = DefaultResolveConfigGenerator {
            module_path_name: None,
            package_prefix_name: None,
        };
        let layers = vec![make_layer("domain", vec!["**/domain/**"])];
        let toml = gen.generate_resolve_toml(&["java".to_string()], &layers);
        assert!(!toml.contains("[resolve.java]"));
    }

    #[test]
    fn test_is_valid_import_identifier() {
        assert!(is_valid_import_identifier("domain"));
        assert!(is_valid_import_identifier("_private"));
        assert!(is_valid_import_identifier("my_pkg2"));
        assert!(!is_valid_import_identifier("2bad"));
        assert!(!is_valid_import_identifier("some-thing"));
        assert!(!is_valid_import_identifier(""));
    }
}
