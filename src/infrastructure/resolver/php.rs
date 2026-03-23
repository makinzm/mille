use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// PHP built-in classes (stdlib) — no namespace prefix in PHP.
///
/// Imports that resolve to these names are classified as `Stdlib`.
static PHP_STDLIB: &[&str] = &[
    "DateTime",
    "DateTimeImmutable",
    "DateInterval",
    "DateTimeZone",
    "DateTimeInterface",
    "PDO",
    "PDOStatement",
    "PDOException",
    "Exception",
    "RuntimeException",
    "InvalidArgumentException",
    "LogicException",
    "BadMethodCallException",
    "OutOfRangeException",
    "OutOfBoundsException",
    "OverflowException",
    "UnderflowException",
    "UnexpectedValueException",
    "DomainException",
    "LengthException",
    "RangeException",
    "ErrorException",
    "BadFunctionCallException",
    "stdClass",
    "ArrayObject",
    "ArrayIterator",
    "SplStack",
    "SplQueue",
    "SplFixedArray",
    "SplPriorityQueue",
    "SplDoublyLinkedList",
    "Closure",
    "Generator",
    "Throwable",
    "Error",
    "TypeError",
    "ValueError",
    "ArithmeticError",
    "DivisionByZeroError",
    "ParseError",
    "Iterator",
    "IteratorAggregate",
    "Traversable",
    "Countable",
    "Stringable",
    "JsonSerializable",
    "Serializable",
    "ArrayAccess",
];

/// Concrete implementation of the `Resolver` port for PHP imports.
///
/// Classification rules:
/// - **Stdlib**: the root class name (first backslash-separated segment) is in `PHP_STDLIB`
/// - **Internal**: import path starts with the configured `base_namespace`
/// - **External**: everything else
///
/// `base_namespace` is configured via `[resolve.php] namespace = "App"` in `mille.toml`,
/// or auto-detected from `composer.json` `autoload.psr-4`.
pub struct PhpResolver {
    base_namespace: String,
    /// PSR-4 source directory (e.g. "src/") — used to map namespace to filesystem path.
    src_dir: String,
}

impl PhpResolver {
    pub fn new(base_namespace: String) -> Self {
        PhpResolver {
            base_namespace,
            src_dir: String::new(),
        }
    }

    /// Build a `PhpResolver` from optional config values.
    ///
    /// Priority:
    /// 1. `manual_namespace` — explicit value from `[resolve.php] namespace`
    /// 2. `composer_json_path` — auto-detect from `composer.json` `autoload.psr-4`
    /// 3. Empty string (no Internal classification possible)
    pub fn from_config(manual_namespace: Option<&str>, composer_json_path: Option<&str>) -> Self {
        let (base_namespace, src_dir) = if let Some(ns) = manual_namespace {
            let dir = composer_json_path
                .and_then(|p| read_psr4_from_composer(p, ns))
                .unwrap_or_default();
            (ns.to_string(), dir)
        } else if let Some(path) = composer_json_path {
            let (ns, dir) = read_namespace_and_dir_from_composer(path);
            (ns, dir)
        } else {
            (String::new(), String::new())
        };
        PhpResolver {
            base_namespace,
            src_dir,
        }
    }
}

impl Resolver for PhpResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        resolve_php_impl(import, &self.base_namespace, &self.src_dir)
    }

    fn resolve_for_project(&self, import: &RawImport, _own_crate: &str) -> ResolvedImport {
        resolve_php_impl(import, &self.base_namespace, &self.src_dir)
    }
}

