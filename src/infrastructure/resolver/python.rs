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
        let clean = import.path.trim_start_matches('.');
        if clean.is_empty() {
            // bare relative import "from . import X" — path is ambiguous without context
            None
        } else if import.path.starts_with('.') {
            // Relative import: resolve relative to the importing file's directory.
            // ".entity" in "crawler/src/domain/repository.py"
            //   → "crawler/src/domain/entity/_.py"
            let file_dir = import.file.rsplit_once('/').map(|x| x.0).unwrap_or("");
            Some(format!("{}/{}/_.py", file_dir, clean.replace('.', "/")))
        } else {
            // Absolute internal import: derive src_root from the importing file's path
            // so the resolved path matches layer glob patterns in monorepos.
            // "domain.entity" in "crawler/src/infrastructure/file_storage.py"
            //   (package_names includes "infrastructure")
            //   → src_root = "crawler/src"
            //   → "crawler/src/domain/entity/_.py"
            let src_root = derive_src_root(&import.file, package_names);
            let prefix = if src_root.is_empty() {
                String::new()
            } else {
                format!("{}/", src_root)
            };
            Some(format!("{}{}/_.py", prefix, clean.replace('.', "/")))
        }
    } else {
        None
    };

    let package_name = if category == ImportCategory::External {
        Some(
            import
                .path
                .split('.')
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

/// Derive the Python source root from the importing file's path.
///
/// Scans path components (excluding the filename) from left to right.
/// Returns everything before the first component that matches a `package_name`.
///
/// Examples:
/// - `"crawler/src/infrastructure/file.py"`, packages = `["infrastructure", "domain"]`
///   → `"crawler/src"` (the `infrastructure` component is at index 2)
/// - `"domain/entity.py"`, packages = `["domain"]`
///   → `""` (the `domain` component is at index 0 — src_root is the project root)
/// - `"src/domain/entity.py"`, packages = `["domain"]`
///   → `"src"`
fn derive_src_root<'a>(file: &'a str, package_names: &[String]) -> &'a str {
    let components: Vec<&str> = file.split('/').collect();
    // Skip the last component (filename).
    let dir_components = components.len().saturating_sub(1);
    for i in 0..dir_components {
        if package_names.iter().any(|p| p.as_str() == components[i]) {
            // src_root = file[..position_before_component_i]
            // Compute byte offset: sum of lengths + separators for components[0..i]
            if i == 0 {
                return &file[..0]; // empty string slice at start
            }
            let prefix_len: usize =
                components[..i].iter().map(|s| s.len()).sum::<usize>() + (i - 1); // separators between components[0..i-1]
            return &file[..prefix_len];
        }
    }
    // No package component found — use the file's directory as a fallback.
    // This keeps the old behavior for files outside any package.
    &file[..0]
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
