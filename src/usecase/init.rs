use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::domain::entity::layer::{DependencyMode, LayerConfig};

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

/// Given a list of directory paths that all share the same base name,
/// return `(dir_path, layer_name)` pairs where each dir gets a unique name.
///
/// When only one dir is present the base name is used as-is.
/// When multiple dirs exist, the first path segment where they differ
/// is prepended as a prefix: e.g. "crawler_domain", "server_domain".
fn find_distinguishing_prefix(dirs: &[String]) -> Vec<(String, String)> {
    let base = dirs[0]
        .split('/')
        .next_back()
        .unwrap_or(dirs[0].as_str())
        .to_string();

    if dirs.len() == 1 {
        return vec![(dirs[0].clone(), base)];
    }

    // Collect parent segments (everything except the last base component) for each dir.
    // Segments are stored root-first so index 0 is the top-level directory.
    let parent_segs: Vec<Vec<&str>> = dirs
        .iter()
        .map(|d| {
            let parts: Vec<&str> = d.split('/').collect();
            if parts.len() > 1 {
                parts[..parts.len() - 1].to_vec()
            } else {
                vec![]
            }
        })
        .collect();

    // Find the first position (root-first) where at least two dirs differ.
    let depth = parent_segs.iter().map(|s| s.len()).min().unwrap_or(0);
    let mut diff_pos: Option<usize> = None;
    for i in 0..depth {
        let first = parent_segs[0].get(i);
        if parent_segs.iter().any(|s| s.get(i) != first) {
            diff_pos = Some(i);
            break;
        }
    }

    dirs.iter()
        .enumerate()
        .map(|(idx, dir)| {
            let prefix = match diff_pos {
                Some(pos) => parent_segs[idx].get(pos).copied().unwrap_or(""),
                // All common parents identical (or no parents): use last parent segment.
                None => parent_segs[idx].last().copied().unwrap_or(""),
            };
            let name = if prefix.is_empty() {
                base.clone()
            } else {
                format!("{}_{}", prefix, base)
            };
            (dir.clone(), name)
        })
        .collect()
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
/// 5. Each resulting group → one LayerConfig; allow list uses qualified layer names.
pub fn infer_layers(analyses: &BTreeMap<String, DirAnalysis>) -> Vec<LayerConfig> {
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
    // Every dir with a unique base name is named by its base name.
    // When multiple dirs share the same base name, each gets a qualified name
    // using the first path segment where they differ as a prefix.
    //
    // Examples:
    //   ["src/domain/entity", "src/infrastructure/entity"]
    //     → parent segs differ at position 1 (domain vs infrastructure)
    //     → "domain_entity", "infrastructure_entity"
    //
    //   ["apps/crawler/src/domain", "apps/server/src/domain"]
    //     → parent segs differ at position 1 (crawler vs server)
    //     → "crawler_domain", "server_domain"
    //
    // NOTE: We group across ALL tiers so that dirs in different tiers but with the same
    // base name are still compared.
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

    for dirs in by_base.values() {
        for (dir, layer_name) in find_distinguishing_prefix(dirs) {
            dir_to_layer.insert(dir, layer_name);
        }
    }

    // Pass 2: build LayerConfig for each (tier, layer_name) group.
    let mut suggestions: Vec<LayerConfig> = vec![];

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

            suggestions.push(LayerConfig {
                name,
                paths,
                dependency_mode: DependencyMode::OptIn,
                allow: allow_names.into_iter().collect(),
                deny: vec![],
                external_mode: DependencyMode::OptIn,
                external_allow: external_allow.into_iter().collect(),
                external_deny: vec![],
                allow_call_patterns: vec![],
            });
        }
    }

    suggestions
}

/// Return true if the string is a valid Python identifier (used to filter package names).
/// Python identifiers start with a letter or underscore and contain only letters, digits,
/// and underscores. Directory names like "2026-03-01-ml-knowledge-base" are excluded.
fn is_valid_python_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_')
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

fn mode_str(m: DependencyMode) -> &'static str {
    match m {
        DependencyMode::OptIn => "opt-in",
        DependencyMode::OptOut => "opt-out",
    }
}

