use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// Concrete implementation of the `Resolver` port for Java imports.
///
/// Classification rules:
/// - internal: path starts with the module name from `[resolve.java]`
/// - external: everything else (including java.* stdlib packages)
pub struct JavaResolver {
    module_name: String,
}

impl JavaResolver {
    pub fn new(module_name: String) -> Self {
        JavaResolver { module_name }
    }
}

impl Resolver for JavaResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        resolve_java_impl(import, &self.module_name)
    }

    /// For Java, `own_crate` is ignored — the stored `module_name` is used instead.
    fn resolve_for_project(&self, import: &RawImport, _own_crate: &str) -> ResolvedImport {
        resolve_java_impl(import, &self.module_name)
    }
}

fn resolve_java_impl(import: &RawImport, module_name: &str) -> ResolvedImport {
    let category = classify_java(&import.path, module_name);
    // For internal Java imports, compute a synthetic resolved path so the
    // ViolationDetector can match it against layer glob patterns.
    // e.g. "com.example.myapp.domain.User" (module="com.example.myapp")
    //   → relative = "domain.User" → resolved_path = "src/domain/_.java"
    //
    // NOTE: Java packages use dots as separators. We take only the first component
    // of the relative path (the sub-package name) to construct a directory-like path
    // that matches typical layer glob patterns such as "src/domain/**".
    let resolved_path = if category == ImportCategory::Internal && !module_name.is_empty() {
        let prefix = format!("{}.", module_name);
        import.path.strip_prefix(&prefix).map(|rel| {
            // rel = "domain.User" → first segment = "domain"
            let first_segment = rel.split('.').next().unwrap_or(rel);
            format!("src/{}/_.java", first_segment)
        })
    } else {
        None
    };
    ResolvedImport {
        raw: import.clone(),
        category,
        resolved_path,
    }
}

/// Classify a Java import path.
///
/// - internal: path starts with `module_name`
/// - external: everything else (java.util.*, third-party, etc.)
pub fn classify_java(path: &str, module_name: &str) -> ImportCategory {
    if !module_name.is_empty()
        && (path == module_name || path.starts_with(&format!("{}.", module_name)))
    {
        return ImportCategory::Internal;
    }

    ImportCategory::External
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};

    fn raw_java(path: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: "some/Foo.java".to_string(),
            kind: ImportKind::Import,
            named_imports: vec![],
        }
    }

    const MODULE: &str = "com.example.myapp";

    #[test]
    fn test_java_internal_is_internal() {
        assert_eq!(
            classify_java("com.example.myapp.domain.User", MODULE),
            ImportCategory::Internal
        );
        assert_eq!(
            classify_java("com.example.myapp.usecase.UserService", MODULE),
            ImportCategory::Internal
        );
    }

    #[test]
    fn test_java_external_is_external() {
        assert_eq!(
            classify_java(
                "org.springframework.web.bind.annotation.RestController",
                MODULE
            ),
            ImportCategory::External
        );
        assert_eq!(
            classify_java("com.fasterxml.jackson.databind.ObjectMapper", MODULE),
            ImportCategory::External
        );
    }

    #[test]
    fn test_java_stdlib_is_external() {
        assert_eq!(
            classify_java("java.util.List", MODULE),
            ImportCategory::External
        );
        assert_eq!(
            classify_java("java.io.InputStream", MODULE),
            ImportCategory::External
        );
        assert_eq!(
            classify_java("javax.persistence.Entity", MODULE),
            ImportCategory::External
        );
    }

    #[test]
    fn test_java_resolver_internal_resolve() {
        let resolver = JavaResolver::new(MODULE.to_string());
        let import = raw_java("com.example.myapp.domain.User");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        // resolved_path should enable ViolationDetector to match "src/domain/**"
        assert_eq!(
            resolved.resolved_path,
            Some("src/domain/_.java".to_string())
        );
    }

    #[test]
    fn test_java_resolver_external_resolve() {
        let resolver = JavaResolver::new(MODULE.to_string());
        let import = raw_java("java.util.List");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_java_resolver_ignores_own_crate_param() {
        let resolver = JavaResolver::new(MODULE.to_string());
        let import = raw_java("com.example.myapp.domain.User");
        let r1 = resolver.resolve_for_project(&import, "ignored");
        let r2 = resolver.resolve(&import);
        assert_eq!(r1.category, r2.category);
        assert_eq!(r1.resolved_path, r2.resolved_path);
    }
}
