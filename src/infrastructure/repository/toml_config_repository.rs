use crate::domain::entity::config::MilleConfig;
use crate::domain::repository::config_repository::ConfigRepository;
use std::fs;
use std::io::{Error, ErrorKind};

pub struct TomlConfigRepository;

impl TomlConfigRepository {
    /// Load config with two-pass parsing: extract `[resolve]` as raw `toml::Value`
    /// before deserializing the rest as `MilleConfig`.
    ///
    /// This keeps language-specific resolve configuration out of the domain layer.
    pub fn load_with_resolve(
        &self,
        path: &str,
    ) -> std::io::Result<(MilleConfig, Option<toml::Value>)> {
        let content = fs::read_to_string(path)?;
        let mut table: toml::Table = content
            .parse()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        // Extract the [resolve] section before deserializing as MilleConfig
        let resolve = table.remove("resolve");

        let config: MilleConfig = toml::Value::Table(table)
            .try_into()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

        Ok((config, resolve))
    }
}

impl ConfigRepository for TomlConfigRepository {
    fn load(&self, path: &str) -> std::io::Result<MilleConfig> {
        let (config, _resolve) = self.load_with_resolve(path)?;
        Ok(config)
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

    #[test]
    fn test_load_with_resolve_extracts_resolve_section() {
        let toml_content = r#"
[project]
name = "myproject"
root = "."
languages = ["go"]

[resolve.go]
module_name = "github.com/example/myproject"

[[layers]]
name = "domain"
paths = ["domain/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#;
        let temp_file = "test_resolve_extract.toml";
        fs::write(temp_file, toml_content).unwrap();

        let repo = TomlConfigRepository;
        let (config, resolve) = repo.load_with_resolve(temp_file).unwrap();

        assert_eq!(config.project.name, "myproject");
        assert!(resolve.is_some());
        let r = resolve.unwrap();
        assert_eq!(
            r.get("go")
                .and_then(|g| g.get("module_name"))
                .and_then(|v| v.as_str()),
            Some("github.com/example/myproject")
        );

        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_with_resolve_no_resolve_returns_none() {
        let toml_content = r#"
[project]
name = "myproject"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#;
        let temp_file = "test_no_resolve.toml";
        fs::write(temp_file, toml_content).unwrap();

        let repo = TomlConfigRepository;
        let (config, resolve) = repo.load_with_resolve(temp_file).unwrap();

        assert_eq!(config.project.name, "myproject");
        assert!(resolve.is_none());

        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_with_resolve_import_path_config() {
        let toml_content = r#"
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
        let temp_file = "test_resolve_import_path.toml";
        fs::write(temp_file, toml_content).unwrap();

        let repo = TomlConfigRepository;
        let (config, resolve) = repo.load_with_resolve(temp_file).unwrap();

        assert_eq!(config.project.name, "myproject");
        assert!(resolve.is_some());
        let r = resolve.unwrap();
        let pkgs = r
            .get("python")
            .and_then(|p| p.get("package_names"))
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(pkgs.len(), 2);

        fs::remove_file(temp_file).unwrap();
    }
}
