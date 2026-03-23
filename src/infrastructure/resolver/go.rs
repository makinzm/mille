use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// Concrete implementation of the `Resolver` port for Go imports.
///
/// Classification rules:
/// - stdlib: no dot in the first path segment (e.g. `"fmt"`, `"net/http"`)
/// - internal: path starts with the module name from `[resolve.go]`
/// - external: everything else
pub struct GoResolver {
    module_name: String,
}

impl GoResolver {
    pub fn new(module_name: String) -> Self {
        GoResolver { module_name }
    }
}

impl Resolver for GoResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        resolve_go_impl(import, &self.module_name)
    }

    /// For Go, `own_crate` is ignored — the stored `module_name` is used instead.
    fn resolve_for_project(&self, import: &RawImport, _own_crate: &str) -> ResolvedImport {
        resolve_go_impl(import, &self.module_name)
    }
}

fn resolve_go_impl(import: &RawImport, module_name: &str) -> ResolvedImport {
    let category = classify_go(&import.path, module_name);
    // For internal Go imports, compute a synthetic resolved path so the
    // ViolationDetector can match it against layer glob patterns.
    // e.g. "github.com/example/gosample/domain" (module="github.com/example/gosample")
    //   → resolved_path = "domain/_.go"  (matches layer glob "domain/**")
    let resolved_path = if category == ImportCategory::Internal && !module_name.is_empty() {
        let prefix = format!("{}/", module_name);
        import
            .path
            .strip_prefix(&prefix)
            .map(|rel| format!("{}/_.go", rel))
    } else {
        None
    };
    let package_name = if category == ImportCategory::External {
        Some(import.path.clone())
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

/// Classify a Go import path.
///
/// - internal: path starts with `module_name`
/// - external: everything else, including Go stdlib packages like `fmt`, `net/http`,
///             `database/sql`. Go architecture rules apply to all non-internal imports,
///             so stdlib packages are subject to `external_allow` / `external_deny` checks.
pub fn classify_go(path: &str, module_name: &str) -> ImportCategory {
    // internal: starts with the project's module name
    if !module_name.is_empty()
        && (path == module_name || path.starts_with(&format!("{}/", module_name)))
    {
        return ImportCategory::Internal;
    }

    ImportCategory::External
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};

    fn raw_go(path: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: "some/file.go".to_string(),
            kind: ImportKind::Import,
            named_imports: vec![],
        }
    }

    const MODULE: &str = "github.com/example/myapp";

    #[test]
    fn test_go_stdlib_is_external() {
        // Go stdlib packages (no dot in first segment) are treated as External
        // so that external_allow / external_deny rules can control them.
        assert_eq!(classify_go("fmt", MODULE), ImportCategory::External);
        assert_eq!(classify_go("net/http", MODULE), ImportCategory::External);
        assert_eq!(classify_go("os", MODULE), ImportCategory::External);
        assert_eq!(
            classify_go("encoding/json", MODULE),
            ImportCategory::External
        );
        assert_eq!(
            classify_go("database/sql", MODULE),
            ImportCategory::External
        );
    }

    #[test]
    fn test_go_internal_is_internal() {
        assert_eq!(
            classify_go("github.com/example/myapp/domain", MODULE),
            ImportCategory::Internal
        );
        assert_eq!(
            classify_go("github.com/example/myapp/usecase", MODULE),
            ImportCategory::Internal
        );
        assert_eq!(
            classify_go("github.com/example/myapp/infrastructure/db", MODULE),
            ImportCategory::Internal
        );
    }

    #[test]
    fn test_go_external_is_external() {
        assert_eq!(
            classify_go("github.com/other/lib", MODULE),
            ImportCategory::External
        );
        assert_eq!(
            classify_go("google.golang.org/grpc", MODULE),
            ImportCategory::External
        );
    }

    #[test]
    fn test_go_resolver_resolve() {
        let resolver = GoResolver::new(MODULE.to_string());
        let import = raw_go("fmt");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_go_resolver_internal_resolve() {
        let resolver = GoResolver::new(MODULE.to_string());
        let import = raw_go("github.com/example/myapp/domain");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        // resolved_path should enable ViolationDetector to match "domain/**"
        assert_eq!(resolved.resolved_path, Some("domain/_.go".to_string()));
    }

    #[test]
    fn test_go_resolver_external_resolve() {
        let resolver = GoResolver::new(MODULE.to_string());
        let import = raw_go("github.com/other/lib");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_go_resolver_ignores_own_crate_param() {
        // resolve_for_project uses stored module_name, not own_crate parameter
        let resolver = GoResolver::new(MODULE.to_string());
        let import = raw_go("github.com/example/myapp/domain");
        let r1 = resolver.resolve_for_project(&import, "ignored");
        let r2 = resolver.resolve(&import);
        assert_eq!(r1.category, r2.category);
        assert_eq!(r1.resolved_path, r2.resolved_path);
    }
}
