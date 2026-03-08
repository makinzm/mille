use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// Concrete implementation of the `Resolver` port for Python imports.
///
/// Classification rules:
/// - **Internal**: relative imports (starting with `.`) or imports whose top-level
///   package name is listed in `package_names`
/// - **External**: everything else
///
/// `package_names` is configured via `[resolve.python] package_names = ["myapp", "domain", …]`
/// in `mille.toml`.
pub struct PythonResolver {
    package_names: Vec<String>,
}

impl PythonResolver {
    pub fn new(package_names: Vec<String>) -> Self {
        PythonResolver { package_names }
    }
}

impl Resolver for PythonResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        resolve_python_impl(import, &self.package_names)
    }

    fn resolve_for_project(&self, import: &RawImport, _own_crate: &str) -> ResolvedImport {
        resolve_python_impl(import, &self.package_names)
    }
}

fn resolve_python_impl(import: &RawImport, package_names: &[String]) -> ResolvedImport {
    let category = classify_python(&import.path, package_names);
    let resolved_path = if category == ImportCategory::Internal {
        // Compute a synthetic resolved path for the ViolationDetector to match against
        // layer glob patterns.
        // "domain.entity" → "domain/entity/_.py"  (matches "domain/**")
        // ".entity"       → strip leading dot, use file directory
        // "."             → current package (use file directory)
        let clean = import.path.trim_start_matches('.');
        if clean.is_empty() {
            // bare relative import "from . import X" — path is ambiguous without context
            None
        } else {
            Some(format!("{}/_.py", clean.replace('.', "/")))
        }
    } else {
        None
    };

    ResolvedImport {
        raw: import.clone(),
        category,
        resolved_path,
    }
}

/// Classify a Python import path.
///
/// - Relative imports (`.` or `.something`) → Internal
/// - Imports whose first dotted segment matches a configured `package_name` → Internal
/// - Everything else → External
pub fn classify_python(path: &str, package_names: &[String]) -> ImportCategory {
    // Relative imports are always internal
    if path.starts_with('.') {
        return ImportCategory::Internal;
    }

    // Absolute imports matching a configured package name are internal
    let top_level = path.split('.').next().unwrap_or(path);
    if package_names.iter().any(|p| p == top_level) {
        return ImportCategory::Internal;
    }

    ImportCategory::External
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};

    fn raw_py(path: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: "domain/entity.py".to_string(),
            kind: ImportKind::Import,
            named_imports: vec![],
        }
    }

    const PACKAGES: &[&str] = &["domain", "usecase", "infrastructure"];

    fn packages() -> Vec<String> {
        PACKAGES.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_relative_import_is_internal() {
        assert_eq!(classify_python(".", &packages()), ImportCategory::Internal);
        assert_eq!(
            classify_python(".entity", &packages()),
            ImportCategory::Internal
        );
        assert_eq!(
            classify_python("..domain", &packages()),
            ImportCategory::Internal
        );
    }

    #[test]
    fn test_package_name_import_is_internal() {
        assert_eq!(
            classify_python("domain.entity", &packages()),
            ImportCategory::Internal
        );
        assert_eq!(
            classify_python("usecase.service", &packages()),
            ImportCategory::Internal
        );
        assert_eq!(
            classify_python("infrastructure.db", &packages()),
            ImportCategory::Internal
        );
    }

    #[test]
    fn test_external_import_is_external() {
        assert_eq!(classify_python("os", &packages()), ImportCategory::External);
        assert_eq!(
            classify_python("sqlalchemy", &packages()),
            ImportCategory::External
        );
        assert_eq!(
            classify_python("numpy.array", &packages()),
            ImportCategory::External
        );
    }

    #[test]
    fn test_empty_package_names_only_relative_is_internal() {
        assert_eq!(classify_python(".", &[]), ImportCategory::Internal);
        assert_eq!(
            classify_python("domain.entity", &[]),
            ImportCategory::External
        );
    }

    #[test]
    fn test_resolver_internal_has_resolved_path() {
        let resolver = PythonResolver::new(packages());
        let import = raw_py("domain.entity");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("domain/entity/_.py".to_string())
        );
    }

    #[test]
    fn test_resolver_monorepo_absolute_import_uses_src_root() {
        // crawler/src/infrastructure/ にあるファイルが domain.entity をインポート
        // → resolved path は crawler/src/domain/entity/_.py (= crawler/src/domain/** にマッチ)
        let resolver = PythonResolver::new(packages());
        let import = RawImport {
            path: "domain.entity".to_string(),
            line: 1,
            file: "crawler/src/infrastructure/file_storage.py".to_string(),
            kind: ImportKind::Import,
            named_imports: vec![],
        };
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("crawler/src/domain/entity/_.py".to_string()),
            "モノレポでは src_root を使った resolved path が必要"
        );
    }

    #[test]
    fn test_resolver_root_level_file_no_src_root_unchanged() {
        // domain/ 直下のファイルはこれまで通り (regression)
        let resolver = PythonResolver::new(packages());
        let import = raw_py("domain.entity"); // file = "domain/entity.py"
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("domain/entity/_.py".to_string()),
            "ルートレベルファイルは従来通りの resolved path"
        );
    }

    #[test]
    fn test_resolver_src_layout_absolute_import() {
        // src/infrastructure/ にあるファイルが domain.usecase をインポート
        // → resolved path は src/domain/usecase/_.py
        let resolver = PythonResolver::new(packages());
        let import = RawImport {
            path: "domain.usecase".to_string(),
            line: 1,
            file: "src/infrastructure/db.py".to_string(),
            kind: ImportKind::Import,
            named_imports: vec![],
        };
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("src/domain/usecase/_.py".to_string()),
        );
    }

    #[test]
    fn test_resolver_external_has_no_resolved_path() {
        let resolver = PythonResolver::new(packages());
        let import = raw_py("sqlalchemy");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
        assert!(resolved.resolved_path.is_none());
    }
}
