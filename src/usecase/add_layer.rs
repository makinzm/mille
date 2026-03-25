use std::collections::BTreeSet;

use crate::domain::entity::layer::{DependencyMode, LayerConfig, NameTarget};
use crate::usecase::init::DirAnalysis;

/// Information about a conflicting existing layer.
pub struct ConflictInfo {
    pub layer_index: usize,
    pub layer_name: String,
    pub overlapping_paths: Vec<String>,
}

/// Check if any existing layer conflicts with the target glob.
///
/// Conflict is detected when:
/// - Exact match (after stripping `/**` suffix)
/// - Parent-child relationship (one path is a prefix of the other)
pub fn find_conflict(existing_layers: &[LayerConfig], target_glob: &str) -> Option<ConflictInfo> {
    let target_base = target_glob.trim_end_matches("/**");

    for (i, layer) in existing_layers.iter().enumerate() {
        let mut overlapping = vec![];
        for path in &layer.paths {
            let existing_base = path.trim_end_matches("/**");
            if existing_base == target_base
                || existing_base.starts_with(&format!("{}/", target_base))
                || target_base.starts_with(&format!("{}/", existing_base))
            {
                overlapping.push(path.clone());
            }
        }
        if !overlapping.is_empty() {
            return Some(ConflictInfo {
                layer_index: i,
                layer_name: layer.name.clone(),
                overlapping_paths: overlapping,
            });
        }
    }
    None
}

/// Build a LayerConfig from a DirAnalysis result.
pub fn build_layer_config(name: &str, target_glob: &str, analysis: &DirAnalysis) -> LayerConfig {
    LayerConfig {
        name: name.to_string(),
        paths: vec![target_glob.to_string()],
        dependency_mode: DependencyMode::OptIn,
        allow: analysis.internal_deps.iter().cloned().collect(),
        deny: vec![],
        external_mode: DependencyMode::OptIn,
        external_allow: analysis.external_pkgs.iter().cloned().collect(),
        external_deny: vec![],
        allow_call_patterns: vec![],
        name_deny: vec![],
        name_allow: vec![],
        name_targets: NameTarget::all(),
        name_deny_ignore: vec![],
    }
}

/// Convert a LayerConfig to a TOML string suitable for appending to a config file.
pub fn layer_to_toml_string(layer: &LayerConfig) -> String {
    let mut out = String::new();
    out.push_str("\n[[layers]]\n");
    out.push_str(&format!("name = \"{}\"\n", layer.name));

    if layer.paths.len() == 1 {
        out.push_str(&format!("paths = [\"{}\"]", layer.paths[0]));
    } else {
        out.push_str("paths = [\n");
        for path in &layer.paths {
            out.push_str(&format!("  \"{}\",\n", path));
        }
        out.push(']');
    }
    out.push('\n');

    out.push_str(&format!(
        "dependency_mode = \"{}\"\n",
        mode_str(layer.dependency_mode)
    ));

    if !layer.allow.is_empty() {
        let allow_str = layer
            .allow
            .iter()
            .map(|a| format!("\"{}\"", a))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("allow = [{}]\n", allow_str));
    }

    out.push_str(&format!(
        "external_mode = \"{}\"\n",
        mode_str(layer.external_mode)
    ));

    if !layer.external_allow.is_empty() {
        let ext_str = layer
            .external_allow
            .iter()
            .map(|e| format!("\"{}\"", e))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("external_allow = [{}]\n", ext_str));
    }

    out
}

/// Replace a layer at `index` in a toml::Table's `layers` array.
pub fn replace_layer_in_table(
    table: &mut toml::Table,
    index: usize,
    layer: &LayerConfig,
) -> Result<(), String> {
    let layers = table
        .get_mut("layers")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "no [[layers]] array in config".to_string())?;

    if index >= layers.len() {
        return Err(format!(
            "layer index {} out of range ({})",
            index,
            layers.len()
        ));
    }

    // Serialize the new layer to a toml::Value
    let mut layer_table = toml::Table::new();
    layer_table.insert("name".into(), toml::Value::String(layer.name.clone()));
    layer_table.insert(
        "paths".into(),
        toml::Value::Array(
            layer
                .paths
                .iter()
                .map(|p| toml::Value::String(p.clone()))
                .collect(),
        ),
    );
    layer_table.insert(
        "dependency_mode".into(),
        toml::Value::String(mode_str(layer.dependency_mode).to_string()),
    );
    if !layer.allow.is_empty() {
        layer_table.insert(
            "allow".into(),
            toml::Value::Array(
                layer
                    .allow
                    .iter()
                    .map(|a| toml::Value::String(a.clone()))
                    .collect(),
            ),
        );
    }
    layer_table.insert(
        "external_mode".into(),
        toml::Value::String(mode_str(layer.external_mode).to_string()),
    );
    if !layer.external_allow.is_empty() {
        layer_table.insert(
            "external_allow".into(),
            toml::Value::Array(
                layer
                    .external_allow
                    .iter()
                    .map(|e| toml::Value::String(e.clone()))
                    .collect(),
            ),
        );
    }

    layers[index] = toml::Value::Table(layer_table);
    Ok(())
}

