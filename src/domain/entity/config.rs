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
