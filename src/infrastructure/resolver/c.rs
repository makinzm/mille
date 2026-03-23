use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::resolved_import::{ImportCategory, ResolvedImport};
use crate::domain::repository::resolver::Resolver;

/// C standard library headers.
///
/// Includes from `<header.h>` that match these are classified as `Stdlib`.
/// Headers not in this list but using `<...>` syntax are classified as `External`.
static C_STDLIB_HEADERS: &[&str] = &[
    "assert.h",
    "complex.h",
    "ctype.h",
    "errno.h",
    "fenv.h",
    "float.h",
    "inttypes.h",
    "iso646.h",
    "limits.h",
    "locale.h",
    "math.h",
    "setjmp.h",
    "signal.h",
    "stdalign.h",
    "stdarg.h",
    "stdatomic.h",
    "stdbool.h",
    "stddef.h",
    "stdint.h",
    "stdio.h",
    "stdlib.h",
    "stdnoreturn.h",
    "string.h",
    "tgmath.h",
    "threads.h",
    "time.h",
    "uchar.h",
    "wchar.h",
    "wctype.h",
    // POSIX commonly used
    "unistd.h",
    "fcntl.h",
    "sys/types.h",
    "sys/stat.h",
    "sys/socket.h",
    "netinet/in.h",
    "arpa/inet.h",
    "pthread.h",
    "dirent.h",
    "dlfcn.h",
];

/// Resolver for C source files.
///
/// Classification rules:
/// - `#include "..."` (ImportKind::Use) → `Internal`
/// - `#include <...>` (ImportKind::Import) → `Stdlib` if header is in C_STDLIB_HEADERS, else `External`
pub struct CResolver;

impl CResolver {
    pub fn new() -> Self {
        CResolver
    }
}

impl Default for CResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver for CResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        let category = match import.kind {
            // Local includes are internal project headers
            ImportKind::Use => ImportCategory::Internal,
            // System includes: check against stdlib list
            ImportKind::Import => {
                if C_STDLIB_HEADERS.contains(&import.path.as_str()) {
                    ImportCategory::Stdlib
                } else {
                    ImportCategory::External
                }
            }
            // Mod is not used for C, but handle gracefully
            ImportKind::Mod => ImportCategory::Unknown,
        };

        let resolved_path = if category == ImportCategory::Internal {
            // For local includes, resolve relative to the source file
            let source_dir = std::path::Path::new(&import.file)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let joined = source_dir.join(&import.path);
            // Normalize path to remove ".." components without filesystem access
            let normalized = normalize_path(&joined);
            Some(
                normalized
                    .to_string_lossy()
                    .replace('\\', "/")
                    .trim_end_matches(".h")
                    .trim_end_matches(".c")
                    .to_string(),
            )
        } else {
            None
        };

        let package_name = if category == ImportCategory::External {
            Some(
                import
                    .path
                    .split('/')
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
}

/// Normalize a path by resolving `.` and `..` components without filesystem access.
fn normalize_path(path: &std::path::Path) -> std::path::PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            c => components.push(c),
        }
    }
    components.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::ImportKind;

    fn make_import(path: &str, kind: ImportKind, file: &str) -> RawImport {
        RawImport {
            path: path.to_string(),
            line: 1,
            file: file.to_string(),
            kind,
            named_imports: vec![],
        }
    }

    #[test]
    fn test_local_include_is_internal() {
        let resolver = CResolver::new();
        let import = make_import(
            "domain/user.h",
            ImportKind::Use,
            "src/usecase/create_user.c",
        );
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Internal);
        assert!(resolved.resolved_path.is_some());
    }

    #[test]
    fn test_system_stdio_is_stdlib() {
        let resolver = CResolver::new();
        let import = make_import("stdio.h", ImportKind::Import, "src/main.c");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Stdlib);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_system_stdlib_is_stdlib() {
        let resolver = CResolver::new();
        let import = make_import("stdlib.h", ImportKind::Import, "src/main.c");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Stdlib);
    }

    #[test]
    fn test_external_lib_is_external() {
        let resolver = CResolver::new();
        let import = make_import("curl/curl.h", ImportKind::Import, "src/infra/http.c");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::External);
        assert!(resolved.resolved_path.is_none());
    }

    #[test]
    fn test_posix_unistd_is_stdlib() {
        let resolver = CResolver::new();
        let import = make_import("unistd.h", ImportKind::Import, "src/main.c");
        let resolved = resolver.resolve(&import);
        assert_eq!(resolved.category, ImportCategory::Stdlib);
    }
}
