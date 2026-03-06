use std::collections::HashMap;

use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// Concrete implementation of the `Resolver` port for TypeScript/JavaScript imports.
///
/// Classification rules:
/// - **Alias**: import path matches a tsconfig `paths` alias → resolved to Internal
/// - **Internal**: relative imports starting with `./` or `../`
/// - **External**: everything else (npm packages, Node.js built-ins like `node:fs`, etc.)
///
/// For internal imports, a `resolved_path` is computed so the ViolationDetector can
/// match it against layer glob patterns (e.g. `domain/**`).
///
/// Example:
/// - file `usecase/user_usecase.ts`, import `../domain/user`
///   → resolved_path = `domain/user/_.ts`  (matches layer glob `domain/**`)
/// - file `src/usecase/user_usecase.ts`, import `@/domain/user`, alias `@/*` → `./src/*`
///   → resolved_path = `src/domain/user/_.ts`
pub struct TypeScriptResolver {
    /// Flattened tsconfig `compilerOptions.paths` mapping.
    /// Key: alias pattern (e.g. `"@/*"`), Value: first target (e.g. `"./src/*"`).
    aliases: HashMap<String, String>,
}

impl TypeScriptResolver {
    pub fn new() -> Self {
        TypeScriptResolver {
            aliases: HashMap::new(),
        }
    }

    pub fn with_aliases(aliases: HashMap<String, String>) -> Self {
        TypeScriptResolver { aliases }
    }
}

impl Default for TypeScriptResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver for TypeScriptResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        resolve_ts_impl(import, &self.aliases)
    }

    fn resolve_for_project(&self, import: &RawImport, _own_crate: &str) -> ResolvedImport {
        resolve_ts_impl(import, &self.aliases)
    }
}

fn resolve_ts_impl(import: &RawImport, aliases: &HashMap<String, String>) -> ResolvedImport {
    // Try alias resolution first (e.g. `@/domain/user` → `src/domain/user`).
    if let Some(expanded) = resolve_alias(&import.path, aliases) {
        let normalized = normalize_path(&expanded);
        let clean = normalized.trim_start_matches("./");
        return ResolvedImport {
            raw: import.clone(),
            category: ImportCategory::Internal,
            resolved_path: Some(format!("{}/_.ts", clean)),
        };
    }

    let category = classify_ts(&import.path);
    let resolved_path = if category == ImportCategory::Internal {
        compute_resolved_path(&import.file, &import.path)
    } else {
        None
    };
    ResolvedImport {
        raw: import.clone(),
        category,
        resolved_path,
    }
}

/// Expand an import path using tsconfig-style alias patterns.
///
/// Supports wildcard patterns: `"@/*"` with target `"./src/*"` expands
/// `"@/domain/user"` → `"./src/domain/user"`.
///
/// Returns `None` if no alias matches.
fn resolve_alias(path: &str, aliases: &HashMap<String, String>) -> Option<String> {
    for (pattern, target) in aliases {
        if let Some(prefix) = pattern.strip_suffix("/*") {
            // Wildcard alias: "@/*" matches "@/..." with prefix "@"
            let expected_prefix = format!("{}/", prefix);
            if path.starts_with(&expected_prefix) {
                let rest = &path[expected_prefix.len()..];
                let target_base = target.trim_end_matches("/*");
                return Some(format!("{}/{}", target_base, rest));
            }
        } else if path == pattern {
            // Exact alias
            return Some(target.clone());
        }
    }
    None
}

/// Classify a TypeScript/JavaScript import path.
///
/// - Relative imports (starting with `./` or `../`) → Internal
/// - Everything else → External
pub fn classify_ts(path: &str) -> ImportCategory {
    if path.starts_with("./") || path.starts_with("../") {
        ImportCategory::Internal
    } else {
        ImportCategory::External
    }
}

