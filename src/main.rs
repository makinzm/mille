use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use clap::Parser;
use mille::domain::entity::import::ImportKind;
use mille::domain::entity::violation::Severity;
use mille::domain::repository::config_repository::ConfigRepository;
use mille::domain::repository::parser::Parser as SourceParser;
use mille::infrastructure::parser::DispatchingParser;
use mille::infrastructure::repository::fs_source_file_repository::FsSourceFileRepository;
use mille::infrastructure::repository::toml_config_repository::TomlConfigRepository;
use mille::infrastructure::resolver::DispatchingResolver;
use mille::presentation::cli::args::Format;
use mille::presentation::cli::args::{Cli, Command};
use mille::presentation::formatter::github_actions::format_all_ga;
use mille::presentation::formatter::json::format_json;
use mille::presentation::formatter::terminal::{
    format_layer_stats, format_summary, format_violation,
};
use mille::usecase::check_architecture;
use mille::usecase::init::{self, DirAnalysis};

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Init {
            output,
            force,
            depth,
        } => {
            let cwd = std::env::current_dir()
                .expect("cannot determine current directory")
                .to_string_lossy()
                .to_string();

            let output_path = std::path::Path::new(&output);

            // Guard: refuse to overwrite unless --force is set
            if output_path.exists() && !force {
                eprintln!(
                    "Error: '{}' already exists. Use --force to overwrite.",
                    output
                );
                std::process::exit(1);
            }

            let project_name = std::path::Path::new(&cwd)
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("project"))
                .to_string_lossy()
                .to_string();

            let languages = init::detect_languages(&cwd);
            println!("Detected languages: {}", languages.join(", "));

            println!("Scanning imports...");
            let parser = DispatchingParser::new();
            let analyses = scan_project(Path::new(&cwd), &parser, depth);

            let mut layers = init::infer_layers(&analyses);

            // Print dependency graph for hospitality
            if !layers.is_empty() {
                println!("\nInferred layer structure:");
                for layer in &layers {
                    if layer.allow.is_empty() {
                        println!("  {:<20} ← (no internal dependencies)", layer.name);
                    } else {
                        println!("  {:<20} → {}", layer.name, layer.allow.join(", "));
                    }
                    if !layer.external_allow.is_empty() {
                        println!("    external: {}", layer.external_allow.join(", "));
                    }
                }
            } else {
                println!("No layers detected.");
            }

            // Append /** glob to each path
            for layer in &mut layers {
                layer.paths = layer.paths.iter().map(|p| format!("{}/**", p)).collect();
            }

            let toml_content = init::generate_toml(&project_name, ".", &languages, &layers);

            match std::fs::write(output_path, &toml_content) {
                Ok(_) => println!("\nGenerated '{}'", output),
                Err(e) => {
                    eprintln!("Error: failed to write '{}': {}", output, e);
                    std::process::exit(1);
                }
            }
        }
        Command::Check { config, format } => {
            // Pre-load config to build the resolver, then pass path to check().
            // NOTE: Double-load is acceptable for a CLI tool.
            let config_repo = TomlConfigRepository;
            let app_config = match config_repo.load(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(3);
                }
            };

            let parser = DispatchingParser::new();
            let resolver = DispatchingResolver::from_config(&app_config, &config);

            match check_architecture::check(
                &config,
                &config_repo,
                &FsSourceFileRepository,
                &parser,
                &resolver,
            ) {
                Ok(result) => {
                    match format {
                        Format::Terminal => {
                            for v in &result.violations {
                                print!("{}", format_violation(v));
                            }
                            print!("{}", format_layer_stats(&result.layer_stats));
                            print!("{}", format_summary(&result.violations));
                        }
                        Format::GithubActions => {
                            print!("{}", format_all_ga(&result.violations));
                        }
                        Format::Json => {
                            print!("{}", format_json(&result.violations));
                        }
                    }

                    let has_error = result
                        .violations
                        .iter()
                        .any(|v| v.severity == Severity::Error);
                    if has_error {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(3);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Project scanning — builds DirAnalysis per source directory
// ---------------------------------------------------------------------------

fn scan_project(
    root: &Path,
    parser: &DispatchingParser,
    depth: Option<usize>,
) -> BTreeMap<String, DirAnalysis> {
    // Pass 1: collect all directories that contain at least one source file
    let mut all_source_dirs: BTreeSet<String> = BTreeSet::new();
    collect_source_dirs(root, root, &mut all_source_dirs);

    // Determine the target depth (explicit or auto-detected)
    let target_depth = depth.unwrap_or_else(|| auto_detect_layer_depth(&all_source_dirs));
    println!("Using layer depth: {}", target_depth);

    // Compute layer dirs: roll up every source dir to the target depth
    let layer_dirs: BTreeSet<String> = all_source_dirs
        .iter()
        .filter_map(|d| ancestor_at_depth(d, target_depth))
        .collect();

    // Pass 2: for each source file, parse imports and build DirAnalysis
    let mut analyses: BTreeMap<String, DirAnalysis> = BTreeMap::new();
    for dir in &layer_dirs {
        analyses.insert(dir.clone(), DirAnalysis::default());
    }
    collect_dir_imports(root, root, parser, &layer_dirs, target_depth, &mut analyses);

    analyses
}

fn is_source_file(name: &str) -> bool {
    matches!(
        name.rsplit('.').next().unwrap_or(""),
        "rs" | "ts" | "tsx" | "js" | "jsx" | "go" | "py"
    )
}

fn collect_source_dirs(root: &Path, dir: &Path, known_dirs: &mut BTreeSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut has_source = false;
    let mut subdirs: Vec<std::path::PathBuf> = vec![];

    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if init::is_excluded_dir(&name) {
            continue;
        }
        if path.is_dir() {
            subdirs.push(path);
        } else if is_source_file(&name) {
            has_source = true;
        }
    }

    if has_source {
        let rel = dir.strip_prefix(root).unwrap_or(dir);
        let rel_str = rel.to_string_lossy().to_string();
        if !rel_str.is_empty() {
            known_dirs.insert(rel_str);
        }
    }

    for subdir in subdirs {
        collect_source_dirs(root, &subdir, known_dirs);
    }
}

fn collect_dir_imports(
    root: &Path,
    dir: &Path,
    parser: &DispatchingParser,
    layer_dirs: &BTreeSet<String>,
    target_depth: usize,
    analyses: &mut BTreeMap<String, DirAnalysis>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        if init::is_excluded_dir(&name) {
            continue;
        }
        if path.is_dir() {
            collect_dir_imports(root, &path, parser, layer_dirs, target_depth, analyses);
        } else if is_source_file(&name) {
            process_source_file(root, &path, parser, layer_dirs, target_depth, analyses);
        }
    }
}

fn process_source_file(
    root: &Path,
    file: &Path,
    parser: &DispatchingParser,
    layer_dirs: &BTreeSet<String>,
    target_depth: usize,
    analyses: &mut BTreeMap<String, DirAnalysis>,
) {
    let Ok(source) = fs::read_to_string(file) else {
        return;
    };
    let file_rel = file.strip_prefix(root).unwrap_or(file);
    let file_rel_str = file_rel.to_string_lossy().to_string();

    let dir_rel = file
        .parent()
        .and_then(|p| p.strip_prefix(root).ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if dir_rel.is_empty() {
        return;
    }

    // Roll up the immediate dir to the target layer depth
    let layer_dir = match ancestor_at_depth(&dir_rel, target_depth) {
        Some(d) => d,
        None => return, // shallower than target depth → skip
    };

    let imports = SourceParser::parse_imports(parser, &source, &file_rel_str);
    let analysis = analyses.entry(layer_dir.clone()).or_default();
    analysis.file_count += 1;

    for imp in &imports {
        // Skip Rust submodule declarations (mod foo;) — they are not cross-layer imports
        if imp.kind == ImportKind::Mod {
            continue;
        }
        match classify_import_for_init(&imp.path, &file_rel_str) {
            Some(InitImport::Internal(seg)) => {
                // Definitely internal — only add as dep if dir found; never as external
                if let Some(dep_dir) = resolve_to_known_dir(&seg, &layer_dir, layer_dirs) {
                    if dep_dir != layer_dir {
                        analysis.internal_deps.insert(dep_dir);
                    }
                }
            }
            Some(InitImport::External(pkg)) => {
                analysis.external_pkgs.insert(pkg);
            }
            Some(InitImport::TryInternal(seg)) => {
                // Try internal first; if no known dir matches, record as external pkg
                if let Some(dep_dir) = resolve_to_known_dir(&seg, &layer_dir, layer_dirs) {
                    if dep_dir != layer_dir {
                        analysis.internal_deps.insert(dep_dir);
                    }
                } else {
                    analysis.external_pkgs.insert(seg);
                }
            }
            None => {}
        }
    }
}

enum InitImport {
    /// Definitely internal (e.g. Rust `crate::X`): don't add as external if dir not found.
    Internal(String),
    /// Definitely external (e.g. Rust `serde::X`): always add to external_pkgs.
    External(String),
    /// Ambiguous (e.g. Go `pkg/X`, Python `X.Y`): try internal match first, fall back to external.
    TryInternal(String),
}

fn classify_import_for_init(path: &str, file_path: &str) -> Option<InitImport> {
    if file_path.ends_with(".rs") {
        classify_rust_import(path)
    } else if file_path.ends_with(".ts")
        || file_path.ends_with(".tsx")
        || file_path.ends_with(".js")
        || file_path.ends_with(".jsx")
    {
        classify_ts_import(path, file_path)
    } else if file_path.ends_with(".go") {
        classify_go_import(path)
    } else if file_path.ends_with(".py") {
        classify_py_import(path)
    } else {
        None
    }
}

fn classify_rust_import(path: &str) -> Option<InitImport> {
    // Stdlib
    if path.starts_with("std::") || path.starts_with("core::") || path.starts_with("alloc::") {
        return None;
    }
    // Self/super — same module, not useful for layer detection
    if path.starts_with("self::") || path.starts_with("super::") {
        return None;
    }
    // Internal: crate::X::...
    if let Some(rest) = path.strip_prefix("crate::") {
        let seg = rest.split("::").next()?.to_string();
        if seg.contains('{') || seg.contains('*') || seg.is_empty() {
            return None;
        }
        return Some(InitImport::Internal(seg));
    }
    // External crate
    let pkg = path.split("::").next()?.to_string();
    if pkg.is_empty() || pkg.contains('{') {
        return None;
    }
    Some(InitImport::External(pkg))
}

fn classify_ts_import(path: &str, _file_path: &str) -> Option<InitImport> {
    if path.starts_with("./") || path.starts_with("../") {
        // Relative import — strip leading `./` and `../` prefixes to get first segment
        let mut p: &str = path;
        loop {
            if let Some(rest) = p.strip_prefix("./") {
                p = rest;
            } else if let Some(rest) = p.strip_prefix("../") {
                p = rest;
            } else {
                break;
            }
        }
        let seg = p.split('/').next()?.to_string();
        if seg.is_empty() {
            return None;
        }
        return Some(InitImport::TryInternal(seg));
    }
    // Absolute / package import
    let pkg = if path.starts_with('@') {
        // scoped package: @scope/name
        let mut parts = path.splitn(3, '/');
        let scope = parts.next()?;
        let name = parts.next()?;
        format!("{}/{}", scope, name)
    } else {
        path.split('/').next()?.to_string()
    };
    if pkg.is_empty() {
        return None;
    }
    Some(InitImport::External(pkg))
}

fn classify_go_import(path: &str) -> Option<InitImport> {
    // Go imports look like "github.com/org/repo/pkg" or "fmt", "os", etc.
    // stdlib: no dots in the first segment
    let first = path.split('/').next()?;
    if !first.contains('.') {
        return None; // stdlib
    }
    // The last segment is the package name
    // Try to match as internal first; if not found, record as external
    let seg = path.split('/').last()?.to_string();
    Some(InitImport::TryInternal(seg))
}

fn classify_py_import(path: &str) -> Option<InitImport> {
    // Relative imports start with '.'
    if path.starts_with('.') {
        let trimmed = path.trim_start_matches('.');
        let seg = trimmed.split('.').next()?.to_string();
        if seg.is_empty() {
            return None;
        }
        return Some(InitImport::TryInternal(seg));
    }
    // Absolute import — first segment might be internal or external
    let seg = path.split('.').next()?.to_string();
    if seg.is_empty() {
        return None;
    }
    Some(InitImport::TryInternal(seg))
}

/// Return the ancestor of `dir` at `depth` path segments, or `None` if `dir` is shallower.
///
/// * `"src/domain/entity", 2` → `Some("src/domain")`
/// * `"src/domain", 2`        → `Some("src/domain")`
/// * `"src", 2`               → `None`
fn ancestor_at_depth(dir: &str, depth: usize) -> Option<String> {
    let segments: Vec<&str> = dir.split('/').collect();
    if segments.len() < depth {
        return None;
    }
    Some(segments[..depth].join("/"))
}

/// Known directory names that are source-layout roots, not layers themselves.
const SOURCE_ROOTS: &[&str] = &["src", "lib", "app", "source", "pkg", "packages"];

/// Auto-detect the layer depth from the set of all source directories.
///
/// Tries depths 1..=6. For each depth, computes candidate layer dirs and filters
/// out those whose base name is a known source root (e.g. `src`, `lib`).
/// Returns the first depth that yields 2–8 candidates. Defaults to 1.
fn auto_detect_layer_depth(all_source_dirs: &BTreeSet<String>) -> usize {
    for depth in 1..=6 {
        let candidates: BTreeSet<String> = all_source_dirs
            .iter()
            .filter_map(|d| ancestor_at_depth(d, depth))
            .filter(|d| {
                let base = d.split('/').next_back().unwrap_or(d.as_str());
                !SOURCE_ROOTS.contains(&base)
            })
            .collect();
        if candidates.len() >= 2 && candidates.len() <= 8 {
            return depth;
        }
    }
    1
}

/// Find a known directory whose base name matches `module_seg`.
/// Prefers directories that share the same parent as `current_dir`.
fn resolve_to_known_dir(
    module_seg: &str,
    current_dir: &str,
    known_dirs: &BTreeSet<String>,
) -> Option<String> {
    let current_parent = current_dir.rsplit_once('/').map(|(p, _)| p).unwrap_or("");

    // First try: sibling dir (same parent directory)
    for dir in known_dirs {
        let base = dir.rsplit('/').next().unwrap_or(dir.as_str());
        if base == module_seg && dir.as_str() != current_dir {
            let parent = dir.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
            if parent == current_parent {
                return Some(dir.clone());
            }
        }
    }

    // Fallback: any known dir with matching base name
    for dir in known_dirs {
        let base = dir.rsplit('/').next().unwrap_or(dir.as_str());
        if base == module_seg && dir.as_str() != current_dir {
            return Some(dir.clone());
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ancestor_at_depth_deep_dir() {
        assert_eq!(
            ancestor_at_depth("src/domain/entity", 2),
            Some("src/domain".to_string())
        );
    }

    #[test]
    fn test_ancestor_at_depth_exact_depth() {
        assert_eq!(
            ancestor_at_depth("src/domain", 2),
            Some("src/domain".to_string())
        );
    }

    #[test]
    fn test_ancestor_at_depth_too_shallow() {
        assert_eq!(ancestor_at_depth("src", 2), None);
    }

    #[test]
    fn test_ancestor_at_depth_depth_one() {
        assert_eq!(ancestor_at_depth("domain", 1), Some("domain".to_string()));
    }

    #[test]
    fn test_auto_detect_layer_depth_src_prefix() {
        // src/domain, src/usecase, src/infrastructure → depth=2
        let dirs: BTreeSet<String> = ["src/domain", "src/usecase", "src/infrastructure"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(auto_detect_layer_depth(&dirs), 2);
    }

    #[test]
    fn test_auto_detect_layer_depth_flat_layout() {
        // domain, usecase, infrastructure at depth=1 → depth=1
        let dirs: BTreeSet<String> = ["domain", "usecase", "infrastructure"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(auto_detect_layer_depth(&dirs), 1);
    }

    #[test]
    fn test_auto_detect_layer_depth_skips_source_roots() {
        // Only "src" at depth=1 (filtered out) → continue to depth=2
        let dirs: BTreeSet<String> = ["src/domain", "src/usecase", "src/infrastructure"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        // depth=1 yields {"src"} → filtered → 0 candidates → skip
        // depth=2 yields {"src/domain", "src/usecase", "src/infrastructure"} → 3 → use
        assert_eq!(auto_detect_layer_depth(&dirs), 2);
    }
}
