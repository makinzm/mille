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
    fn test_config_with_resolve_section_still_parses() {
        // [resolve] section is now handled by infrastructure's two-pass parsing,
        // but MilleConfig should still parse when [resolve] is absent.
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["mylang"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(result.is_ok(), "parse should succeed: {:?}", result.err());
    }

    #[test]
    fn test_layer_config_with_name_deny_parses() {
        // name_deny を含む [[layers]] が parse できる
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["mylang"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws", "gcp"]
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(
            result.is_ok(),
            "name_deny で parse できるべき: {:?}",
            result.err()
        );
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
languages = ["mylang"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws"]
name_targets = ["file", "symbol"]
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(
            result.is_ok(),
            "name_targets で parse できるべき: {:?}",
            result.err()
        );
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
languages = ["mylang"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws"]
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(
            result.is_ok(),
            "name_targets 省略で parse できるべき: {:?}",
            result.err()
        );
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
languages = ["mylang"]

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"

[severity]
naming_violation = "error"
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(
            result.is_ok(),
            "naming_violation で parse できるべき: {:?}",
            result.err()
        );
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
languages = ["mylang"]

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

    #[test]
    fn test_layer_config_with_name_deny_ignore_parses() {
        // name_deny_ignore を含む [[layers]] が parse できる
        let toml = r#"
[project]
name = "myproject"
root = "."
languages = ["mylang"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-out"
external_mode = "opt-out"
name_deny = ["aws", "gcp"]
name_deny_ignore = ["**/test_*.rs", "tests/**"]
"#;
        let result = toml::from_str::<MilleConfig>(toml);
        assert!(
            result.is_ok(),
            "name_deny_ignore で parse できるべき: {:?}",
            result.err()
        );
        let config = result.unwrap();
        assert_eq!(
            config.layers[0].name_deny_ignore,
            vec!["**/test_*.rs", "tests/**"]
        );
    }
}