fn resolve_php_impl(import: &RawImport, base_namespace: &str, src_dir: &str) -> ResolvedImport {
    let category = classify_php(&import.path, base_namespace);
    let resolved_path = if category == ImportCategory::Internal && !base_namespace.is_empty() {
        // Strip base namespace prefix from the import path, then prepend src_dir.
        // e.g. "App\Domain\User" with base="App" src_dir="src/"
        //   → strip "App\" → "Domain\User" → "src/Domain/User.php"
        let stripped = import
            .path
            .strip_prefix(&format!("{}\\", base_namespace))
            .unwrap_or(&import.path);
        let slash_path = stripped.replace('\\', "/");
        if src_dir.is_empty() {
            Some(format!("{}.php", slash_path))
        } else {
            let dir = src_dir.trim_end_matches('/');
            Some(format!("{}/{}.php", dir, slash_path))
        }
    } else {
        None
    };
    let package_name = if category == ImportCategory::External {
        Some(
            import
                .path
                .split('\\')
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

/// Classify a PHP import path.
///
/// - Strips leading `\` (global namespace prefix)
/// - Checks root class name against `PHP_STDLIB` → Stdlib
/// - Checks if path starts with `base_namespace` → Internal
/// - Otherwise → External
pub fn classify_php(path: &str, base_namespace: &str) -> ImportCategory {
    // Strip leading backslash (global namespace prefix like `\DateTime`)
    let path = path.trim_start_matches('\\');

    // Get the root namespace segment (first backslash-separated component)
    let root = path.split('\\').next().unwrap_or(path);

    // PHP built-in classes are classified as Stdlib
    if PHP_STDLIB.contains(&root) {
        return ImportCategory::Stdlib;
    }

    // Check against configured base namespace
    if !base_namespace.is_empty()
        && (path == base_namespace || path.starts_with(&format!("{}\\", base_namespace)))
    {
        return ImportCategory::Internal;
    }

    ImportCategory::External
}

/// Read the base namespace from a `composer.json` file path.
///
/// Looks for the first key in `autoload.psr-4` and strips the trailing `\`.
/// e.g. `"autoload": { "psr-4": { "App\\": "src/" } }` → `"App"`
pub fn read_namespace_from_composer(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    read_namespace_from_composer_content(&content)
}

/// Read both namespace and directory from a `composer.json` file path.
fn read_namespace_and_dir_from_composer(path: &str) -> (String, String) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (String::new(), String::new()),
    };
    let (ns, dir) = read_namespace_and_dir_from_content(&content);
    (ns.unwrap_or_default(), dir.unwrap_or_default())
}

/// Read the PSR-4 directory for a given namespace from `composer.json`.
fn read_psr4_from_composer(path: &str, namespace: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let psr4 = value
        .get("autoload")
        .and_then(|a| a.get("psr-4"))
        .and_then(|p| p.as_object())?;

    // Look for the namespace key (with trailing backslash)
    let key = format!("{}\\", namespace);
    psr4.get(&key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Parse base namespace from `composer.json` content string.
pub fn read_namespace_from_composer_content(content: &str) -> Option<String> {
    read_namespace_and_dir_from_content(content).0
}

/// Parse both namespace and directory from `composer.json` content string.
fn read_namespace_and_dir_from_content(content: &str) -> (Option<String>, Option<String>) {
    let value: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };
    let psr4 = match value
        .get("autoload")
        .and_then(|a| a.get("psr-4"))
        .and_then(|p| p.as_object())
    {
        Some(p) => p,
        None => return (None, None),
    };

    // Take the first PSR-4 entry
    if let Some((key, val)) = psr4.iter().next() {
        let ns = key.trim_end_matches('\\').to_string();
        let dir = val.as_str().map(|s| s.to_string());
        (Some(ns), dir)
    } else {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::{ImportKind, RawImport};

    fn raw_php(path: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: "app/Models/User.php".to_string(),
            kind: ImportKind::Import,
            named_imports: vec![],
        }
    }

    const BASE_NS: &str = "App";

    // ------------------------------------------------------------------
    // classify_php
    // ------------------------------------------------------------------

    #[test]
    fn test_php_internal_is_internal() {
        assert_eq!(
            classify_php("App\\Models\\User", BASE_NS),
            ImportCategory::Internal
        );
        assert_eq!(
            classify_php("App\\Services\\Auth", BASE_NS),
            ImportCategory::Internal
        );
        // Exact match
        assert_eq!(classify_php("App", BASE_NS), ImportCategory::Internal);
    }

    #[test]
    fn test_php_stdlib_datetime() {
        assert_eq!(classify_php("DateTime", BASE_NS), ImportCategory::Stdlib);
    }

    #[test]
    fn test_php_stdlib_pdo() {
        assert_eq!(classify_php("PDO", BASE_NS), ImportCategory::Stdlib);
    }

    #[test]
    fn test_php_stdlib_exception() {
        assert_eq!(classify_php("Exception", BASE_NS), ImportCategory::Stdlib);
        assert_eq!(
            classify_php("RuntimeException", BASE_NS),
            ImportCategory::Stdlib
        );
    }

    #[test]
    fn test_php_stdlib_leading_backslash() {
        // `\DateTime` (global namespace prefix) should still be Stdlib
        assert_eq!(classify_php("\\DateTime", BASE_NS), ImportCategory::Stdlib);
        assert_eq!(classify_php("\\PDO", BASE_NS), ImportCategory::Stdlib);
    }

    #[test]
    fn test_php_external_is_external() {
        assert_eq!(
            classify_php("Illuminate\\Http\\Request", BASE_NS),
            ImportCategory::External
        );
        assert_eq!(
            classify_php("Symfony\\Component\\HttpFoundation\\Request", BASE_NS),
            ImportCategory::External
        );
    }

    #[test]
    fn test_classify_empty_base_namespace() {
        // With no base namespace, nothing can be Internal
        assert_eq!(
            classify_php("App\\Models\\User", ""),
            ImportCategory::External
        );
        // Stdlib still works
        assert_eq!(classify_php("DateTime", ""), ImportCategory::Stdlib);
    }

    // ------------------------------------------------------------------
    // PhpResolver
    // ------------------------------------------------------------------

    #[test]
    fn test_php_resolver_internal_resolved_path() {
        let resolver = PhpResolver::new(BASE_NS.to_string());
        let import = raw_php("App\\Models\\User");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        // Without src_dir, resolved_path strips namespace prefix only
        assert_eq!(resolved.resolved_path, Some("Models/User.php".to_string()));
    }

    #[test]
    fn test_php_resolver_internal_resolved_path_with_src_dir() {
        let resolver = PhpResolver {
            base_namespace: BASE_NS.to_string(),
            src_dir: "src/".to_string(),
        };
        let import = raw_php("App\\Models\\User");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("src/Models/User.php".to_string())
        );
    }

    #[test]
    fn test_php_resolver_external_no_path() {
        let resolver = PhpResolver::new(BASE_NS.to_string());
        let import = raw_php("Illuminate\\Http\\Request");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_php_resolver_stdlib_no_path() {
        let resolver = PhpResolver::new(BASE_NS.to_string());
        let import = raw_php("DateTime");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Stdlib);
        assert!(resolved.resolved_path.is_none());
    }

    // ------------------------------------------------------------------
    // read_namespace_from_composer_content
    // ------------------------------------------------------------------

    #[test]
    fn test_read_namespace_from_composer() {
        let composer_json = r#"{
  "name": "myapp/myapp",
  "autoload": {
    "psr-4": {
      "App\\": "src/"
    }
  }
}"#;
        let result = read_namespace_from_composer_content(composer_json);
        assert_eq!(result, Some("App".to_string()));
    }

    #[test]
    fn test_read_namespace_from_composer_no_psr4() {
        let composer_json = r#"{"name": "myapp/myapp"}"#;
        let result = read_namespace_from_composer_content(composer_json);
        assert!(result.is_none());
    }
}