fn mode_str(m: DependencyMode) -> &'static str {
    match m {
        DependencyMode::OptIn => "opt-in",
        DependencyMode::OptOut => "opt-out",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn make_layer(name: &str, paths: Vec<&str>) -> LayerConfig {
        LayerConfig {
            name: name.to_string(),
            paths: paths.into_iter().map(|p| p.to_string()).collect(),
            dependency_mode: DependencyMode::OptIn,
            allow: vec![],
            deny: vec![],
            external_mode: DependencyMode::OptIn,
            external_allow: vec![],
            external_deny: vec![],
            allow_call_patterns: vec![],
            name_deny: vec![],
            name_allow: vec![],
            name_targets: NameTarget::all(),
            name_deny_ignore: vec![],
        }
    }

    // ---------------------------------------------------------------
    // find_conflict
    // ---------------------------------------------------------------

    #[test]
    fn find_conflict_no_overlap() {
        let layers = vec![
            make_layer("domain", vec!["src/domain/**"]),
            make_layer("usecase", vec!["src/usecase/**"]),
        ];
        assert!(find_conflict(&layers, "src/newlayer/**").is_none());
    }

    #[test]
    fn find_conflict_exact_match() {
        let layers = vec![
            make_layer("domain", vec!["src/domain/**"]),
            make_layer("usecase", vec!["src/usecase/**"]),
        ];
        let conflict = find_conflict(&layers, "src/domain/**");
        assert!(conflict.is_some());
        let c = conflict.unwrap();
        assert_eq!(c.layer_index, 0);
        assert_eq!(c.layer_name, "domain");
    }

    #[test]
    fn find_conflict_parent_child() {
        let layers = vec![make_layer("src", vec!["src/**"])];
        let conflict = find_conflict(&layers, "src/domain/**");
        assert!(conflict.is_some());
        let c = conflict.unwrap();
        assert_eq!(c.layer_name, "src");

        // Reverse: target is parent of existing
        let layers2 = vec![make_layer("domain", vec!["src/domain/**"])];
        let conflict2 = find_conflict(&layers2, "src/**");
        assert!(conflict2.is_some());
    }

    // ---------------------------------------------------------------
    // build_layer_config
    // ---------------------------------------------------------------

    #[test]
    fn build_layer_config_defaults() {
        let analysis = DirAnalysis::default();
        let config = build_layer_config("newlayer", "src/newlayer/**", &analysis);
        assert_eq!(config.name, "newlayer");
        assert_eq!(config.paths, vec!["src/newlayer/**"]);
        assert_eq!(config.dependency_mode, DependencyMode::OptIn);
        assert_eq!(config.external_mode, DependencyMode::OptIn);
        assert!(config.allow.is_empty());
        assert!(config.external_allow.is_empty());
    }

    #[test]
    fn build_layer_config_with_external_deps() {
        let mut ext = BTreeSet::new();
        ext.insert("serde".to_string());
        ext.insert("tokio".to_string());
        let analysis = DirAnalysis {
            external_pkgs: ext,
            ..Default::default()
        };
        let config = build_layer_config("infra", "src/infra/**", &analysis);
        assert!(config.external_allow.contains(&"serde".to_string()));
        assert!(config.external_allow.contains(&"tokio".to_string()));
    }

    #[test]
    fn build_layer_config_with_internal_deps() {
        let mut deps = BTreeSet::new();
        deps.insert("domain".to_string());
        let analysis = DirAnalysis {
            internal_deps: deps,
            ..Default::default()
        };
        let config = build_layer_config("usecase", "src/usecase/**", &analysis);
        assert!(config.allow.contains(&"domain".to_string()));
    }

    // ---------------------------------------------------------------
    // layer_to_toml_string
    // ---------------------------------------------------------------

    #[test]
    fn layer_to_toml_string_format() {
        let layer = make_layer("newlayer", vec!["src/newlayer/**"]);
        let toml_str = layer_to_toml_string(&layer);
        assert!(toml_str.contains("[[layers]]"));
        assert!(toml_str.contains("name = \"newlayer\""));
        assert!(toml_str.contains("paths = [\"src/newlayer/**\"]"));
        assert!(toml_str.contains("dependency_mode = \"opt-in\""));
        assert!(toml_str.contains("external_mode = \"opt-in\""));
    }

    // ---------------------------------------------------------------
    // replace_layer_in_table
    // ---------------------------------------------------------------

    #[test]
    fn replace_layer_in_table_preserves_others() {
        let toml_content = r#"
[project]
name = "test"
root = "."
languages = ["rust"]

[[layers]]
name = "domain"
paths = ["src/domain/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"

[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
external_mode = "opt-in"
"#;
        let mut table: toml::Table = toml_content.parse().unwrap();

        let new_layer = make_layer("domain_v2", vec!["src/domain/**"]);
        replace_layer_in_table(&mut table, 0, &new_layer).unwrap();

        // Project section preserved
        assert!(table.contains_key("project"));

        // Layers array still has 2 entries
        let layers = table["layers"].as_array().unwrap();
        assert_eq!(layers.len(), 2);

        // First layer replaced
        assert_eq!(
            layers[0].get("name").unwrap().as_str().unwrap(),
            "domain_v2"
        );
        // Second layer preserved
        assert_eq!(
            layers[1].get("name").unwrap().as_str().unwrap(),
            "usecase"
        );
    }
}