/// Generate a mille.toml config string (pure function, no side effects).
pub fn generate_toml(
    project_name: &str,
    root: &str,
    languages: &[String],
    layers: &[LayerConfig],
    go_module_name: Option<&str>,
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

    // [resolve.go] — Go プロジェクトかつ module_name が判明している場合のみ出力
    let is_go = languages.iter().any(|l| l == "go");
    if is_go {
        if let Some(mn) = go_module_name.filter(|m| !m.is_empty()) {
            out.push('\n');
            out.push_str("[resolve.go]\n");
            out.push_str(&format!("module_name = \"{}\"\n", mn));
        }
    }

    // Derive Python package names from layer path base directories.
    // Used for [resolve.python] and to filter internal names from external_allow.
    let is_python = languages.iter().any(|l| l == "python");
    let py_pkg_names: BTreeSet<String> = if is_python {
        layers
            .iter()
            .flat_map(|layer| layer.paths.iter())
            .filter_map(|path| {
                let p = path.trim_end_matches("/**").trim_end_matches('/');
                p.split('/').next_back().map(|s| s.to_string())
            })
            .filter(|s| is_valid_python_identifier(s))
            .collect()
    } else {
        BTreeSet::new()
    };

    // [resolve.python] — Python プロジェクトの場合のみ出力
    if is_python && !py_pkg_names.is_empty() {
        out.push('\n');
        out.push_str("[resolve.python]\n");
        let names_str = py_pkg_names
            .iter()
            .map(|n| format!("\"{}\"", n))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("package_names = [{}]\n", names_str));
    }

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
            mode_str(layer.dependency_mode)
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

        out.push_str(&format!(
            "external_mode = \"{}\"\n",
            mode_str(layer.external_mode)
        ));

        // Python の場合、package_names に含まれる名前は内部パッケージなので external_allow から除外
        let filtered_external: Vec<&String> = layer
            .external_allow
            .iter()
            .filter(|e| !py_pkg_names.contains(*e))
            .collect();
        if !filtered_external.is_empty() {
            let ext_str = filtered_external
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
    use crate::domain::entity::layer::{DependencyMode, LayerConfig};
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
        // Two sub-projects both have a "domain" dir with different parents →
        // they must NOT be merged; each gets a distinguishing prefix.
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
            2,
            "two domain dirs from different sub-projects must be separate layers, got {:?}",
            layers.iter().map(|l| &l.name).collect::<Vec<_>>()
        );
        let names: Vec<&str> = layers.iter().map(|l| l.name.as_str()).collect();
        assert!(
            names.contains(&"crawler_domain") || names.contains(&"server_domain"),
            "layers should have distinguishing prefixes, got {:?}",
            names
        );
    }

    #[test]
    fn test_infer_layers_separate_same_name_dirs_different_subproject() {
        // crawler/src/domain + ingest/src/domain + server/src/domain → 3 separate layers
        let mut analyses = BTreeMap::new();
        for sub in &["crawler", "ingest", "server"] {
            analyses.insert(
                format!("apps/{}/src/domain", sub),
                DirAnalysis {
                    internal_deps: BTreeSet::new(),
                    external_pkgs: BTreeSet::new(),
                    file_count: 1,
                },
            );
        }
        let layers = infer_layers(&analyses);
        assert_eq!(
            layers.len(),
            3,
            "each sub-project domain must be a separate layer, got {:?}",
            layers.iter().map(|l| &l.name).collect::<Vec<_>>()
        );
        let names: Vec<String> = layers.iter().map(|l| l.name.clone()).collect();
        assert!(
            names.contains(&"crawler_domain".to_string()),
            "expected crawler_domain, got {:?}",
            names
        );
        assert!(
            names.contains(&"ingest_domain".to_string()),
            "expected ingest_domain, got {:?}",
            names
        );
        assert!(
            names.contains(&"server_domain".to_string()),
            "expected server_domain, got {:?}",
            names
        );
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
        }
    }

    #[test]
    fn test_generate_toml_contains_project_section() {
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers, None);
        assert!(toml.contains("[project]"), "must contain [project]");
    }

    #[test]
    fn test_generate_toml_contains_layer_sections() {
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers, None);
        assert!(toml.contains("[[layers]]"), "must contain [[layers]]");
    }

    #[test]
    fn test_generate_toml_includes_external_mode() {
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers, None);
        assert!(
            toml.contains("external_mode = \"opt-in\""),
            "must contain external_mode\n{}",
            toml
        );
    }

    #[test]
    fn test_generate_toml_with_external_allow() {
        let mut layer = make_layer("infrastructure", vec!["src/infrastructure/**"]);
        layer.external_allow = vec!["serde".to_string(), "tokio".to_string()];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &[layer], None);
        assert!(
            toml.contains("external_allow"),
            "must contain external_allow\n{}",
            toml
        );
        assert!(toml.contains("serde"), "must include serde\n{}", toml);
    }

    #[test]
    fn test_generate_toml_multi_path_format() {
        let layers = vec![make_layer(
            "domain",
            vec!["apps/crawler/src/domain/**", "apps/server/src/domain/**"],
        )];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers, None);
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
    // generate_toml — resolve.python
    // ------------------------------------------------------------------

    #[test]
    fn test_generate_toml_python_adds_resolve_section() {
        // Python プロジェクトでは [resolve.python] package_names が出力されるべき
        let layers = vec![
            make_layer("domain", vec!["src/domain/**"]),
            make_layer("usecase", vec!["src/usecase/**"]),
            make_layer("infrastructure", vec!["src/infrastructure/**"]),
        ];
        let toml = generate_toml("myproject", ".", &["python".to_string()], &layers, None);
        assert!(
            toml.contains("[resolve.python]"),
            "Python プロジェクトは [resolve.python] を含むべき\n{}",
            toml
        );
        assert!(
            toml.contains("package_names"),
            "package_names フィールドが必要\n{}",
            toml
        );
        assert!(
            toml.contains("\"domain\""),
            "domain が含まれるべき\n{}",
            toml
        );
        assert!(
            toml.contains("\"usecase\""),
            "usecase が含まれるべき\n{}",
            toml
        );
        assert!(
            toml.contains("\"infrastructure\""),
            "infrastructure が含まれるべき\n{}",
            toml
        );
    }

    #[test]
    fn test_generate_toml_rust_no_resolve_section() {
        // Rust プロジェクトでは [resolve.python] は出力されない
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers, None);
        assert!(
            !toml.contains("[resolve.python]"),
            "Rust プロジェクトに [resolve.python] は不要\n{}",
            toml
        );
    }

    #[test]
    fn test_generate_toml_python_monorepo_package_names_deduplicated() {
        // 複数サブプロジェクトで同じ base name が重複しても一度だけ出力
        let layers = vec![
            make_layer("crawler_domain", vec!["crawler/src/domain/**"]),
            make_layer("server_domain", vec!["server/src/domain/**"]),
            make_layer("crawler_usecase", vec!["crawler/src/usecase/**"]),
        ];
        let toml = generate_toml("myproject", ".", &["python".to_string()], &layers, None);
        // "domain" は1回だけ
        let domain_count = toml.matches("\"domain\"").count();
        assert_eq!(domain_count, 1, "domain は重複なし。toml:\n{}", toml);
    }

    #[test]
    fn test_generate_toml_python_filters_package_names_from_external_allow() {
        // external_allow に package_names と同じ名前が混入しないべき
        // (mille init スキャン時に "domain.entity" が External 扱いされて domain が混入するケース)
        let mut domain_layer = make_layer("domain", vec!["src/domain/**"]);
        domain_layer.external_allow = vec![
            "domain".to_string(),
            "abc".to_string(),
            "dataclasses".to_string(),
        ];
        let layers = vec![domain_layer];
        let toml = generate_toml("myproject", ".", &["python".to_string()], &layers, None);
        // "domain" は package_names に含まれるので external_allow から除外されるべき
        // abc, dataclasses は残るべき
        // external_allow の行に "domain" が含まれないことを確認
        let has_domain_in_ext_allow = toml
            .lines()
            .any(|line| line.contains("external_allow") && line.contains("\"domain\""));
        assert!(
            !has_domain_in_ext_allow,
            "domain は external_allow に出力されるべきでない\n{}",
            toml
        );
        let has_abc_in_ext_allow = toml
            .lines()
            .any(|line| line.contains("external_allow") && line.contains("\"abc\""));
        assert!(
            has_abc_in_ext_allow,
            "abc は external_allow に残るべき\n{}",
            toml
        );
    }

    // ------------------------------------------------------------------
    // generate_toml — resolve.go
    // ------------------------------------------------------------------

    #[test]
    fn test_generate_toml_go_adds_resolve_section() {
        // Go プロジェクトで module_name が渡された場合 [resolve.go] が出力される
        let layers = vec![make_layer("domain", vec!["go/domain/**"])];
        let toml = generate_toml(
            "myproject",
            ".",
            &["go".to_string()],
            &layers,
            Some("github.com/example/myproject"),
        );
        assert!(
            toml.contains("[resolve.go]"),
            "Go プロジェクトは [resolve.go] を含むべき\n{}",
            toml
        );
        assert!(
            toml.contains("module_name = \"github.com/example/myproject\""),
            "module_name が出力されるべき\n{}",
            toml
        );
    }

    #[test]
    fn test_generate_toml_go_no_resolve_without_module_name() {
        // Go プロジェクトでも module_name が None なら [resolve.go] は出力しない
        let layers = vec![make_layer("domain", vec!["go/domain/**"])];
        let toml = generate_toml("myproject", ".", &["go".to_string()], &layers, None);
        assert!(
            !toml.contains("[resolve.go]"),
            "module_name なしは [resolve.go] を出力しない\n{}",
            toml
        );
    }

    #[test]
    fn test_generate_toml_rust_no_resolve_go_section() {
        // Rust プロジェクトでは go_module_name を渡しても [resolve.go] は出力しない
        let layers = vec![make_layer("domain", vec!["src/domain/**"])];
        let toml = generate_toml(
            "myproject",
            ".",
            &["rust".to_string()],
            &layers,
            Some("github.com/example/ignored"),
        );
        assert!(
            !toml.contains("[resolve.go]"),
            "Rust プロジェクトに [resolve.go] は不要\n{}",
            toml
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
