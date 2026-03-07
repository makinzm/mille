use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

/// Per-directory import analysis, built externally by the infrastructure layer.
/// This is a plain data type — no I/O here.
#[derive(Default)]
pub struct DirAnalysis {
    /// Relative dir paths (from project root) this dir directly imports from.
    pub internal_deps: BTreeSet<String>,
    /// External package/crate names imported by files in this dir.
    pub external_pkgs: BTreeSet<String>,
    /// Number of source files in this directory.
    pub file_count: usize,
}

/// An inferred layer suggestion.
pub struct LayerSuggestion {
    pub name: String,
    pub paths: Vec<String>,
    pub dependency_mode: &'static str,
    pub allow: Vec<String>,
    pub external_allow: Vec<String>,
}

/// Topological sort of directories based on their internal dependency edges.
///
/// Returns dirs grouped by tier:
/// - tier[0] = "leaf" dirs — no internal deps (domain-like, imported by others)
/// - tier[N] = dirs all of whose deps are resolved in lower tiers (presentation-like)
///
/// Cycles are collected into a final tier so the function never panics.
pub fn topological_sort(deps: &BTreeMap<String, BTreeSet<String>>) -> Vec<Vec<String>> {
    todo!("implement topological_sort")
}

/// Infer layer suggestions from per-directory import analysis.
///
/// Algorithm:
/// 1. Build internal dep graph (only known dirs).
/// 2. Topological sort → tiers.
/// 3. Group dirs in the same tier by base name (last path segment).
/// 4. Each group → one LayerSuggestion with combined paths + allow + external_allow.
pub fn infer_layers(analyses: &BTreeMap<String, DirAnalysis>) -> Vec<LayerSuggestion> {
    todo!("implement infer_layers")
}

/// Return true if a directory name should be skipped during scanning.
pub fn is_excluded_dir(name: &str) -> bool {
    matches!(
        name,
        "target"
            | "node_modules"
            | "dist"
            | "build"
            | "out"
            | "__pycache__"
            | ".venv"
            | "venv"
            | "vendor"
            | "coverage"
            | ".next"
            | ".nuxt"
            | "migration"
            | "migrations"
    ) || name.starts_with('.')
        || name.starts_with("flycheck")
}

/// Detect project languages from file extensions under `root`.
/// Returns a sorted, deduplicated list of language names.
pub fn detect_languages(root: &str) -> Vec<String> {
    let mut langs: BTreeSet<String> = BTreeSet::new();
    collect_languages(Path::new(root), &mut langs);
    langs.into_iter().collect()
}

fn collect_languages(dir: &Path, langs: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if is_excluded_dir(name) {
            continue;
        }
        if path.is_dir() {
            collect_languages(&path, langs);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if let Some(lang) = ext_to_language(ext) {
                langs.insert(lang.to_string());
            }
        }
    }
}

fn ext_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("rust"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "go" => Some("go"),
        "py" => Some("python"),
        _ => None,
    }
}

