use crate::domain::repository::source_file_repository::SourceFileRepository;

/// Concrete implementation of the `SourceFileRepository` port.
/// Expands glob patterns and returns `.rs` and `.go` file paths relative to the working directory.
pub struct FsSourceFileRepository;

impl SourceFileRepository for FsSourceFileRepository {
    fn collect(&self, patterns: &[String]) -> Vec<String> {
        let mut files = Vec::new();
        for pattern in patterns {
            // Direct .rs / .go file path (no glob characters).
            if (pattern.ends_with(".rs") || pattern.ends_with(".go"))
                && !pattern.contains('*')
                && !pattern.contains('?')
                && !pattern.contains('[')
            {
                if std::path::Path::new(pattern).exists() {
                    files.push(pattern.clone());
                }
                continue;
            }
            // Directory-based pattern that ends with /** or is a plain directory name.
            // Strip trailing /** and expand to <dir>/**/*.ext and <dir>/*.ext.
            // This handles the common case: "src/domain/**" → finds all .rs/.go files.
            if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
                let base = pattern.trim_end_matches('/');
                for ext in ["rs", "go"] {
                    for search in [
                        format!("{}/**/*.{}", base, ext),
                        format!("{}/*.{}", base, ext),
                    ] {
                        if let Ok(entries) = glob::glob(&search) {
                            files.extend(
                                entries
                                    .filter_map(|e| e.ok())
                                    .map(|p| p.to_string_lossy().to_string()),
                            );
                        }
                    }
                }
                continue;
            }
            if pattern.ends_with("/**") {
                // Strip /** and expand to <dir>/**/*.ext and <dir>/*.ext.
                let base = pattern.trim_end_matches("/**");
                for ext in ["rs", "go"] {
                    for search in [
                        format!("{}/**/*.{}", base, ext),
                        format!("{}/*.{}", base, ext),
                    ] {
                        if let Ok(entries) = glob::glob(&search) {
                            files.extend(
                                entries
                                    .filter_map(|e| e.ok())
                                    .map(|p| p.to_string_lossy().to_string()),
                            );
                        }
                    }
                }
                continue;
            }
            // Other glob patterns (e.g. "*.go", "src/*.rs", "cmd/**/*.go"):
            // Use glob::glob directly and filter to .rs / .go files.
            if let Ok(entries) = glob::glob(pattern) {
                files.extend(
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|p| {
                            let s = p.to_string_lossy();
                            s.ends_with(".rs") || s.ends_with(".go")
                        })
                        .map(|p| p.to_string_lossy().to_string()),
                );
            }
        }
        files.sort();
        files.dedup();
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collects_rs_files_from_pattern() {
        let repo = FsSourceFileRepository;
        let files = repo.collect(&["src/domain/**".to_string()]);
        assert!(!files.is_empty(), "should find .rs files under src/domain/");
        assert!(files.iter().all(|f| f.ends_with(".rs")));
        assert!(files.iter().any(|f| f.contains("src/domain/")));
    }

    #[test]
    fn test_collects_specific_file() {
        let repo = FsSourceFileRepository;
        let files = repo.collect(&["src/main.rs".to_string()]);
        assert_eq!(files, vec!["src/main.rs".to_string()]);
    }

    #[test]
    fn test_nonexistent_pattern_returns_empty() {
        let repo = FsSourceFileRepository;
        let files = repo.collect(&["src/nonexistent_layer/**".to_string()]);
        assert!(files.is_empty());
    }

    #[test]
    fn test_deduplicates_overlapping_patterns() {
        let repo = FsSourceFileRepository;
        let files = repo.collect(&["src/domain/**".to_string(), "src/domain/**".to_string()]);
        let mut sorted = files.clone();
        sorted.dedup();
        assert_eq!(files.len(), sorted.len(), "duplicates must be removed");
    }

    #[test]
    fn test_collects_go_files_from_pattern() {
        let repo = FsSourceFileRepository;
        let files = repo.collect(&["tests/fixtures/go_sample/domain/**".to_string()]);
        assert!(
            !files.is_empty(),
            "should find .go files under go_sample/domain/"
        );
        assert!(files.iter().any(|f| f.ends_with(".go")));
    }

    #[test]
    fn test_collects_specific_go_file() {
        let repo = FsSourceFileRepository;
        let files = repo.collect(&["tests/fixtures/go_sample/domain/user.go".to_string()]);
        assert_eq!(
            files,
            vec!["tests/fixtures/go_sample/domain/user.go".to_string()]
        );
    }

    #[test]
    fn test_collects_files_with_bare_glob_pattern() {
        // "*.go" should match all .go files in the current directory.
        // This is important for single-layer projects where all source files
        // live in the project root (e.g. packages/go/).
        let repo = FsSourceFileRepository;
        // We can't rely on CWD having .go files, so use a directory-relative glob.
        let files = repo.collect(&["tests/fixtures/go_sample/domain/*.go".to_string()]);
        assert!(
            !files.is_empty(),
            "bare glob '*.go' must match .go files in the directory"
        );
        assert!(
            files.iter().all(|f| f.ends_with(".go")),
            "all matched files must be .go files"
        );
    }
}
