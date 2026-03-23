use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// Concrete implementation of the `Resolver` port for Rust imports.
pub struct RustResolver;

impl Resolver for RustResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        resolve_impl(import, "")
    }

    fn resolve_for_project(&self, import: &RawImport, own_crate: &str) -> ResolvedImport {
        resolve_impl(import, own_crate)
    }
}

/// Resolve a Rust `RawImport` into a categorised `ResolvedImport`.
/// `own_crate` is the project's own crate name (e.g. `"mille"`); paths starting
/// with `<own_crate>::` are treated as Internal, matching the behaviour of `crate::`.
pub fn resolve(import: &RawImport) -> ResolvedImport {
    resolve_impl(import, "")
}

fn resolve_impl(import: &RawImport, own_crate: &str) -> ResolvedImport {
    let category = classify(&import.path, own_crate);
    let resolved_path = if category == ImportCategory::Internal {
        resolve_crate_path(&import.path, own_crate)
    } else {
        None
    };
    let package_name = if category == ImportCategory::External {
        Some(
            import
                .path
                .split("::")
                .next()
                .unwrap_or(&import.path)
                .to_string(),
        )
    } else {
        None
    };
    ResolvedImport {
        raw: import.clone(),
        category,
        resolved_path,
        package_name,
    }
}

/// Classify the import category from its path string.
/// `own_crate` — when non-empty, paths starting with `<own_crate>::` are treated
/// as Internal (this handles `use mille::infrastructure::…` in `main.rs`).
fn classify(path: &str, own_crate: &str) -> ImportCategory {
    if path.starts_with("std::") || path.starts_with("core::") || path.starts_with("alloc::") {
        return ImportCategory::Stdlib;
    }
    if path.starts_with("crate::") || path.starts_with("super::") || path.starts_with("self::") {
        return ImportCategory::Internal;
    }
    if !own_crate.is_empty() {
        let prefix = format!("{}::", own_crate);
        if path.starts_with(&prefix) {
            return ImportCategory::Internal;
        }
    }
    ImportCategory::External
}

/// Normalise a `crate::` or `<own_crate>::` path to a file-system path relative to the repo root.
/// Returns `None` for wildcards, grouped imports, or unresolvable paths.
fn resolve_crate_path(path: &str, own_crate: &str) -> Option<String> {
    let relative = if let Some(r) = path.strip_prefix("crate::") {
        r
    } else if !own_crate.is_empty() {
        let prefix = format!("{}::", own_crate);
        path.strip_prefix(prefix.as_str())?
    } else {
        return None;
    };
    // Grouped imports (e.g. `crate::domain::{a, b}`) or wildcards cannot map to a single file.
    if relative.contains('{') || relative.contains('*') {
        return None;
    }
    Some(format!("src/{}", relative.replace("::", "/")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};
    use crate::domain::entity::resolved_import::ImportCategory;

    fn raw(path: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: "src/any/file.rs".to_string(),
            kind: ImportKind::Use,
            named_imports: vec![],
        }
    }

    // ------------------------------------------------------------------
    // classify
    // ------------------------------------------------------------------

    #[test]
    fn test_std_is_stdlib() {
        assert_eq!(classify("std::fs", ""), ImportCategory::Stdlib);
        assert_eq!(classify("std::io::Write", ""), ImportCategory::Stdlib);
    }

    #[test]
    fn test_core_is_stdlib() {
        assert_eq!(classify("core::fmt", ""), ImportCategory::Stdlib);
    }

    #[test]
    fn test_alloc_is_stdlib() {
        assert_eq!(classify("alloc::vec::Vec", ""), ImportCategory::Stdlib);
    }

    #[test]
    fn test_crate_is_internal() {
        assert_eq!(
            classify("crate::domain::entity::config", ""),
            ImportCategory::Internal
        );
    }

    #[test]
    fn test_super_is_internal() {
        assert_eq!(classify("super::something", ""), ImportCategory::Internal);
    }

    #[test]
    fn test_self_is_internal() {
        assert_eq!(classify("self::helper", ""), ImportCategory::Internal);
    }

    #[test]
    fn test_external_crate_is_external() {
        assert_eq!(classify("serde::Deserialize", ""), ImportCategory::External);
        assert_eq!(classify("toml::from_str", ""), ImportCategory::External);
    }

    #[test]
    fn test_own_crate_is_internal() {
        // When own_crate = "mille", paths like mille::infrastructure:: are Internal.
        assert_eq!(
            classify("mille::infrastructure::parser::rust::RustParser", "mille"),
            ImportCategory::Internal
        );
        // Without own_crate context, same path is External.
        assert_eq!(
            classify("mille::infrastructure::parser::rust::RustParser", ""),
            ImportCategory::External
        );
    }

    // ------------------------------------------------------------------
    // resolve_crate_path
    // ------------------------------------------------------------------

    #[test]
    fn test_simple_crate_path() {
        assert_eq!(
            resolve_crate_path("crate::domain::entity::config", ""),
            Some("src/domain/entity/config".to_string())
        );
    }

    #[test]
    fn test_crate_path_single_segment() {
        assert_eq!(
            resolve_crate_path("crate::main", ""),
            Some("src/main".to_string())
        );
    }

    #[test]
    fn test_grouped_import_returns_none() {
        assert_eq!(
            resolve_crate_path("crate::domain::{entity, repository}", ""),
            None
        );
    }

    #[test]
    fn test_wildcard_import_returns_none() {
        assert_eq!(resolve_crate_path("crate::domain::*", ""), None);
    }

    #[test]
    fn test_non_crate_path_returns_none() {
        assert_eq!(resolve_crate_path("serde::Deserialize", ""), None);
        assert_eq!(resolve_crate_path("super::something", ""), None);
    }

    #[test]
    fn test_own_crate_path_resolves_to_src() {
        assert_eq!(
            resolve_crate_path("mille::infrastructure::parser::rust::RustParser", "mille"),
            Some("src/infrastructure/parser/rust/RustParser".to_string())
        );
        // Without own_crate, same path is unresolvable.
        assert_eq!(
            resolve_crate_path("mille::infrastructure::parser::rust::RustParser", ""),
            None
        );
    }

    // ------------------------------------------------------------------
    // resolve (full)
    // ------------------------------------------------------------------

    #[test]
    fn test_resolve_crate_import() {
        let r = resolve(&raw("crate::domain::entity::config"));
        assert_eq!(r.category, ImportCategory::Internal);
        assert_eq!(
            r.resolved_path,
            Some("src/domain/entity/config".to_string())
        );
    }

    #[test]
    fn test_resolve_stdlib_import() {
        let r = resolve(&raw("std::fs"));
        assert_eq!(r.category, ImportCategory::Stdlib);
        assert!(r.resolved_path.is_none());
    }

    #[test]
    fn test_resolve_external_import() {
        let r = resolve(&raw("serde::Deserialize"));
        assert_eq!(r.category, ImportCategory::External);
        assert!(r.resolved_path.is_none());
    }

    #[test]
    fn test_resolve_preserves_raw() {
        let original = raw("crate::domain::entity::config");
        let r = resolve(&original);
        assert_eq!(r.raw.path, "crate::domain::entity::config");
        assert_eq!(r.raw.file, "src/any/file.rs");
    }
}
