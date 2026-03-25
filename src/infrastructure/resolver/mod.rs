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
///
/// Only resolvers for languages listed in `[project] languages` are registered.
/// Unknown extensions fall back to the Rust resolver (if registered) for backwards
/// compatibility with existing single-language Rust projects.
pub struct DispatchingResolver {
    /// Maps file extensions (e.g. ".rs", ".ts") to their resolver.
    resolvers: HashMap<&'static str, Box<dyn Resolver>>,
}

impl DispatchingResolver {
    /// Build a `DispatchingResolver` from a raw resolve config value, config file path,
    /// and the list of languages declared in `[project] languages`.
    pub fn from_resolve_config(
        resolve: Option<&toml::Value>,
        config_path: &str,
        languages: &[String],
    ) -> Self {
        let config_dir = std::path::Path::new(config_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));

        let mut resolvers: HashMap<&'static str, Box<dyn Resolver>> = HashMap::new();

        for lang in languages {
            match lang.as_str() {
                "rust" => {
                    resolvers.insert(".rs", Box::new(RustResolver));
                }
                "go" => {
                    let go_module = resolve
                        .and_then(|r| r.get("go"))
                        .and_then(|g| g.get("module_name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    resolvers.insert(".go", Box::new(GoResolver::new(go_module)));
                }
                "python" => {
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
                    resolvers.insert(".py", Box::new(PythonResolver::new(python_packages)));
                }
                "typescript" | "javascript" => {
                    let ts_aliases = load_ts_aliases(config_path, resolve);
                    let ts_resolver = TypeScriptResolver::with_aliases(ts_aliases);
                    resolvers.insert(".ts", Box::new(ts_resolver));
                    // Share the same resolver config for all TS/JS extensions.
                    // We need separate instances since Box doesn't impl Clone,
                    // but the aliases are the same.
                    let ts_aliases2 = load_ts_aliases(config_path, resolve);
                    resolvers.insert(
                        ".tsx",
                        Box::new(TypeScriptResolver::with_aliases(ts_aliases2)),
                    );
                    let ts_aliases3 = load_ts_aliases(config_path, resolve);
                    resolvers.insert(
                        ".js",
                        Box::new(TypeScriptResolver::with_aliases(ts_aliases3)),
                    );
                    let ts_aliases4 = load_ts_aliases(config_path, resolve);
                    resolvers.insert(
                        ".jsx",
                        Box::new(TypeScriptResolver::with_aliases(ts_aliases4)),
                    );
                }
                "java" | "kotlin" => {
                    let java_value = resolve.and_then(|r| r.get("java"));
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
                    resolvers.insert(".java", Box::new(java_resolver));
                    // Kotlin uses the same Java resolver logic
                    let java_value2 = resolve.and_then(|r| r.get("java"));
                    let kt_resolver = if let Some(jcfg) = java_value2 {
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
                    resolvers.insert(".kt", Box::new(kt_resolver));
                }
                "php" => {
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
                    resolvers.insert(".php", Box::new(php_resolver));
                }
                "c" => {
                    resolvers.insert(".c", Box::new(CResolver::new()));
                    resolvers.insert(".h", Box::new(CResolver::new()));
                }
                _ => {
                    // Unknown language — skip silently.
                    // The parser layer handles unsupported languages separately.
                }
            }
        }

        DispatchingResolver { resolvers }
    }

    /// Look up the resolver for a given file path by its extension.
    fn resolver_for(&self, file: &str) -> Option<&dyn Resolver> {
        let dot_pos = file.rfind('.')?;
        let ext = &file[dot_pos..];
        self.resolvers.get(ext).map(|r| r.as_ref())
    }
}

impl Resolver for DispatchingResolver {
    fn resolve(&self, import: &RawImport) -> ResolvedImport {
        if let Some(r) = self.resolver_for(&import.file) {
            r.resolve(import)
        } else {
            // Fallback: treat unknown extensions as Rust for backwards compatibility.
            RustResolver.resolve(import)
        }
    }

    fn resolve_for_project(&self, import: &RawImport, own_crate: &str) -> ResolvedImport {
        if let Some(r) = self.resolver_for(&import.file) {
            r.resolve_for_project(import, own_crate)
        } else {
            RustResolver.resolve_for_project(import, own_crate)
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
