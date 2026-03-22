use std::collections::BTreeSet;

use crate::domain::entity::layer::LayerConfig;

/// Port for generating language-specific resolve configuration sections.
///
/// The domain layer does not know about individual language configurations.
/// Concrete implementations live in the infrastructure layer.
pub trait ResolveConfigGenerator {
    /// Generate the TOML text for `[resolve.*]` sections based on detected languages
    /// and layer configurations.
    fn generate_resolve_toml(&self, languages: &[String], layers: &[LayerConfig]) -> String;

    /// Return the set of package names that are internal to the project.
    /// These should be filtered out of `external_allow` lists.
    fn internal_package_names(
        &self,
        languages: &[String],
        layers: &[LayerConfig],
    ) -> BTreeSet<String>;
}
