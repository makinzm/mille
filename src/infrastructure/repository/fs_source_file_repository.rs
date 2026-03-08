use crate::domain::repository::source_file_repository::SourceFileRepository;

/// Concrete implementation of the `SourceFileRepository` port.
/// Expands glob patterns and returns source file paths relative to the working directory.
/// Supported extensions: `.rs`, `.go`, `.py`, `.ts`, `.tsx`, `.js`, `.jsx`
pub struct FsSourceFileRepository;

const SOURCE_EXTENSIONS: &[&str] = &["rs", "go", "py", "ts", "tsx", "js", "jsx"];

fn is_source_file(path: &str) -> bool {
    SOURCE_EXTENSIONS
        .iter()
        .any(|ext| path.ends_with(&format!(".{}", ext)))
}

/// Returns true if the path contains an excluded directory component
/// (e.g. `.venv`, `node_modules`, `target`, etc.).
fn has_excluded_component(path: &str) -> bool {
    path.split('/').any(|seg| {
        matches!(
            seg,
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
        ) || (seg.starts_with('.') && seg.len() > 1)
            || seg.starts_with("flycheck")
    })
}

impl SourceFileRepository for FsSourceFileRepository {
    fn collect(&self, patterns: &[String]) -> Vec<String> {
        let mut files = Vec::new();
        for pattern in patterns {
            // Direct source file path (no glob characters).
            if is_source_file(pattern)
                && !pattern.contains('*')
                && !pattern.contains('?')
                && !pattern.contains('[')
            {
                if std::path::Path::new(pattern).exists() {
                    files.push(pattern.clone());
                }
                continue;
            }
            // Directory-based pattern (no glob characters): expand to all source files.
            // Handles "src/domain/**" → finds all source files under src/domain/.
            if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
                let base = pattern.trim_end_matches('/');
                for ext in SOURCE_EXTENSIONS {
                    for search in [
                        format!("{}/**/*.{}", base, ext),
                        format!("{}/*.{}", base, ext),
                    ] {
                        if let Ok(entries) = glob::glob(&search) {
                            files.extend(
                                entries
                                    .filter_map(|e| e.ok())
                                    .map(|p| p.to_string_lossy().to_string())
                                    .filter(|p| !has_excluded_component(p)),
                            );
                        }
                    }
                }
                continue;
            }
            if pattern.ends_with("/**") {
                // Strip /** and expand to <dir>/**/*.ext and <dir>/*.ext.
                let base = pattern.trim_end_matches("/**");
                for ext in SOURCE_EXTENSIONS {
                    for search in [
                        format!("{}/**/*.{}", base, ext),
                        format!("{}/*.{}", base, ext),
                    ] {
                        if let Ok(entries) = glob::glob(&search) {
                            files.extend(
                                entries
                                    .filter_map(|e| e.ok())
                                    .map(|p| p.to_string_lossy().to_string())
                                    .filter(|p| !has_excluded_component(p)),
                            );
                        }
                    }
                }
                continue;
            }
            // Other glob patterns (e.g. "*.go", "src/*.rs", "cmd/**/*.go"):
            // Use glob::glob directly and filter to supported source files.
            if let Ok(entries) = glob::glob(pattern) {
                files.extend(
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|p| is_source_file(&p.to_string_lossy()))
                        .map(|p| p.to_string_lossy().to_string())
                        .filter(|p| !has_excluded_component(p)),
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
    fn test_collect_skips_venv_paths() {
        // Create a temp dir with .venv/lib/fake.py — it must be excluded.
        let tmp = std::env::temp_dir().join(format!("mille_venv_test_{}", std::process::id()));
        std::fs::create_dir_all(tmp.join(".venv/lib")).unwrap();
        std::fs::write(tmp.join(".venv/lib/fake.py"), "# fake").unwrap();
        std::fs::create_dir_all(tmp.join("src")).unwrap();
        std::fs::write(tmp.join("src/app.py"), "# real").unwrap();

        let repo = FsSourceFileRepository;
        let pattern = format!("{}/**", tmp.to_string_lossy());
        let files = repo.collect(&[pattern]);

        // Cleanup
        let _ = std::fs::remove_dir_all(&tmp);

        assert!(
            files.iter().all(|f| !f.contains("/.venv/")),
            ".venv paths must be excluded, got: {:?}",
            files
        );
        assert!(
            files.iter().any(|f| f.ends_with("src/app.py")),
            "real source file must still be collected"
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
