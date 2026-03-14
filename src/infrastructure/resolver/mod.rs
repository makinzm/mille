pub mod go;
pub mod java;
pub mod python;
pub mod rust;
pub mod typescript;

use std::collections::HashMap;

use self::go::GoResolver;
use self::java::JavaResolver;
use self::python::PythonResolver;
use self::rust::RustResolver;
use self::typescript::TypeScriptResolver;
use crate::domain::entity::config::MilleConfig;
use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::ResolvedImport;
use crate::domain::repository::resolver::Resolver;

/// Dispatches to the appropriate resolver based on file extension.
pub struct DispatchingResolver {
    rust: RustResolver,
    go: GoResolver,
    python: PythonResolver,
    typescript: TypeScriptResolver,
    java: JavaResolver,
}

impl DispatchingResolver {
    pub fn new(go: GoResolver, python: PythonResolver, typescript: TypeScriptResolver) -> Self {
        DispatchingResolver {
            rust: RustResolver,
            go,
            python,
            typescript,
            java: JavaResolver::new(String::new()),
        }
    }

    /// Build a `DispatchingResolver` from a loaded `MilleConfig`.
    ///
    /// Language-specific config extraction lives here so callers only need to
    /// call this one method — adding a new language only requires changing this
    /// file.
    pub fn from_config(app_config: &MilleConfig, config_path: &str) -> Self {
        let go_module = app_config
            .resolve
            .as_ref()
            .and_then(|r| r.go.as_ref())
            .map(|g| g.module_name.clone())
            .unwrap_or_default();

        let python_packages = app_config
            .resolve
            .as_ref()
            .and_then(|r| r.python.as_ref())
            .map(|p| p.package_names.clone())
            .unwrap_or_default();

        let ts_aliases = load_ts_aliases(config_path, app_config);

        let java_config = app_config.resolve.as_ref().and_then(|r| r.java.as_ref());

        // Resolve config_dir so relative pom.xml / build.gradle paths work.
        let config_dir = std::path::Path::new(config_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));

        let java_resolver = if let Some(jcfg) = java_config {
            let manual_name = jcfg.module_name.as_deref();
            let pom_path = jcfg
                .pom_xml
                .as_deref()
                .map(|p| config_dir.join(p).to_string_lossy().into_owned());
            let gradle_path = jcfg
                .build_gradle
                .as_deref()
                .map(|p| config_dir.join(p).to_string_lossy().into_owned());
            JavaResolver::from_config(
                manual_name,
                pom_path.as_deref(),
                gradle_path.as_deref(),
                None, // settings.gradle auto-discovered relative to build.gradle
            )
        } else {
            JavaResolver::new(String::new())
        };

        DispatchingResolver {
            rust: RustResolver,
            go: GoResolver::new(go_module),
            python: PythonResolver::new(python_packages),
            typescript: TypeScriptResolver::with_aliases(ts_aliases),
            java: java_resolver,
        }
    }
}

/// Load TypeScript path aliases from the tsconfig.json referenced in mille.toml.
///
/// Reads `resolve.typescript.tsconfig`, parses `compilerOptions.paths`, and
/// returns a flat map of pattern → first target.  Returns an empty map if the
/// field is absent, the file is missing, or it has no paths entries.
fn load_ts_aliases(config_path: &str, app_config: &MilleConfig) -> HashMap<String, String> {
    let tsconfig_rel = match app_config
        .resolve
        .as_ref()
        .and_then(|r| r.typescript.as_ref())
        .map(|t| t.tsconfig.as_str())
    {
        Some(p) => p.to_string(),
        None => return HashMap::new(),
    };

    // Resolve tsconfig path relative to the directory of mille.toml.
    let config_dir = std::path::Path::new(config_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let tsconfig_path = config_dir.join(&tsconfig_rel);

    let content = match std::fs::read_to_string(&tsconfig_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };

    // NOTE: serde_json cannot parse tsconfig files that contain comments (//),
    //       but the tsconfig.json produced by tsc --init includes comments.
    //       We strip single-line comments before parsing as a best-effort.
    let stripped = strip_json_line_comments(&content);

    let value: serde_json::Value = match serde_json::from_str(&stripped) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let paths = match value
        .get("compilerOptions")
        .and_then(|c| c.get("paths"))
        .and_then(|p| p.as_object())
    {
        Some(p) => p,
        None => return HashMap::new(),
    };

    let mut aliases = HashMap::new();
    for (pattern, targets) in paths {
        if let Some(first) = targets.as_array().and_then(|a| a.first()) {
            if let Some(target) = first.as_str() {
                aliases.insert(pattern.clone(), target.to_string());
            }
        }
    }
    aliases
}

/// Strip `//` single-line comments from a JSON string (best-effort for tsconfig files).
fn strip_json_line_comments(s: &str) -> String {
    s.lines()
        .map(|line| {
            // Only strip if `//` appears outside a string value.
            // Simple heuristic: find the first `//` not inside a quoted segment.
            let mut in_string = false;
            let mut escaped = false;
            let bytes = line.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                if escaped {
                    escaped = false;
                } else if in_string {
                    if b == b'\\' {
                        escaped = true;
                    } else if b == b'"' {
                        in_string = false;
                    }
                } else if b == b'"' {
                    in_string = true;
                } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    return &line[..i];
                }
                i += 1;
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_ts_js(file: &str) -> bool {
    file.ends_with(".ts")
        || file.ends_with(".tsx")
        || file.ends_with(".js")
        || file.ends_with(".jsx")
}

impl Resolver for DispatchingResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        if import.file.ends_with(".go") {
            self.go.resolve(import)
        } else if import.file.ends_with(".py") {
            self.python.resolve(import)
        } else if is_ts_js(&import.file) {
            self.typescript.resolve(import)
        } else if import.file.ends_with(".java") || import.file.ends_with(".kt") {
            self.java.resolve(import)
        } else {
            self.rust.resolve(import)
        }
    }

    fn resolve_for_project(&self, import: &RawImport, own_crate: &str) -> ResolvedImport {
        if import.file.ends_with(".go") {
            self.go.resolve_for_project(import, own_crate)
        } else if import.file.ends_with(".py") {
            self.python.resolve_for_project(import, own_crate)
        } else if is_ts_js(&import.file) {
            self.typescript.resolve_for_project(import, own_crate)
        } else if import.file.ends_with(".java") || import.file.ends_with(".kt") {
            self.java.resolve_for_project(import, own_crate)
        } else {
            self.rust.resolve_for_project(import, own_crate)
        }
    }
}
