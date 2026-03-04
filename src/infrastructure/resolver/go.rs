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
    todo!("GoResolver not yet implemented: import={:?}, module_name={:?}", import, module_name)
}

/// Classify a Go import path.
pub fn classify_go(path: &str, module_name: &str) -> ImportCategory {
    todo!("classify_go not yet implemented: path={:?}, module_name={:?}", path, module_name)
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
        }
    }

    const MODULE: &str = "github.com/example/myapp";

    #[test]
    fn test_go_stdlib_is_stdlib() {
        assert_eq!(classify_go("fmt", MODULE), ImportCategory::Stdlib);
        assert_eq!(classify_go("net/http", MODULE), ImportCategory::Stdlib);
        assert_eq!(classify_go("os", MODULE), ImportCategory::Stdlib);
        assert_eq!(classify_go("encoding/json", MODULE), ImportCategory::Stdlib);
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
        assert_eq!(resolved.category, ImportCategory::Stdlib);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_go_resolver_internal_resolve() {
        let resolver = GoResolver::new(MODULE.to_string());
        let import = raw_go("github.com/example/myapp/domain");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
    }

    #[test]
    fn test_go_resolver_external_resolve() {
        let resolver = GoResolver::new(MODULE.to_string());
        let import = raw_go("github.com/other/lib");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
    }

    #[test]
    fn test_go_resolver_ignores_own_crate_param() {
        // resolve_for_project uses stored module_name, not own_crate parameter
        let resolver = GoResolver::new(MODULE.to_string());
        let import = raw_go("github.com/example/myapp/domain");
        let r1 = resolver.resolve_for_project(&import, "ignored");
        let r2 = resolver.resolve(&import);
        assert_eq!(r1.category, r2.category);
    }
}
