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
    if deps.is_empty() {
        return vec![];
    }

    // in_degree[node] = number of node's known dependencies (what it imports from)
    // Nodes with in_degree 0 are "leaves" (nothing they depend on is known) → tier 0
    let mut in_degree: BTreeMap<&str, usize> = BTreeMap::new();
    for (node, node_deps) in deps {
        let known_count = node_deps
            .iter()
            .filter(|d| deps.contains_key(d.as_str()))
            .count();
        in_degree.insert(node.as_str(), known_count);
    }

    let mut remaining: BTreeSet<&str> = deps.keys().map(|k| k.as_str()).collect();
    let mut tiers: Vec<Vec<String>> = vec![];

    loop {
        let mut tier: Vec<&str> = remaining
            .iter()
            .copied()
            .filter(|n| in_degree[n] == 0)
            .collect();
        tier.sort();

        if tier.is_empty() {
            break;
        }

        for &node in &tier {
            remaining.remove(node);
            // Decrement in_degree for every node that imports from this node
            for (candidate, candidate_deps) in deps {
                if candidate_deps.contains(node) && remaining.contains(candidate.as_str()) {
                    if let Some(deg) = in_degree.get_mut(candidate.as_str()) {
                        *deg = deg.saturating_sub(1);
                    }
                }
            }
        }

        tiers.push(tier.into_iter().map(|s| s.to_string()).collect());
    }

    // Remaining nodes are in cycles — collect into a final tier
    if !remaining.is_empty() {
        let mut cycle_tier: Vec<String> = remaining.iter().map(|s| s.to_string()).collect();
        cycle_tier.sort();
        tiers.push(cycle_tier);
    }

    tiers
}

