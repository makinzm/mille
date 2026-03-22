use serde::Deserialize;

use crate::domain::entity::layer::LayerConfig;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub root: String,
    pub languages: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct IgnoreConfig {
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub test_patterns: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct ResolveConfig {
    pub typescript: Option<TsResolveConfig>,
    pub go: Option<GoResolveConfig>,
    pub python: Option<PythonResolveConfig>,
    pub java: Option<JavaResolveConfig>,
    #[serde(default)]
    pub aliases: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct TsResolveConfig {
    pub tsconfig: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct GoResolveConfig {
    pub module_name: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct JavaResolveConfig {
    /// Base package name that identifies internal imports.
    /// e.g. "com.example.myapp" — imports starting with this prefix are Internal.
    /// If omitted, mille auto-detects from `pom_xml` or `build_gradle`.
    #[serde(default)]
    pub module_name: Option<String>,
    /// Path to pom.xml (relative to mille.toml). If set, `groupId.artifactId`
    /// is used as the module name when `module_name` is not explicitly specified.
    #[serde(default)]
    pub pom_xml: Option<String>,
    /// Path to build.gradle (relative to mille.toml). If set, `group.rootProject.name`
    /// is used as the module name when `module_name` is not explicitly specified.
    #[serde(default)]
    pub build_gradle: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct PythonResolveConfig {
    /// Source root relative to project root. Optional — if omitted, mille derives it
    /// automatically from the importing file's path and `package_names`.
    /// NOTE: this field is currently not consumed by the resolver; the resolver always
    /// auto-derives the source root. The field is kept for forward compatibility.
    #[serde(default)]
    pub src_root: String,
    /// Top-level package names that are part of this project.
    /// Imports starting with any of these names are classified as Internal.
    #[serde(default)]
    pub package_names: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct SeverityConfig {
    #[serde(default = "default_error")]
    pub dependency_violation: String,
    #[serde(default = "default_error")]
    pub external_violation: String,
    #[serde(default = "default_error")]
    pub call_pattern_violation: String,
    #[serde(default = "default_warning")]
    pub unknown_import: String,
    #[serde(default = "default_error")]
    pub naming_violation: String,
}

fn default_error() -> String {
    "error".to_string()
}

fn default_warning() -> String {
    "warning".to_string()
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct MilleConfig {
    pub project: ProjectConfig,
    #[serde(rename = "layers", default)]
    pub layers: Vec<LayerConfig>,
    pub ignore: Option<IgnoreConfig>,
    pub resolve: Option<ResolveConfig>,
    #[serde(default = "default_severity")]
    pub severity: SeverityConfig,
}

fn default_severity() -> SeverityConfig {
    SeverityConfig {
        dependency_violation: default_error(),
        external_violation: default_error(),
        call_pattern_violation: default_error(),
        unknown_import: default_warning(),
        naming_violation: default_error(),
    }
}

impl Default for SeverityConfig {
    fn default() -> Self {
        default_severity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_resolve_config_without_src_root_parses() {
        // src_root なしの [resolve.python] は parse エラーにならないべき
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["python"]

[resolve.python]
package_names = ["domain", "usecase"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(
            result.is_ok(),
            "src_root なしで parse できるべき: {:?}",
            result.err()
        );
        let config = result.unwrap();
        let py = config
            .resolve
            .unwrap()
            .python
            .expect("python config should exist");
        assert_eq!(py.src_root, "");
        assert_eq!(py.package_names, vec!["domain", "usecase"]);
    }

    #[test]
    fn test_python_resolve_config_with_src_root_still_parses() {
        // 既存の src_root ありの設定は引き続き parse できる (regression)
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["python"]

[resolve.python]
src_root = "src"
package_names = ["domain"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(
            result.is_ok(),
            "src_root ありも parse できるべき: {:?}",
            result.err()
        );
        let config = result.unwrap();
        let py = config.resolve.unwrap().python.unwrap();
        assert_eq!(py.src_root, "src");
    }

    #[test]
    fn test_layer_config_with_name_deny_parses() {
        // name_deny を含む [[layers]] が parse できる
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["rust"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws", "gcp"]
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(result.is_ok(), "name_deny で parse できるべき: {:?}", result.err());
        let config = result.unwrap();
        assert_eq!(config.layers[0].name_deny, vec!["aws", "gcp"]);
    }

    #[test]
    fn test_layer_config_with_name_targets_parses() {
        use crate::domain::entity::layer::NameTarget;
        // name_targets を含む [[layers]] が parse できる
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["rust"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws"]
name_targets = ["file", "symbol"]
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(result.is_ok(), "name_targets で parse できるべき: {:?}", result.err());
        let config = result.unwrap();
        assert_eq!(
            config.layers[0].name_targets,
            vec![NameTarget::File, NameTarget::Symbol]
        );
    }

    #[test]
    fn test_layer_config_name_targets_default_is_all() {
        use crate::domain::entity::layer::NameTarget;
        // name_targets を省略したとき全ターゲットがデフォルトになる
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["rust"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws"]
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(result.is_ok(), "name_targets 省略で parse できるべき: {:?}", result.err());
        let config = result.unwrap();
        assert_eq!(config.layers[0].name_targets, NameTarget::all());
    }

    #[test]
    fn test_severity_config_with_naming_violation_parses() {
        // naming_violation = "error" を含む [severity] が parse できる
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["rust"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"

[severity]
naming_violation = "error"
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(result.is_ok(), "naming_violation で parse できるべき: {:?}", result.err());
        let config = result.unwrap();
        assert_eq!(config.severity.naming_violation, "error");
    }

    #[test]
    fn test_severity_config_naming_violation_default_is_error() {
        // naming_violation を省略したときデフォルトは "error"
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["rust"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.severity.naming_violation, "error");
    }
}
