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

    /// Build a `JavaResolver` from optional build file paths.
    ///
    /// Priority:
    /// 1. `manual_module_name` — explicit value from `[resolve.java] module_name`
    /// 2. `pom_xml` — auto-detect from Maven pom.xml (groupId.artifactId)
    /// 3. `build_gradle` + `settings_gradle` — auto-detect from Gradle files
    /// 4. Empty string (no classification possible)
    pub fn from_config(
        manual_module_name: Option<&str>,
        pom_xml_path: Option<&str>,
        build_gradle_path: Option<&str>,
        settings_gradle_path: Option<&str>,
    ) -> Self {
        let module_name = if let Some(name) = manual_module_name {
            name.to_string()
        } else if let Some(pom_path) = pom_xml_path {
            read_module_from_pom(pom_path).unwrap_or_default()
        } else if let Some(gradle_path) = build_gradle_path {
            read_module_from_gradle(gradle_path, settings_gradle_path).unwrap_or_default()
        } else {
            String::new()
        };
        JavaResolver { module_name }
    }
}

/// Parse `groupId` and `artifactId` from a pom.xml file path.
/// Returns `"groupId.artifactId"` or `None` if not found.
pub fn read_module_from_pom(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    read_module_from_pom_content(&content)
}

/// Parse module name from pom.xml content string.
///
/// Extracts the first `<groupId>` and first `<artifactId>` tags from the
/// content and returns `"groupId.artifactId"`.
pub fn read_module_from_pom_content(content: &str) -> Option<String> {
    let group_id = extract_xml_tag(content, "groupId")?;
    let artifact_id = extract_xml_tag(content, "artifactId")?;
    Some(format!("{}.{}", group_id, artifact_id))
}

/// Extract the text content of the first occurrence of `<tag>text</tag>` in `s`.
fn extract_xml_tag<'a>(s: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = s.find(&open)? + open.len();
    let end = s[start..].find(&close)? + start;
    Some(s[start..end].trim())
}

/// Parse `group` from build.gradle and `rootProject.name` from settings.gradle.
/// Returns `"group.name"` or `None` if not found.
pub fn read_module_from_gradle(
    build_gradle_path: &str,
    settings_gradle_path: Option<&str>,
) -> Option<String> {
    let build_content = std::fs::read_to_string(build_gradle_path).ok()?;
    let settings_path = settings_gradle_path.unwrap_or("settings.gradle");
    // Try to find settings.gradle in the same directory as build.gradle
    let settings_content = {
        let dir = std::path::Path::new(build_gradle_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let candidate = dir.join(settings_path);
        std::fs::read_to_string(&candidate)
            .or_else(|_| std::fs::read_to_string(settings_path))
            .unwrap_or_default()
    };
    read_module_from_gradle_content(&build_content, &settings_content)
}

/// Parse module name from build.gradle and settings.gradle content strings.
///
/// Extracts `group` from build.gradle (e.g. `group = 'com.example'`) and
/// `rootProject.name` from settings.gradle (e.g. `rootProject.name = 'myapp'`),
/// then returns `"group.name"`.
pub fn read_module_from_gradle_content(
    build_gradle: &str,
    settings_gradle: &str,
) -> Option<String> {
    let group = extract_gradle_value(build_gradle, "group")?;
    let name = extract_gradle_value(settings_gradle, "rootProject.name")?;
    Some(format!("{}.{}", group, name))
}

/// Extract a value assigned via `key = 'value'` or `key = "value"` from a Gradle file.
fn extract_gradle_value<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    for line in content.lines() {
        let trimmed = line.trim();
        // Match `key = 'value'` or `key = "value"` or `key='value'` etc.
        if let Some(rest) = trimmed.strip_prefix(key) {
            let rest = rest.trim();
            if let Some(rest) = rest.strip_prefix('=') {
                let rest = rest.trim();
                // Strip surrounding quotes
                let value = if (rest.starts_with('\'') && rest.ends_with('\''))
                    || (rest.starts_with('"') && rest.ends_with('"'))
                {
                    &rest[1..rest.len() - 1]
                } else {
                    rest
                };
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    None
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
    // For internal Java imports, compute a resolved path using slashes so that
    // the ViolationDetector can match it against layer glob patterns like
    // "**/domain/**".
    //
    // e.g. "com.example.myapp.domain.User" → "com/example/myapp/domain/User.java"
    // This enables both "src/domain/**" (if the project uses src/ prefix) and
    // "**/domain/**" to match correctly.
    let resolved_path = if category == ImportCategory::Internal && !module_name.is_empty() {
        let slash_path = import.path.replace('.', "/");
        Some(format!("{}.java", slash_path))
    } else {
        None
    };
    // Java uses full dotted path as external_allow key (e.g. "java.util.List")
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
        // resolved_path uses slash-separated path so globs like "**/domain/**" work.
        // e.g. "com.example.myapp.domain.User" -> "com/example/myapp/domain/User.java"
        assert_eq!(
            resolved.resolved_path,
            Some("com/example/myapp/domain/User.java".to_string())
        );
    }

    #[test]
    fn test_java_resolver_path_uses_slashes() {
        // Dots in the import path must become slashes so that globs like
        // "**/domain/**" can match the resolved path.
        let resolver = JavaResolver::new(MODULE.to_string());
        let import = raw_java("com.example.myapp.usecase.UserService");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert_eq!(
            resolved.resolved_path,
            Some("com/example/myapp/usecase/UserService.java".to_string())
        );
    }

    #[test]
    fn test_read_module_from_pom_xml() {
        let pom_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
  <groupId>com.example</groupId>
  <artifactId>myapp</artifactId>
  <version>1.0.0</version>
</project>"#;
        let result = read_module_from_pom_content(pom_xml);
        assert_eq!(result, Some("com.example.myapp".to_string()));
    }

    #[test]
    fn test_read_module_from_pom_xml_missing_group_id() {
        let pom_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
  <artifactId>myapp</artifactId>
</project>"#;
        let result = read_module_from_pom_content(pom_xml);
        assert_eq!(result, None);
    }

    #[test]
    fn test_read_module_from_gradle_content() {
        let build_gradle = "group = 'com.example'\nversion = '1.0.0'";
        let settings_gradle = "rootProject.name = 'myapp'";
        let result = read_module_from_gradle_content(build_gradle, settings_gradle);
        assert_eq!(result, Some("com.example.myapp".to_string()));
    }

    #[test]
    fn test_read_module_from_gradle_content_double_quotes() {
        let build_gradle = r#"group = "com.example"
version = "1.0.0""#;
        let settings_gradle = r#"rootProject.name = "myapp""#;
        let result = read_module_from_gradle_content(build_gradle, settings_gradle);
        assert_eq!(result, Some("com.example.myapp".to_string()));
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