/// Generate a mille.toml config string (pure function, no side effects).
pub fn generate_toml(
    project_name: &str,
    root: &str,
    languages: &[String],
    layers: &[LayerSuggestion],
) -> String {
    todo!("implement generate_toml")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::PathBuf;

    // ------------------------------------------------------------------
    // Stdlib-only RAII temp dir (avoids tempfile external dep in usecase)
    // ------------------------------------------------------------------

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "mille_init2_test_{}_{}",
                std::process::id(),
                label
            ));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &PathBuf {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn make_file(base: &std::path::Path, rel: &str) {
        let path = base.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }

    fn btree(pairs: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
        pairs
            .iter()
            .map(|(k, vs)| {
                (
                    k.to_string(),
                    vs.iter().map(|v| v.to_string()).collect(),
                )
            })
            .collect()
    }

    // ------------------------------------------------------------------
    // topological_sort
    // ------------------------------------------------------------------

    #[test]
    fn test_topological_sort_empty() {
        let result = topological_sort(&BTreeMap::new());
        assert!(result.is_empty());
    }

    #[test]
    fn test_topological_sort_single_dir_no_deps() {
        let deps = btree(&[("domain", &[])]);
        let tiers = topological_sort(&deps);
        assert_eq!(tiers.len(), 1);
        assert_eq!(tiers[0], vec!["domain"]);
    }

    #[test]
    fn test_topological_sort_chain() {
        // usecase → domain: domain first, usecase second
        let deps = btree(&[("domain", &[]), ("usecase", &["domain"])]);
        let tiers = topological_sort(&deps);
        assert_eq!(tiers.len(), 2);
        assert_eq!(tiers[0], vec!["domain"]);
        assert_eq!(tiers[1], vec!["usecase"]);
    }

    #[test]
    fn test_topological_sort_three_tier_chain() {
        // presentation → usecase → domain
        let deps = btree(&[
            ("domain", &[]),
            ("usecase", &["domain"]),
            ("presentation", &["usecase"]),
        ]);
        let tiers = topological_sort(&deps);
        assert_eq!(tiers.len(), 3);
        assert_eq!(tiers[0], vec!["domain"]);
        assert_eq!(tiers[1], vec!["usecase"]);
        assert_eq!(tiers[2], vec!["presentation"]);
    }

    #[test]
    fn test_topological_sort_diamond() {
        // usecase → domain, infra → domain, presentation → usecase + infra
        let deps = btree(&[
            ("domain", &[]),
            ("infra", &["domain"]),
            ("usecase", &["domain"]),
            ("presentation", &["usecase", "infra"]),
        ]);
        let tiers = topological_sort(&deps);
        assert_eq!(tiers[0], vec!["domain"]);
        // infra and usecase are both at tier 1
        assert_eq!(tiers[1], vec!["infra", "usecase"]);
        assert_eq!(tiers[2], vec!["presentation"]);
    }

    #[test]
    fn test_topological_sort_cycle_does_not_panic() {
        // a → b, b → a: cycle
        let deps = btree(&[("a", &["b"]), ("b", &["a"])]);
        let tiers = topological_sort(&deps);
        // Must not panic; cycle members end up in some tier
        assert!(!tiers.is_empty());
        let all_dirs: Vec<String> = tiers.into_iter().flatten().collect();
        assert!(all_dirs.contains(&"a".to_string()));
        assert!(all_dirs.contains(&"b".to_string()));
    }

    #[test]
    fn test_topological_sort_unknown_deps_ignored() {
        // usecase depends on "ghost" which is not in the map
        let deps = btree(&[("domain", &[]), ("usecase", &["domain", "ghost"])]);
        let tiers = topological_sort(&deps);
        // ghost is not a known dir, so usecase only waits on domain
        assert_eq!(tiers[0], vec!["domain"]);
        assert_eq!(tiers[1], vec!["usecase"]);
    }

    // ------------------------------------------------------------------
    // infer_layers
    // ------------------------------------------------------------------

    #[test]
    fn test_infer_layers_empty() {
        let result = infer_layers(&BTreeMap::new());
        assert!(result.is_empty());
    }

    #[test]
    fn test_infer_layers_single_dir_no_deps() {
        let mut analyses = BTreeMap::new();
        analyses.insert(
            "src/domain".to_string(),
            DirAnalysis {
                internal_deps: BTreeSet::new(),
                external_pkgs: BTreeSet::new(),
                file_count: 1,
            },
        );
        let layers = infer_layers(&analyses);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].name, "domain");
        assert!(layers[0].allow.is_empty());
        assert!(layers[0].external_allow.is_empty());
    }

    #[test]
    fn test_infer_layers_chain_domain_usecase() {
        let mut analyses = BTreeMap::new();
        analyses.insert(
            "src/domain".to_string(),
            DirAnalysis {
                internal_deps: BTreeSet::new(),
                external_pkgs: BTreeSet::new(),
                file_count: 1,
            },
        );
        let mut usecase_deps = BTreeSet::new();
        usecase_deps.insert("src/domain".to_string());
        analyses.insert(
            "src/usecase".to_string(),
            DirAnalysis {
                internal_deps: usecase_deps,
                external_pkgs: BTreeSet::new(),
                file_count: 1,
            },
        );
        let layers = infer_layers(&analyses);
        assert_eq!(layers.len(), 2);
        // domain comes first (tier 0)
        assert_eq!(layers[0].name, "domain");
        // usecase comes second (tier 1) and allows domain
        assert_eq!(layers[1].name, "usecase");
        assert!(layers[1].allow.contains(&"domain".to_string()));
    }

    #[test]
    fn test_infer_layers_groups_dirs_by_base_name() {
        // Two sub-projects both have a "domain" dir → one merged layer
        let mut analyses = BTreeMap::new();
        analyses.insert(
            "apps/crawler/src/domain".to_string(),
            DirAnalysis {
                internal_deps: BTreeSet::new(),
                external_pkgs: BTreeSet::new(),
                file_count: 2,
            },
        );
        analyses.insert(
            "apps/server/src/domain".to_string(),
            DirAnalysis {
                internal_deps: BTreeSet::new(),
                external_pkgs: BTreeSet::new(),
                file_count: 3,
            },
        );
        let layers = infer_layers(&analyses);
        assert_eq!(layers.len(), 1, "two domain dirs should merge into one layer");
        assert_eq!(layers[0].name, "domain");
        assert_eq!(layers[0].paths.len(), 2);
    }

    #[test]
    fn test_infer_layers_collects_external_deps() {
        let mut analyses = BTreeMap::new();
        let mut ext = BTreeSet::new();
        ext.insert("serde".to_string());
        ext.insert("tokio".to_string());
        analyses.insert(
            "src/infrastructure".to_string(),
            DirAnalysis {
                internal_deps: BTreeSet::new(),
                external_pkgs: ext,
                file_count: 2,
            },
        );
        let layers = infer_layers(&analyses);
        assert!(layers[0].external_allow.contains(&"serde".to_string()));
        assert!(layers[0].external_allow.contains(&"tokio".to_string()));
    }

    // ------------------------------------------------------------------
    // generate_toml
    // ------------------------------------------------------------------

    #[test]
    fn test_generate_toml_contains_project_section() {
        let layers = vec![LayerSuggestion {
            name: "domain".to_string(),
            paths: vec!["src/domain/**".to_string()],
            dependency_mode: "opt-in",
            allow: vec![],
            external_allow: vec![],
        }];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers);
        assert!(toml.contains("[project]"), "must contain [project]");
    }

    #[test]
    fn test_generate_toml_contains_layer_sections() {
        let layers = vec![LayerSuggestion {
            name: "domain".to_string(),
            paths: vec!["src/domain/**".to_string()],
            dependency_mode: "opt-in",
            allow: vec![],
            external_allow: vec![],
        }];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers);
        assert!(toml.contains("[[layers]]"), "must contain [[layers]]");
    }

    #[test]
    fn test_generate_toml_with_external_allow() {
        let layers = vec![LayerSuggestion {
            name: "infrastructure".to_string(),
            paths: vec!["src/infrastructure/**".to_string()],
            dependency_mode: "opt-in",
            allow: vec![],
            external_allow: vec!["serde".to_string(), "tokio".to_string()],
        }];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers);
        assert!(
            toml.contains("external_allow"),
            "must contain external_allow\n{}",
            toml
        );
        assert!(toml.contains("serde"), "must include serde\n{}", toml);
    }

    #[test]
    fn test_generate_toml_multi_path_format() {
        let layers = vec![LayerSuggestion {
            name: "domain".to_string(),
            paths: vec![
                "apps/crawler/src/domain/**".to_string(),
                "apps/server/src/domain/**".to_string(),
            ],
            dependency_mode: "opt-in",
            allow: vec![],
            external_allow: vec![],
        }];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers);
        // With multiple paths, each should appear on its own line
        assert!(
            toml.contains("apps/crawler/src/domain/**"),
            "first path missing"
        );
        assert!(
            toml.contains("apps/server/src/domain/**"),
            "second path missing"
        );
    }

    // ------------------------------------------------------------------
    // detect_languages
    // ------------------------------------------------------------------

    #[test]
    fn test_detect_languages_rust() {
        let tmp = TempDir::new("lang_rust");
        make_file(tmp.path(), "src/main.rs");
        let langs = detect_languages(tmp.path().to_str().unwrap());
        assert_eq!(langs, vec!["rust".to_string()]);
    }

    #[test]
    fn test_detect_languages_multiple() {
        let tmp = TempDir::new("lang_multi");
        make_file(tmp.path(), "src/main.rs");
        make_file(tmp.path(), "src/index.ts");
        let langs = detect_languages(tmp.path().to_str().unwrap());
        assert!(langs.contains(&"rust".to_string()));
        assert!(langs.contains(&"typescript".to_string()));
    }

    #[test]
    fn test_is_excluded_dir_skips_known() {
        assert!(is_excluded_dir("target"));
        assert!(is_excluded_dir("node_modules"));
        assert!(is_excluded_dir("dist"));
        assert!(is_excluded_dir(".git"));
        assert!(is_excluded_dir("flycheck_some_file"));
        assert!(!is_excluded_dir("domain"));
        assert!(!is_excluded_dir("usecase"));
        assert!(!is_excluded_dir("infrastructure"));
    }
}
