use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// A detected layer suggestion derived from directory scanning.
pub struct LayerSuggestion {
    pub name: String,
    pub paths: Vec<String>,
    pub dependency_mode: &'static str,
    pub allow: Vec<String>,
}

/// Known directory name → (layer name, dependency_mode, allow list) mapping.
/// Order matters: entries checked in order; first match wins per directory.
const KNOWN_LAYERS: &[(&str, &str, &str, &[&str])] = &[
    ("domain", "domain", "opt-in", &[]),
    ("model", "domain", "opt-in", &[]),
    ("entities", "domain", "opt-in", &[]),
    ("entity", "domain", "opt-in", &[]),
    ("usecase", "usecase", "opt-in", &["domain"]),
    ("application", "usecase", "opt-in", &["domain"]),
    ("use_case", "usecase", "opt-in", &["domain"]),
    ("usecases", "usecase", "opt-in", &["domain"]),
    ("infrastructure", "infrastructure", "opt-out", &[]),
    ("infra", "infrastructure", "opt-out", &[]),
    ("adapter", "infrastructure", "opt-out", &[]),
    ("adapters", "infrastructure", "opt-out", &[]),
    (
        "presentation",
        "presentation",
        "opt-in",
        &["usecase", "domain"],
    ),
    ("handler", "presentation", "opt-in", &["usecase", "domain"]),
    ("handlers", "presentation", "opt-in", &["usecase", "domain"]),
    (
        "controller",
        "presentation",
        "opt-in",
        &["usecase", "domain"],
    ),
    ("api", "presentation", "opt-in", &["usecase", "domain"]),
];

/// Scan `root` recursively (up to depth 3) and return layer suggestions
/// based on well-known directory name patterns.
/// Each logical layer name appears at most once; the first path found wins.
pub fn scan_layers(root: &str) -> Vec<LayerSuggestion> {
    let root_path = Path::new(root);

    // Collect all directory paths up to depth 3
    let mut dirs: Vec<(String, String)> = Vec::new(); // (dir_name, relative_path)
    collect_dirs(root_path, root_path, 0, 3, &mut dirs);

    // Match dirs against known layer patterns; deduplicate by layer name
    let mut seen_names: BTreeSet<String> = BTreeSet::new();
    // We want a stable ordering: use insertion order via Vec
    let mut suggestions: Vec<LayerSuggestion> = Vec::new();

    for (dir_name, rel_path) in &dirs {
        if let Some((_, layer_name, dep_mode, allow)) = KNOWN_LAYERS
            .iter()
            .find(|(pattern, _, _, _)| *pattern == dir_name.as_str())
        {
            if seen_names.insert(layer_name.to_string()) {
                suggestions.push(LayerSuggestion {
                    name: layer_name.to_string(),
                    paths: vec![format!("{}/**", rel_path)],
                    dependency_mode: dep_mode,
                    allow: allow.iter().map(|s| s.to_string()).collect(),
                });
            }
        }
    }

    // Sort suggestions in a stable architectural order
    let order = ["domain", "usecase", "infrastructure", "presentation"];
    suggestions.sort_by_key(|s| {
        order
            .iter()
            .position(|n| *n == s.name.as_str())
            .unwrap_or(usize::MAX)
    });

    suggestions
}

/// Recursively collect (dir_name, relative_path_from_root) pairs up to `max_depth`.
fn collect_dirs(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    out: &mut Vec<(String, String)>,
) {
    if depth >= max_depth {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        // Skip hidden dirs and common non-source dirs
        if dir_name.starts_with('.') || dir_name == "target" || dir_name == "node_modules" {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        out.push((dir_name.clone(), rel));
        collect_dirs(root, &path, depth + 1, max_depth, out);
    }
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
        if name.starts_with('.') || name == "target" || name == "node_modules" {
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

/// Generate a TOML config string (no side effects).
pub fn generate_toml(
    project_name: &str,
    root: &str,
    languages: &[String],
    layers: &[LayerSuggestion],
) -> String {
    let langs_toml = languages
        .iter()
        .map(|l| format!("\"{}\"", l))
        .collect::<Vec<_>>()
        .join(", ");

    let mut out = format!(
        "[project]\nname = \"{}\"\nroot = \"{}\"\nlanguages = [{}]\n",
        project_name, root, langs_toml
    );

    for layer in layers {
        let paths_toml = layer
            .paths
            .iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(", ");

        out.push('\n');
        out.push_str("[[layers]]\n");
        out.push_str(&format!("name = \"{}\"\n", layer.name));
        out.push_str(&format!("paths = [{}]\n", paths_toml));
        out.push_str(&format!(
            "dependency_mode = \"{}\"\n",
            layer.dependency_mode
        ));

        if layer.dependency_mode == "opt-in" {
            let allow_toml = layer
                .allow
                .iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("allow = [{}]\n", allow_toml));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Minimal RAII temp-dir using only stdlib (avoids tempfile dev-dependency
    /// inside src/, which would be flagged as an ExternalViolation by mille).
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            // Use process id + label to avoid collisions between parallel tests.
            let dir = std::env::temp_dir().join(format!(
                "mille_init_test_{}_{}",
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

    fn make_dir(base: &std::path::Path, rel: &str) {
        fs::create_dir_all(base.join(rel)).unwrap();
    }

    fn make_file(base: &std::path::Path, rel: &str) {
        let path = base.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }

    // ------------------------------------------------------------------
    // scan_layers
    // ------------------------------------------------------------------

    #[test]
    fn test_scan_layers_empty_dir() {
        let tmp = TempDir::new("scan_empty");
        let result = scan_layers(tmp.path().to_str().unwrap());
        assert!(result.is_empty(), "no known layer dirs → empty vec");
    }

    #[test]
    fn test_scan_layers_detects_domain() {
        let tmp = TempDir::new("scan_domain");
        make_dir(tmp.path(), "src/domain");
        let result = scan_layers(tmp.path().to_str().unwrap());
        assert!(
            result.iter().any(|l| l.name == "domain"),
            "src/domain should be detected as 'domain' layer"
        );
    }

    #[test]
    fn test_scan_layers_detects_multiple() {
        let tmp = TempDir::new("scan_multiple");
        make_dir(tmp.path(), "src/domain");
        make_dir(tmp.path(), "src/usecase");
        make_dir(tmp.path(), "src/infrastructure");
        let result = scan_layers(tmp.path().to_str().unwrap());
        let names: Vec<&str> = result.iter().map(|l| l.name.as_str()).collect();
        assert!(names.contains(&"domain"), "domain should be detected");
        assert!(names.contains(&"usecase"), "usecase should be detected");
        assert!(
            names.contains(&"infrastructure"),
            "infrastructure should be detected"
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
        assert!(langs.contains(&"rust".to_string()), "should detect rust");
        assert!(
            langs.contains(&"typescript".to_string()),
            "should detect typescript"
        );
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
        }];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers);
        assert!(
            toml.contains("[project]"),
            "generated TOML must contain [project] section"
        );
    }

    #[test]
    fn test_generate_toml_contains_layer_sections() {
        let layers = vec![LayerSuggestion {
            name: "domain".to_string(),
            paths: vec!["src/domain/**".to_string()],
            dependency_mode: "opt-in",
            allow: vec![],
        }];
        let toml = generate_toml("myproject", ".", &["rust".to_string()], &layers);
        assert!(
            toml.contains("[[layers]]"),
            "generated TOML must contain [[layers]] section"
        );
    }
}
