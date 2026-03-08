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
}
