use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};

/// Resolve a Rust `RawImport` into a categorised `ResolvedImport`.
pub fn resolve(import: &RawImport) -> ResolvedImport {
    todo!()
}

/// Classify the import category from its path string.
fn classify(path: &str) -> ImportCategory {
    todo!()
}

/// Normalise a `crate::` path to a file-system path relative to the repo root.
/// Returns `None` for wildcards, grouped imports, or unresolvable paths.
fn resolve_crate_path(path: &str) -> Option<String> {
    todo!()
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
        }
    }

    // ------------------------------------------------------------------
    // classify
    // ------------------------------------------------------------------

    #[test]
    fn test_std_is_stdlib() {
        assert_eq!(classify("std::fs"), ImportCategory::Stdlib);
        assert_eq!(classify("std::io::Write"), ImportCategory::Stdlib);
    }

    #[test]
    fn test_core_is_stdlib() {
        assert_eq!(classify("core::fmt"), ImportCategory::Stdlib);
    }

    #[test]
    fn test_alloc_is_stdlib() {
        assert_eq!(classify("alloc::vec::Vec"), ImportCategory::Stdlib);
    }

    #[test]
    fn test_crate_is_internal() {
        assert_eq!(classify("crate::domain::entity::config"), ImportCategory::Internal);
    }

    #[test]
    fn test_super_is_internal() {
        assert_eq!(classify("super::something"), ImportCategory::Internal);
    }

    #[test]
    fn test_self_is_internal() {
        assert_eq!(classify("self::helper"), ImportCategory::Internal);
    }

    #[test]
    fn test_external_crate_is_external() {
        assert_eq!(classify("serde::Deserialize"), ImportCategory::External);
        assert_eq!(classify("toml::from_str"), ImportCategory::External);
    }

    // ------------------------------------------------------------------
    // resolve_crate_path
    // ------------------------------------------------------------------

    #[test]
    fn test_simple_crate_path() {
        assert_eq!(
            resolve_crate_path("crate::domain::entity::config"),
            Some("src/domain/entity/config".to_string())
        );
    }

    #[test]
    fn test_crate_path_single_segment() {
        assert_eq!(
            resolve_crate_path("crate::main"),
            Some("src/main".to_string())
        );
    }

    #[test]
    fn test_grouped_import_returns_none() {
        assert_eq!(
            resolve_crate_path("crate::domain::{entity, repository}"),
            None
        );
    }

    #[test]
    fn test_wildcard_import_returns_none() {
        assert_eq!(resolve_crate_path("crate::domain::*"), None);
    }

    #[test]
    fn test_non_crate_path_returns_none() {
        assert_eq!(resolve_crate_path("serde::Deserialize"), None);
        assert_eq!(resolve_crate_path("super::something"), None);
    }

    // ------------------------------------------------------------------
    // resolve (full)
    // ------------------------------------------------------------------

    #[test]
    fn test_resolve_crate_import() {
        let r = resolve(&raw("crate::domain::entity::config"));
        assert_eq!(r.category, ImportCategory::Internal);
        assert_eq!(r.resolved_path, Some("src/domain/entity/config".to_string()));
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
