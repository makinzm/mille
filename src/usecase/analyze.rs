use std::collections::HashMap;

use crate::domain::entity::resolved_import::ImportCategory;
use crate::domain::repository::config_repository::ConfigRepository;
use crate::domain::repository::parser::Parser;
use crate::domain::repository::resolver::Resolver;
use crate::domain::repository::source_file_repository::SourceFileRepository;

pub struct AnalyzeResult {
    pub nodes: Vec<LayerNode>,
    pub edges: Vec<LayerEdge>,
}

pub struct LayerNode {
    pub name: String,
    pub file_count: usize,
}

pub struct LayerEdge {
    pub from: String,
    pub to: String,
    pub import_count: usize,
}

/// Returns true if `path` matches any of the given glob patterns.
fn matches_any_glob(path: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pat| {
        glob::Pattern::new(pat)
            .map(|p| p.matches(path))
            .unwrap_or(false)
    })
}

/// Find the first layer whose paths glob patterns match `file_path`.
fn find_layer_name<'a>(
    file_path: &str,
    layers: &'a [crate::domain::entity::layer::LayerConfig],
) -> Option<&'a str> {
    layers.iter().find_map(|layer| {
        let matches = layer.paths.iter().any(|pat| {
            glob::Pattern::new(pat)
                .ok()
                .map(|p| p.matches(file_path))
                .unwrap_or(false)
        });
        if matches {
            Some(layer.name.as_str())
        } else {
            None
        }
    })
}

/// Run the analyze pipeline: parse & resolve imports, then aggregate layer-level edges.
/// Does NOT apply any violation rules — exit code is always 0.
pub fn analyze(
    config_path: &str,
    config_repo: &dyn ConfigRepository,
    file_repo: &dyn SourceFileRepository,
    parser: &dyn Parser,
    resolver: &dyn Resolver,
) -> Result<AnalyzeResult, String> {
    let config = config_repo.load(config_path).map_err(|e| e.to_string())?;

    let ignore_paths = config
        .ignore
        .as_ref()
        .map(|i| i.paths.as_slice())
        .unwrap_or(&[]);

    // Build nodes (file_count per layer)
    let mut file_counts: Vec<usize> = vec![0; config.layers.len()];
    let mut all_resolved = Vec::new();

    for (idx, layer) in config.layers.iter().enumerate() {
        let mut files = file_repo.collect(&layer.paths);
        files.retain(|f| !matches_any_glob(f, ignore_paths));
        file_counts[idx] = files.len();

        for file_path in &files {
            let source = std::fs::read_to_string(file_path)
                .map_err(|e| format!("failed to read {}: {}", file_path, e))?;
            let raw = parser.parse_imports(&source, file_path);
            all_resolved.extend(
                raw.iter()
                    .map(|r| resolver.resolve_for_project(r, &config.project.name)),
            );
        }
    }

    let nodes: Vec<LayerNode> = config
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| LayerNode {
            name: l.name.clone(),
            file_count: file_counts[i],
        })
        .collect();

    // Aggregate layer-level edges: (from_layer, to_layer) → count
    let mut edge_counts: HashMap<(String, String), usize> = HashMap::new();

    for import in &all_resolved {
        if import.category != ImportCategory::Internal {
            continue;
        }
        let Some(from_layer) = find_layer_name(&import.raw.file, &config.layers) else {
            continue;
        };
        let Some(resolved_path) = &import.resolved_path else {
            continue;
        };
        let Some(to_layer) = find_layer_name(resolved_path, &config.layers) else {
            continue;
        };
        if from_layer == to_layer {
            continue;
        }
        *edge_counts
            .entry((from_layer.to_string(), to_layer.to_string()))
            .or_insert(0) += 1;
    }

    // Sort edges for deterministic output
    let mut edges: Vec<LayerEdge> = edge_counts
        .into_iter()
        .map(|((from, to), count)| LayerEdge {
            from,
            to,
            import_count: count,
        })
        .collect();
    edges.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

    Ok(AnalyzeResult { nodes, edges })
}
