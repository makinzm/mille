use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// Concrete implementation of the `Resolver` port for Elixir imports.
///
/// Classification rules:
/// - **Internal**: modules whose top-level namespace matches `app_name`
///   (e.g. `MyApp.Domain.User` when `app_name = "MyApp"`)
/// - **External**: everything else (e.g. `Ecto.Repo`, `Logger`, `Enum`)
///
/// Resolved path for internal imports:
/// - `MyApp.Domain.User` with `app_name="MyApp"` → `lib/domain/user.ex`
///   1. Strip the `app_name` prefix → `Domain.User`
///   2. Split on `.` → `["Domain", "User"]`
///   3. Convert each segment to snake_case (lowercase) → `["domain", "user"]`
///   4. Join with `/`, prefix `lib/`, suffix `.ex` → `lib/domain/user.ex`
pub struct ElixirResolver {
    app_name: String,
}

impl ElixirResolver {
    pub fn new(app_name: String) -> Self {
        ElixirResolver { app_name }
    }
}

impl Resolver for ElixirResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        resolve_elixir_impl(import, &self.app_name)
    }

    fn resolve_for_project(&self, import: &RawImport, _own_crate: &str) -> ResolvedImport {
        resolve_elixir_impl(import, &self.app_name)
    }
}

fn resolve_elixir_impl(import: &RawImport, app_name: &str) -> ResolvedImport {
    let category = classify_elixir(&import.path, app_name);

    let resolved_path = if category == ImportCategory::Internal {
        Some(elixir_module_to_path(&import.path, app_name))
    } else {
        None
    };

    let package_name = if category == ImportCategory::External {
        // For external modules, use the top-level namespace as package name
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

/// Classify an Elixir module path.
///
/// - Modules starting with `app_name.` (or equal to `app_name`) → Internal
/// - Everything else → External
pub fn classify_elixir(path: &str, app_name: &str) -> ImportCategory {
    if app_name.is_empty() {
        return ImportCategory::External;
    }

    // Exact match: the imported path is the app itself
    if path == app_name {
        return ImportCategory::Internal;
    }

    // Prefix match: path starts with app_name followed by a dot
    let prefix = format!("{}.", app_name);
    if path.starts_with(&prefix) {
        return ImportCategory::Internal;
    }

    ImportCategory::External
}

/// Convert an Elixir module path to a file path under `lib/`.
///
/// Steps:
/// 1. Strip the `app_name` prefix (and the following dot)
/// 2. Split on `.`
/// 3. Convert each segment to lowercase (PascalCase → snake_case for simple names)
/// 4. Join with `/`, prepend `lib/`, append `.ex`
///
/// Example: `"MyApp.Domain.User"` with `app_name="MyApp"` → `"lib/domain/user.ex"`
pub fn elixir_module_to_path(module_path: &str, app_name: &str) -> String {
    // Strip app_name prefix
    let rest = if module_path == app_name {
        ""
    } else {
        let prefix = format!("{}.", app_name);
        module_path.strip_prefix(&prefix).unwrap_or(module_path)
    };

    if rest.is_empty() {
        return format!("lib/{}.ex", app_name.to_lowercase());
    }

    let segments: Vec<String> = rest.split('.').map(|s| s.to_lowercase()).collect();
    format!("lib/{}.ex", segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};

    fn raw_ex(path: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: "lib/usecase/service.ex".to_string(),
            kind: ImportKind::Import,
            named_imports: vec![],
        }
    }

    const APP_NAME: &str = "MyApp";

    #[test]
    fn test_internal_module() {
        assert_eq!(
            classify_elixir("MyApp.Domain.User", APP_NAME),
            ImportCategory::Internal
        );
    }

    #[test]
    fn test_external_module() {
        assert_eq!(
            classify_elixir("Ecto.Repo", APP_NAME),
            ImportCategory::External
        );
    }

    #[test]
    fn test_logger_is_external() {
        assert_eq!(
            classify_elixir("Logger", APP_NAME),
            ImportCategory::External
        );
    }

    #[test]
    fn test_enum_is_external() {
        assert_eq!(
            classify_elixir("Enum", APP_NAME),
            ImportCategory::External
        );
    }

    #[test]
    fn test_app_root_is_internal() {
        assert_eq!(classify_elixir("MyApp", APP_NAME), ImportCategory::Internal);
    }

    #[test]
    fn test_internal_resolved_path() {
        let resolver = ElixirResolver::new(APP_NAME.to_string());
        let import = raw_ex("MyApp.Domain.User");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("lib/domain/user.ex".to_string())
        );
    }

    #[test]
    fn test_external_has_no_resolved_path() {
        let resolver = ElixirResolver::new(APP_NAME.to_string());
        let import = raw_ex("Ecto.Repo");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_external_package_name() {
        let resolver = ElixirResolver::new(APP_NAME.to_string());
        let import = raw_ex("Ecto.Repo");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.package_name, Some("Ecto".to_string()));
    }

    #[test]
    fn test_module_to_path_domain_user() {
        assert_eq!(
            elixir_module_to_path("MyApp.Domain.User", "MyApp"),
            "lib/domain/user.ex"
        );
    }

    #[test]
    fn test_module_to_path_usecase_service() {
        assert_eq!(
            elixir_module_to_path("MyApp.Usecase.Service", "MyApp"),
            "lib/usecase/service.ex"
        );
    }

    #[test]
    fn test_module_to_path_infrastructure_repo() {
        assert_eq!(
            elixir_module_to_path("MyApp.Infrastructure.Repo", "MyApp"),
            "lib/infrastructure/repo.ex"
        );
    }

    #[test]
    fn test_empty_app_name_is_external() {
        assert_eq!(
            classify_elixir("MyApp.Domain.User", ""),
            ImportCategory::External
        );
    }
}
