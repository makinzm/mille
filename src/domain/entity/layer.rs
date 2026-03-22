use serde::Deserialize;

use crate::domain::entity::name::NameKind;

/// Which `name_targets` to check for naming violations.
#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum NameTarget {
    File,
    Symbol,
    Variable,
    Comment,
}

impl NameTarget {
    pub fn all() -> Vec<NameTarget> {
        vec![
            NameTarget::File,
            NameTarget::Symbol,
            NameTarget::Variable,
            NameTarget::Comment,
        ]
    }

    pub fn as_name_kind(self) -> NameKind {
        match self {
            NameTarget::File => NameKind::File,
            NameTarget::Symbol => NameKind::Symbol,
            NameTarget::Variable => NameKind::Variable,
            NameTarget::Comment => NameKind::Comment,
        }
    }
}

fn default_name_targets() -> Vec<NameTarget> {
    NameTarget::all()
}

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
    /// Forbidden keywords for naming convention check (case-insensitive partial match).
    #[serde(default)]
    pub name_deny: Vec<String>,
    /// Substrings that are explicitly allowed even if they contain a denied keyword.
    /// Before checking `name_deny`, each allowed string is stripped from the identifier.
    /// Example: `name_allow = ["category"]` prevents "ImportCategory" from being flagged
    /// for keyword matching (because "category" is stripped first, leaving no match).
    #[serde(default)]
    pub name_allow: Vec<String>,
    /// Which targets to check. Defaults to all targets when omitted.
    #[serde(default = "default_name_targets")]
    pub name_targets: Vec<NameTarget>,
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
