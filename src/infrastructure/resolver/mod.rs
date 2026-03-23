pub mod c;
pub mod go;
pub mod java;
pub mod php;
pub mod python;
pub mod rust;
pub mod typescript;

use std::collections::HashMap;

use self::c::CResolver;
use self::go::GoResolver;
use self::java::JavaResolver;
use self::php::PhpResolver;
use self::python::PythonResolver;
use self::rust::RustResolver;
use self::typescript::TypeScriptResolver;
use crate::domain::entity::import::RawImport;
use crate::domain::entity::resolved_import::ResolvedImport;
use crate::domain::repository::resolver::Resolver;

/// Dispatches to the appropriate resolver based on file extension.
pub struct DispatchingResolver {
    c: CResolver,
    rust: RustResolver,
    go: GoResolver,
    python: PythonResolver,
    typescript: TypeScriptResolver,
    java: JavaResolver,
    php: PhpResolver,
}

impl DispatchingResolver {
    pub fn new(go: GoResolver, python: PythonResolver, typescript: TypeScriptResolver) -> Self {
        DispatchingResolver {
            c: CResolver::new(),
            rust: RustResolver,
            go,
            python,
            typescript,
            java: JavaResolver::new(String::new()),
            php: PhpResolver::new(String::new()),
        }
    }

    /// Build a `DispatchingResolver` from a raw resolve config value and config file path.
    pub fn from_resolve_config(resolve: Option<&toml::Value>, config_path: &str) -> Self {
        let go_module = resolve
            .and_then(|r| r.get("go"))
            .and_then(|g| g.get("module_name"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let python_packages: Vec<String> = resolve
            .and_then(|r| r.get("python"))
            .and_then(|p| p.get("package_names"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let ts_aliases = load_ts_aliases(config_path, resolve);

        let java_value = resolve.and_then(|r| r.get("java"));

        let config_dir = std::path::Path::new(config_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));

        let java_resolver = if let Some(jcfg) = java_value {
            let manual_name = jcfg.get("module_name").and_then(|v| v.as_str());
            let pom_path = jcfg
                .get("pom_xml")
                .and_then(|v| v.as_str())
                .map(|p| config_dir.join(p).to_string_lossy().into_owned());
            let gradle_path = jcfg
                .get("build_gradle")
                .and_then(|v| v.as_str())
                .map(|p| config_dir.join(p).to_string_lossy().into_owned());
            JavaResolver::from_config(
                manual_name,
                pom_path.as_deref(),
                gradle_path.as_deref(),
                None,
            )
        } else {
            JavaResolver::new(String::new())
        };

        let php_resolver = if let Some(pcfg) = resolve.and_then(|r| r.get("php")) {
            let manual_ns = pcfg.get("namespace").and_then(|v| v.as_str());
            let composer_path = pcfg
                .get("composer_json")
                .and_then(|v| v.as_str())
                .map(|p| config_dir.join(p).to_string_lossy().into_owned());
            PhpResolver::from_config(manual_ns, composer_path.as_deref())
        } else {
            PhpResolver::new(String::new())
        };

        DispatchingResolver {
            c: CResolver::new(),
            rust: RustResolver,
            go: GoResolver::new(go_module),
            python: PythonResolver::new(python_packages),
            typescript: TypeScriptResolver::with_aliases(ts_aliases),
            java: java_resolver,
            php: php_resolver,
        }
    }
}

fn load_ts_aliases(config_path: &str, resolve: Option<&toml::Value>) -> HashMap<String, String> {
    let tsconfig_rel = match resolve
        .and_then(|r| r.get("typescript"))
        .and_then(|t| t.get("tsconfig"))
        .and_then(|v| v.as_str())
    {
        Some(p) => p.to_string(),
        None => return HashMap::new(),
    };

    let config_dir = std::path::Path::new(config_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let tsconfig_path = config_dir.join(&tsconfig_rel);

    let content = match std::fs::read_to_string(&tsconfig_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };

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

fn strip_json_line_comments(s: &str) -> String {
    s.lines()
        .map(|line| {
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

fn is_c(file: &str) -> bool {
    file.ends_with(".c") || file.ends_with(".h")
}

fn is_ts_js(file: &str) -> bool {
    file.ends_with(".ts")
        || file.ends_with(".tsx")
        || file.ends_with(".js")
        || file.ends_with(".jsx")
}

impl Resolver for DispatchingResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        if is_c(&import.file) {
            self.c.resolve(import)
        } else if import.file.ends_with(".go") {
            self.go.resolve(import)
        } else if import.file.ends_with(".py") {
            self.python.resolve(import)
        } else if is_ts_js(&import.file) {
            self.typescript.resolve(import)
        } else if import.file.ends_with(".java") || import.file.ends_with(".kt") {
            self.java.resolve(import)
        } else if import.file.ends_with(".php") {
            self.php.resolve(import)
        } else {
            self.rust.resolve(import)
        }
    }

    fn resolve_for_project(&self, import: &RawImport, own_crate: &str) -> ResolvedImport {
        if is_c(&import.file) {
            self.c.resolve_for_project(import, own_crate)
        } else if import.file.ends_with(".go") {
            self.go.resolve_for_project(import, own_crate)
        } else if import.file.ends_with(".py") {
            self.python.resolve_for_project(import, own_crate)
        } else if is_ts_js(&import.file) {
            self.typescript.resolve_for_project(import, own_crate)
        } else if import.file.ends_with(".java") || import.file.ends_with(".kt") {
            self.java.resolve_for_project(import, own_crate)
        } else if import.file.ends_with(".php") {
            self.php.resolve_for_project(import, own_crate)
        } else {
            self.rust.resolve_for_project(import, own_crate)
        }
    }
}
