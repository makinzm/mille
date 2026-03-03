use crate::domain::entity::config::MilleConfig;
use crate::domain::repository::config_repository::ConfigRepository;
use std::fs;
use std::io::{Error, ErrorKind};

pub struct TomlConfigRepository;

impl ConfigRepository for TomlConfigRepository {
    fn load(&self, path: &str) -> std::io::Result<MilleConfig> {
        let content = fs::read_to_string(path)?;
        // We haven't implemented toml mapping yet, this will fail the target tests
        toml::from_str(&content).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::layer::DependencyMode;

    #[test]
    fn test_load_valid_toml() {
        let toml_content = r#"
[project]
name = "mille"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
allow = []
external_mode = "opt-in"
"#;
        // Temporarily write the content to a file to test the load
        let temp_file = "test_valid.toml";
        fs::write(temp_file, toml_content).unwrap();

        let repo = TomlConfigRepository;
        let config = repo.load(temp_file).unwrap();

        assert_eq!(config.project.name, "mille");
        assert_eq!(config.layers.len(), 1);
        assert_eq!(config.layers[0].name, "domain");
        assert_eq!(config.layers[0].dependency_mode, DependencyMode::OptIn);

        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_invalid_toml() {
        let toml_content = r#"
[project]
# missing basic project configs

[[layers]]
        "#;
        let temp_file = "test_invalid.toml";
        fs::write(temp_file, toml_content).unwrap();

        let repo = TomlConfigRepository;
        let result = repo.load(temp_file);

        assert!(result.is_err(), "Should return error for missing config");

        fs::remove_file(temp_file).unwrap();
    }
}