/// Compute a resolved path for internal (relative) imports.
///
/// Given the source file path and a relative import path, computes the canonical
/// path so the ViolationDetector can match it against layer glob patterns.
///
/// Example:
/// - file: `usecase/user_usecase.ts`, import: `../domain/user`
/// - file_dir: `usecase`
/// - joined: `usecase/../domain/user` → normalized: `domain/user`
/// - resolved_path: `domain/user/_.ts`
fn compute_resolved_path(file: &str, import_path: &str) -> Option<String> {
    let file_dir = std::path::Path::new(file).parent()?;

    // Join the file's directory with the relative import path and normalize.
    let joined = file_dir.join(import_path);
    let normalized = normalize_path(&joined.to_string_lossy());

    // Strip leading "./" if present after normalization.
    let clean = normalized.trim_start_matches("./");

    // Use a generic extension for the resolved path since the actual extension
    // may vary (.ts, .tsx, .js, .jsx). The layer glob patterns use ** so any
    // extension will match.
    Some(format!("{}/_.ts", clean))
}

/// Normalize a path string by resolving `.` and `..` components.
///
/// Does not perform any filesystem access. Returns the normalized path as a String.
fn normalize_path(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }
    if segments.is_empty() {
        ".".to_string()
    } else {
        segments.join("/")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};

    fn raw_ts(path: &str, file: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: file.to_string(),
            kind: ImportKind::Import,
            named_imports: vec![],
        }
    }

    #[test]
    fn test_alias_wildcard_resolves_to_internal() {
        let mut aliases = HashMap::new();
        aliases.insert("@/*".to_string(), "./src/*".to_string());
        let resolver = TypeScriptResolver::with_aliases(aliases);
        let import = raw_ts("@/domain/user", "src/usecase/user_usecase.ts");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("src/domain/user/_.ts".to_string())
        );
    }

    #[test]
    fn test_alias_no_match_falls_through_to_external() {
        let mut aliases = HashMap::new();
        aliases.insert("@/*".to_string(), "./src/*".to_string());
        let resolver = TypeScriptResolver::with_aliases(aliases);
        let import = raw_ts("react", "src/usecase/user_usecase.ts");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
    }

    #[test]
    fn test_relative_dotslash_is_internal() {
        assert_eq!(classify_ts("./user"), ImportCategory::Internal);
        assert_eq!(classify_ts("./domain/user"), ImportCategory::Internal);
    }

    #[test]
    fn test_relative_dotdot_is_internal() {
        assert_eq!(classify_ts("../domain/user"), ImportCategory::Internal);
        assert_eq!(classify_ts("../../shared/types"), ImportCategory::Internal);
    }

    #[test]
    fn test_npm_package_is_external() {
        assert_eq!(classify_ts("react"), ImportCategory::External);
        assert_eq!(classify_ts("some-lib"), ImportCategory::External);
        assert_eq!(classify_ts("@types/node"), ImportCategory::External);
    }

    #[test]
    fn test_node_builtin_is_external() {
        assert_eq!(classify_ts("node:fs"), ImportCategory::External);
        assert_eq!(classify_ts("node:path"), ImportCategory::External);
        assert_eq!(classify_ts("fs"), ImportCategory::External);
    }

    #[test]
    fn test_resolved_path_parent_dir() {
        let resolver = TypeScriptResolver::new();
        let import = raw_ts("../domain/user", "usecase/user_usecase.ts");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(resolved.resolved_path, Some("domain/user/_.ts".to_string()));
    }

    #[test]
    fn test_resolved_path_same_dir() {
        let resolver = TypeScriptResolver::new();
        let import = raw_ts("./entity", "domain/user.ts");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("domain/entity/_.ts".to_string())
        );
    }

    #[test]
    fn test_external_has_no_resolved_path() {
        let resolver = TypeScriptResolver::new();
        let import = raw_ts("node:fs", "infrastructure/db.ts");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_normalize_path_dotdot() {
        assert_eq!(normalize_path("usecase/../domain/user"), "domain/user");
        assert_eq!(normalize_path("a/b/../../c"), "c");
    }

    #[test]
    fn test_normalize_path_dot() {
        assert_eq!(normalize_path("domain/./user"), "domain/user");
    }
}