/// Infer layer suggestions from per-directory import analysis.
///
/// Algorithm:
/// 1. Build internal dep graph (only known dirs).
/// 2. Topological sort → tiers.
/// 3. Within each tier, group dirs by base name.
/// 4. Within each base-name group, sub-group by immediate parent's base name.
///    - All sub-groups share the same parent base → one layer named `{base}`.
///    - Multiple different parent bases → one layer per sub-group named `{parent}_{base}`.
///      (e.g. domain/entity + infrastructure/entity → domain_entity + infrastructure_entity)
/// 5. Each resulting group → one LayerSuggestion; allow list uses qualified layer names.
pub fn infer_layers(analyses: &BTreeMap<String, DirAnalysis>) -> Vec<LayerSuggestion> {
    if analyses.is_empty() {
        return vec![];
    }

    // Build internal dep graph: dir_path → set of dir_paths it imports from (known dirs only)
    let known_dirs: BTreeSet<&str> = analyses.keys().map(|k| k.as_str()).collect();
    let dep_graph: BTreeMap<String, BTreeSet<String>> = analyses
        .iter()
        .map(|(dir, analysis)| {
            let internal_only: BTreeSet<String> = analysis
                .internal_deps
                .iter()
                .filter(|d| known_dirs.contains(d.as_str()))
                .cloned()
                .collect();
            (dir.clone(), internal_only)
        })
        .collect();

    let tiers = topological_sort(&dep_graph);

    // Pass 1: assign a layer name to every dir.
    // Dirs with the same base name are merged only when their immediate parent
    // also shares the same base name (e.g. monorepo siblings under the same "src" parent).
    // Otherwise each gets a qualified name: "{parent_base}_{base}".
    //
    // NOTE: We group across ALL tiers so that dirs in different tiers but with the same
    // base name are still compared (e.g. domain/entity in tier 0 vs infrastructure/entity
    // in tier 1 are both named "entity" → qualified to "domain_entity" / "infrastructure_entity").
    let mut dir_to_layer: BTreeMap<String, String> = BTreeMap::new();

    // Collect all dirs across all tiers, group by base name
    let mut by_base: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for dir in tiers.iter().flatten() {
        let base = dir
            .split('/')
            .next_back()
            .unwrap_or(dir.as_str())
            .to_string();
        by_base.entry(base).or_default().push(dir.clone());
    }

    for (base_name, dirs) in &by_base {
        // Sub-group by immediate parent's base name
        let mut by_parent: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for dir in dirs {
            let parent_base = dir
                .rsplit_once('/')
                .map(|(parent, _)| parent.split('/').next_back().unwrap_or(""))
                .unwrap_or("")
                .to_string();
            by_parent.entry(parent_base).or_default().push(dir.clone());
        }

        let needs_prefix = by_parent.len() > 1;
        for (parent_base, group_dirs) in &by_parent {
            let layer_name = if needs_prefix && !parent_base.is_empty() {
                format!("{}_{}", parent_base, base_name)
            } else {
                base_name.clone()
            };
            for dir in group_dirs {
                dir_to_layer.insert(dir.clone(), layer_name.clone());
            }
        }
    }

    // Pass 2: build LayerSuggestion for each (tier, layer_name) group.
    let mut suggestions: Vec<LayerSuggestion> = vec![];

    for tier in &tiers {
        // Group dirs by their assigned layer name within this tier
        let mut by_layer: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for dir in tier {
            let layer_name = dir_to_layer.get(dir).cloned().unwrap_or_else(|| {
                dir.split('/')
                    .next_back()
                    .unwrap_or(dir.as_str())
                    .to_string()
            });
            by_layer.entry(layer_name).or_default().push(dir.clone());
        }

        for (name, paths) in by_layer {
            // Collect allow: qualified layer names of dirs this group depends on
            let mut allow_names: BTreeSet<String> = BTreeSet::new();
            for path in &paths {
                if let Some(analysis) = analyses.get(path) {
                    for dep in &analysis.internal_deps {
                        if known_dirs.contains(dep.as_str()) {
                            if let Some(dep_layer) = dir_to_layer.get(dep) {
                                if dep_layer != &name {
                                    allow_names.insert(dep_layer.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Collect external_allow: all external packages used by any path in this group
            let mut external_allow: BTreeSet<String> = BTreeSet::new();
            for path in &paths {
                if let Some(analysis) = analyses.get(path) {
                    external_allow.extend(analysis.external_pkgs.iter().cloned());
                }
            }

            suggestions.push(LayerSuggestion {
                name,
                paths,
                dependency_mode: "opt-in",
                allow: allow_names.into_iter().collect(),
                external_allow: external_allow.into_iter().collect(),
            });
        }
    }

    suggestions
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
    let mut out = String::new();

    // [project] section
    out.push_str("[project]\n");
    out.push_str(&format!("name = \"{}\"\n", project_name));
    out.push_str(&format!("root = \"{}\"\n", root));
    let langs_str = languages
        .iter()
        .map(|l| format!("\"{}\"", l))
        .collect::<Vec<_>>()
        .join(", ");
    out.push_str(&format!("languages = [{}]\n", langs_str));

    // [[layers]] sections
    for layer in layers {
        out.push('\n');
        out.push_str("[[layers]]\n");
        out.push_str(&format!("name = \"{}\"\n", layer.name));

        // paths: single-line if one path, array if multiple
        if layer.paths.len() == 1 {
            out.push_str(&format!("paths = [\"{}\"]", layer.paths[0]));
        } else {
            out.push_str("paths = [\n");
            for path in &layer.paths {
                out.push_str(&format!("  \"{}\",\n", path));
            }
            out.push(']');
        }
        out.push('\n');

        out.push_str(&format!(
            "dependency_mode = \"{}\"\n",
            layer.dependency_mode
        ));

        if !layer.allow.is_empty() {
            let allow_str = layer
                .allow
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("allow = [{}]\n", allow_str));
        }

        if !layer.external_allow.is_empty() {
            let ext_str = layer
                .external_allow
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("external_allow = [{}]\n", ext_str));
        }
    }

    out
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
            .map(|(k, vs)| (k.to_string(), vs.iter().map(|v| v.to_string()).collect()))
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
        assert_eq!(
            layers.len(),
            1,
            "two domain dirs should merge into one layer"
        );
        assert_eq!(layers[0].name, "domain");
        assert_eq!(layers[0].paths.len(), 2);
    }

    #[test]
    fn test_infer_layers_disambiguates_same_name_different_parent() {
        // domain/entity and infrastructure/entity → different parents → separate layers
        let mut analyses = BTreeMap::new();
        analyses.insert(
            "src/domain/entity".to_string(),
            DirAnalysis {
                internal_deps: BTreeSet::new(),
                external_pkgs: BTreeSet::new(),
                file_count: 2,
            },
        );
        analyses.insert(
            "src/infrastructure/entity".to_string(),
            DirAnalysis {
                internal_deps: BTreeSet::new(),
                external_pkgs: BTreeSet::new(),
                file_count: 2,
            },
        );
        let layers = infer_layers(&analyses);
        assert_eq!(layers.len(), 2, "different parents → two separate layers");
        let names: Vec<&str> = layers.iter().map(|l| l.name.as_str()).collect();
        assert!(
            names.contains(&"domain_entity"),
            "expected domain_entity, got {:?}",
            names
        );
        assert!(
            names.contains(&"infrastructure_entity"),
            "expected infrastructure_entity, got {:?}",
            names
        );
    }

    #[test]
    fn test_infer_layers_allow_uses_qualified_name() {
        // infrastructure/entity depends on domain/entity
        // → allow should reference "domain_entity", not just "entity"
        let mut analyses = BTreeMap::new();
        analyses.insert(
            "src/domain/entity".to_string(),
            DirAnalysis {
                internal_deps: BTreeSet::new(),
                external_pkgs: BTreeSet::new(),
                file_count: 1,
            },
        );
        let mut infra_deps = BTreeSet::new();
        infra_deps.insert("src/domain/entity".to_string());
        analyses.insert(
            "src/infrastructure/entity".to_string(),
            DirAnalysis {
                internal_deps: infra_deps,
                external_pkgs: BTreeSet::new(),
                file_count: 1,
            },
        );
        let layers = infer_layers(&analyses);
        let infra = layers
            .iter()
            .find(|l| l.name == "infrastructure_entity")
            .expect("infrastructure_entity layer should exist");
        assert!(
            infra.allow.contains(&"domain_entity".to_string()),
            "allow should use qualified name 'domain_entity', got {:?}",
            infra.allow
        );
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
