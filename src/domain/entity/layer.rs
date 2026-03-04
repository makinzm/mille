use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct LayerConfig {
    pub name: String,
    pub paths: Vec<String>,
    pub dependency_mode: DependencyMode,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    pub external_mode: DependencyMode,
    #[serde(default)]
    pub external_allow: Vec<String>,
    #[serde(default)]
    pub external_deny: Vec<String>,
    #[serde(default)]
    pub allow_call_patterns: Vec<CallPattern>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyMode {
    OptIn,
    OptOut,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
pub struct CallPattern {
    pub callee_layer: String,
    pub allow_methods: Vec<String>,
}
