use crate::domain::entity::violation::Violation;
use crate::domain::repository::config_repository::ConfigRepository;
use crate::domain::service::violation_detector::ViolationDetector;
use crate::infrastructure::parser::rust::{parse_rust_call_exprs, parse_rust_imports};
use crate::infrastructure::repository::toml_config_repository::TomlConfigRepository;
use crate::infrastructure::resolver;

pub struct CheckResult {
    pub violations: Vec<Violation>,
    pub layer_stats: Vec<LayerStat>,
}

pub struct LayerStat {
    pub name: String,
    pub file_count: usize,
    pub violation_count: usize,
}

/// Run the full check pipeline: load config → collect files → parse → resolve → detect.
pub fn check(config_path: &str) -> Result<CheckResult, String> {
    let config = TomlConfigRepository
        .load(config_path)
        .map_err(|e| e.to_string())?;

    let mut all_resolved = Vec::new();
    let mut all_call_exprs = Vec::new();
    let mut layer_stats: Vec<LayerStat> = config
        .layers
        .iter()
        .map(|l| LayerStat {
            name: l.name.clone(),
            file_count: 0,
            violation_count: 0,
        })
        .collect();

    for (idx, layer) in config.layers.iter().enumerate() {
        let files = collect_rust_files(&layer.paths);
        layer_stats[idx].file_count = files.len();
        for file_path in &files {
            let source = std::fs::read_to_string(file_path)
                .map_err(|e| format!("failed to read {}: {}", file_path, e))?;
            let raw = parse_rust_imports(&source, file_path);
            all_resolved.extend(raw.iter().map(resolver::rust::resolve));
            all_call_exprs.extend(parse_rust_call_exprs(&source, file_path));
        }
    }

    let detector = ViolationDetector::new(&config.layers);
    let mut violations = detector.detect(&all_resolved);
    violations.extend(detector.detect_call_patterns(&all_call_exprs, &all_resolved));

    for stat in &mut layer_stats {
        stat.violation_count = violations
            .iter()
            .filter(|v| v.from_layer == stat.name)
            .count();
    }

    Ok(CheckResult {
        violations,
        layer_stats,
    })
}

/// Expand layer path glob patterns into concrete `.rs` file paths.
fn collect_rust_files(patterns: &[String]) -> Vec<String> {
    let mut files = Vec::new();
    for pattern in patterns {
        if pattern.ends_with(".rs") {
            if std::path::Path::new(pattern).exists() {
                files.push(pattern.clone());
            }
            continue;
        }
        let base = pattern.trim_end_matches("/**").trim_end_matches('/');
        for search in [format!("{}/**/*.rs", base), format!("{}/*.rs", base)] {
            if let Ok(entries) = glob::glob(&search) {
                files.extend(
                    entries
                        .filter_map(|e| e.ok())
                        .map(|p| p.to_string_lossy().to_string()),
                );
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_rust_files_from_domain_pattern() {
        let files = collect_rust_files(&["src/domain/**".to_string()]);
        assert!(!files.is_empty(), "should find .rs files under src/domain/");
        assert!(
            files.iter().all(|f| f.ends_with(".rs")),
            "all results should be .rs files"
        );
        assert!(
            files.iter().any(|f| f.contains("src/domain/")),
            "paths should include src/domain/"
        );
    }

    #[test]
    fn test_collect_rust_files_specific_file() {
        let files = collect_rust_files(&["src/main.rs".to_string()]);
        assert_eq!(files, vec!["src/main.rs".to_string()]);
    }

    #[test]
    fn test_collect_rust_files_nonexistent_returns_empty() {
        let files = collect_rust_files(&["src/nonexistent_layer/**".to_string()]);
        assert!(files.is_empty());
    }

    #[test]
    fn test_collect_rust_files_deduplicates() {
        // Same pattern listed twice should not return duplicates.
        let files = collect_rust_files(&["src/domain/**".to_string(), "src/domain/**".to_string()]);
        let mut sorted = files.clone();
        sorted.dedup();
        assert_eq!(files.len(), sorted.len(), "duplicates should be removed");
    }

    // ------------------------------------------------------------------
    // Integration / Dogfooding
    // ------------------------------------------------------------------

    #[test]
    fn test_dogfood_check_mille_toml() {
        let result = check("mille.toml").expect("should load mille.toml without error");
        assert!(
            result.violations.is_empty(),
            "mille should not violate its own architecture rules.\nViolations found:\n{:#?}",
            result.violations
        );
    }

    #[test]
    fn test_check_result_has_layer_stats() {
        let result = check("mille.toml").expect("should load mille.toml without error");
        assert!(
            !result.layer_stats.is_empty(),
            "check result should include per-layer statistics"
        );
    }
}
