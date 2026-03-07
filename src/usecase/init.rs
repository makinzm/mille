/// A detected layer suggestion derived from directory scanning.
pub struct LayerSuggestion {
    pub name: String,
    pub paths: Vec<String>,
    pub dependency_mode: &'static str,
    pub allow: Vec<String>,
}

/// Scan `root` recursively (up to depth 3) and return layer suggestions
/// based on well-known directory name patterns.
pub fn scan_layers(root: &str) -> Vec<LayerSuggestion> {
    todo!("implement scan_layers")
}

/// Detect project languages from file extensions under `root`.
/// Returns a sorted, deduplicated list of language names.
pub fn detect_languages(root: &str) -> Vec<String> {
    todo!("implement detect_languages")
}

/// Generate a TOML config string (no side effects).
pub fn generate_toml(
    project_name: &str,
    root: &str,
    languages: &[String],
    layers: &[LayerSuggestion],
) -> String {
    todo!("implement generate_toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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
        let tmp = TempDir::new().unwrap();
        let result = scan_layers(tmp.path().to_str().unwrap());
        assert!(result.is_empty(), "no known layer dirs → empty vec");
    }

    #[test]
    fn test_scan_layers_detects_domain() {
        let tmp = TempDir::new().unwrap();
        make_dir(tmp.path(), "src/domain");
        let result = scan_layers(tmp.path().to_str().unwrap());
        assert!(
            result.iter().any(|l| l.name == "domain"),
            "src/domain should be detected as 'domain' layer"
        );
    }

    #[test]
    fn test_scan_layers_detects_multiple() {
        let tmp = TempDir::new().unwrap();
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
        let tmp = TempDir::new().unwrap();
        make_file(tmp.path(), "src/main.rs");
        let langs = detect_languages(tmp.path().to_str().unwrap());
        assert_eq!(langs, vec!["rust".to_string()]);
    }

    #[test]
    fn test_detect_languages_multiple() {
        let tmp = TempDir::new().unwrap();
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
