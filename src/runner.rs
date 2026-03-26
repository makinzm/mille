//! CLI entry point shared by the native binary and the PyPI wrapper.
//!
//! Call [`run_cli`] from `main()` or from the Python `_main` shim.
//! Any new subcommand added to [`crate::presentation::cli::args::Command`]
//! is automatically available to all distributions — no per-package updates needed.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use clap::Parser;

use crate::domain::entity::import::ImportKind;
use crate::domain::entity::violation::Severity;
use crate::domain::repository::parser::Parser as SourceParser;
use crate::infrastructure::parser::DispatchingParser;
use crate::infrastructure::repository::fs_source_file_repository::FsSourceFileRepository;
use crate::infrastructure::repository::toml_config_repository::TomlConfigRepository;
use crate::infrastructure::resolver::DispatchingResolver;
use crate::presentation::cli::args::AnalyzeFormat;
use crate::presentation::cli::args::FailOn;
use crate::presentation::cli::args::Format;
use crate::presentation::cli::args::ReportExternalFormat;
use crate::presentation::cli::args::{Cli, Command, ReportCommand};
use crate::presentation::formatter::github_actions::format_all_ga;
use crate::presentation::formatter::json::format_json;
use crate::presentation::formatter::svg::format_svg;
use crate::presentation::formatter::terminal::{
    format_layer_stats, format_summary, format_violation,
};
use crate::usecase::add_layer;
use crate::usecase::analyze;
use crate::usecase::check_architecture;
use crate::usecase::init::{self, DirAnalysis};
use crate::usecase::report_external;

/// Parse `std::env::args()` and run the matching subcommand.
///
/// Exits the process with the appropriate code on error or violation.
pub fn run_cli() {
    let cli = Cli::parse();
    run_cli_inner(cli);
}

/// Parse `args` and run the matching subcommand.
///
/// Use this from non-native entry points (e.g. the Python wrapper) where
/// `std::env::args()` contains extra interpreter-injected arguments that
/// clap would otherwise misinterpret as subcommands.
pub fn run_cli_from<I, T>(args: I)
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);
    run_cli_inner(cli);
}

/// Change the working directory to the specified path if it is not ".".
fn apply_path(path: &str) {
    if path != "." {
        let target = std::path::Path::new(path);
        if !target.is_dir() {
            eprintln!("Error: '{}' is not a directory or does not exist", path);
            std::process::exit(3);
        }
        if let Err(e) = std::env::set_current_dir(target) {
            eprintln!("Error: failed to change directory to '{}': {}", path, e);
            std::process::exit(3);
        }
    }
}

fn run_cli_inner(cli: Cli) {
    // Add command does NOT call apply_path — it operates relative to cwd
    if !matches!(cli.command, Command::Add { .. }) {
        apply_path(&cli.command.common().path);
    }

    match cli.command {
        Command::Report { subcommand } => match subcommand {
            ReportCommand::External {
                common: _,
                config,
                format,
                output,
            } => {
                let config_repo = TomlConfigRepository;
                let (app_config, resolve) = match config_repo.load_with_resolve(&config) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(3);
                    }
                };

                let parser = DispatchingParser::new();
                let resolver = DispatchingResolver::from_resolve_config(
                    resolve.as_ref(),
                    &config,
                    &app_config.project.languages,
                );

                match report_external::report_external(
                    &config,
                    &config_repo,
                    &FsSourceFileRepository,
                    &parser,
                    &resolver,
                ) {
                    Ok(result) => {
                        let content = match format {
                            ReportExternalFormat::Terminal => {
                                format_report_external_terminal(&result)
                            }
                            ReportExternalFormat::Json => format_report_external_json(&result),
                        };

                        match output {
                            Some(path) => {
                                if std::path::Path::new(&path).exists() {
                                    eprintln!(
                                        "Error: '{}' already exists. Remove it first if you want to overwrite.",
                                        path
                                    );
                                    std::process::exit(1);
                                }
                                match fs::write(&path, &content) {
                                    Ok(_) => eprintln!("Written to '{}'", path),
                                    Err(e) => {
                                        eprintln!("Error: failed to write '{}': {}", path, e);
                                        std::process::exit(1);
                                    }
                                }
                            }
                            None => print!("{}", content),
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(3);
                    }
                }
            }
        },
        Command::Analyze {
            common: _,
            config,
            format,
            output,
        } => {
            let config_repo = TomlConfigRepository;
            let (app_config, resolve) = match config_repo.load_with_resolve(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(3);
                }
            };

            let parser = DispatchingParser::new();
            let resolver = DispatchingResolver::from_resolve_config(
                resolve.as_ref(),
                &config,
                &app_config.project.languages,
            );

            match analyze::analyze(
                &config,
                &config_repo,
                &FsSourceFileRepository,
                &parser,
                &resolver,
            ) {
                Ok(result) => {
                    let content = match format {
                        AnalyzeFormat::Terminal => format_analyze_terminal(&result),
                        AnalyzeFormat::Json => format_analyze_json(&result),
                        AnalyzeFormat::Dot => format_analyze_dot(&result),
                        AnalyzeFormat::Svg => format_svg(&result),
                    };

                    match output {
                        Some(path) => {
                            // NOTE: Refuse to overwrite existing files to prevent accidental data loss.
                            if std::path::Path::new(&path).exists() {
                                eprintln!(
                                    "Error: '{}' already exists. Remove it first if you want to overwrite.",
                                    path
                                );
                                std::process::exit(1);
                            }
                            match fs::write(&path, &content) {
                                Ok(_) => eprintln!("Written to '{}'", path),
                                Err(e) => {
                                    eprintln!("Error: failed to write '{}': {}", path, e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        None => print!("{}", content),
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(3);
                }
            }
        }
        Command::Init {
            common: _,
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

            let lang_detector = crate::infrastructure::parser::ExtensionLanguageDetector;
            let languages = init::detect_languages(&cwd, &lang_detector);
            println!("Detected languages: {}", languages.join(", "));

            // Detect Go module name from go.mod (if present)
            let go_module_name = if languages.iter().any(|l| l == "go") {
                detect_go_module_name(Path::new(&cwd))
            } else {
                None
            };

            // Detect Java/Kotlin module name from pom.xml or build.gradle (if present)
            let is_jvm = languages.iter().any(|l| l == "java" || l == "kotlin");
            let java_module_name = if is_jvm {
                detect_java_module_name(Path::new(&cwd))
            } else {
                None
            };

            println!("Scanning imports...");
            let parser = DispatchingParser::new();

            // JVM languages bypass depth-based scanning: package declarations define layers
            let analyses = if is_jvm {
                scan_jvm_project(Path::new(&cwd), &parser, java_module_name.as_deref())
            } else {
                scan_project(Path::new(&cwd), &parser, depth, go_module_name.as_deref())
            };

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

            // JVM: use **/layer/** globs (package-based, depth-independent).
            // Others: append /** to the scanned directory path.
            for layer in &mut layers {
                if is_jvm {
                    layer.paths = layer.paths.iter().map(|p| format!("**/{p}/**")).collect();
                } else {
                    layer.paths = layer.paths.iter().map(|p| format!("{}/**", p)).collect();
                }
            }

            let resolve_generator =
                crate::infrastructure::resolve_config_generator::DefaultResolveConfigGenerator {
                    module_path_name: go_module_name,
                    package_prefix_name: java_module_name,
                };
            let toml_content =
                init::generate_toml(&project_name, ".", &languages, &layers, &resolve_generator);

            match std::fs::write(output_path, &toml_content) {
                Ok(_) => println!("\nGenerated '{}'", output),
                Err(e) => {
                    eprintln!("Error: failed to write '{}': {}", output, e);
                    std::process::exit(1);
                }
            }
        }
        Command::Check {
            common: _,
            config,
            format,
            fail_on,
        } => {
            // Pre-load config to build the resolver, then pass path to check().
            // NOTE: Double-load is acceptable for a CLI tool.
            let config_repo = TomlConfigRepository;
            let (app_config, resolve) = match config_repo.load_with_resolve(&config) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(3);
                }
            };

            let parser = DispatchingParser::new();
            let resolver = DispatchingResolver::from_resolve_config(
                resolve.as_ref(),
                &config,
                &app_config.project.languages,
            );

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

                    let should_fail = match fail_on {
                        // --fail-on warning: exit 1 for any violation (error or warning)
                        FailOn::Warning => result.violations.iter().any(|v| {
                            v.severity == Severity::Error || v.severity == Severity::Warning
                        }),
                        // --fail-on error (default): exit 1 only for errors
                        FailOn::Error => result
                            .violations
                            .iter()
                            .any(|v| v.severity == Severity::Error),
                    };
                    if should_fail {
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(3);
                }
            }
        }
        Command::Add {
            common,
            config,
            name,
            force,
            depth,
        } => {
            let target_path = &common.path;

            // 1. Config must exist
            if !Path::new(&config).exists() {
                eprintln!("Error: '{}' not found. Run 'mille init' first.", config);
                std::process::exit(3);
            }

            // 2. Target must be a directory
            if !Path::new(target_path).is_dir() {
                eprintln!("Error: '{}' is not a directory", target_path);
                std::process::exit(3);
            }

            // 3. Validate config is parseable
            let config_repo = TomlConfigRepository;
            if let Err(e) = config_repo.load_with_resolve(&config) {
                eprintln!("Error: failed to parse '{}': {}", config, e);
                std::process::exit(3);
            }

            // 4. Scan and build layers
            println!("Scanning '{}'...", target_path);
            let new_layers = if depth.is_some() {
                // --depth: scan subdirectories and produce multiple layers
                let target_abs = std::env::current_dir()
                    .expect("cannot determine current directory")
                    .join(target_path);
                let parser = DispatchingParser::new();
                let analyses = scan_target_with_depth(&target_abs, &parser, depth, target_path);
                let mut layers = init::infer_layers(&analyses);
                // Append /** glob to paths
                for layer in &mut layers {
                    layer.paths = layer.paths.iter().map(|p| format!("{}/**", p)).collect();
                }
                if layers.is_empty() {
                    println!("No layers detected under '{}'.", target_path);
                } else {
                    println!("Detected {} layers:", layers.len());
                    for layer in &layers {
                        if layer.allow.is_empty() {
                            println!("  {:<20} ← (no internal dependencies)", layer.name);
                        } else {
                            println!("  {:<20} → {}", layer.name, layer.allow.join(", "));
                        }
                    }
                }
                layers
            } else {
                // No depth: single layer
                let layer_name = name.unwrap_or_else(|| {
                    Path::new(target_path)
                        .file_name()
                        .unwrap_or(std::ffi::OsStr::new(target_path))
                        .to_string_lossy()
                        .to_string()
                });
                let target_glob = format!("{}/**", target_path);
                let analysis = scan_single_dir(Path::new(target_path));
                println!(
                    "  {} files, {} internal deps, {} external deps",
                    analysis.file_count,
                    analysis.internal_deps.len(),
                    analysis.external_pkgs.len()
                );
                vec![add_layer::build_layer_config(
                    &layer_name,
                    &target_glob,
                    &analysis,
                )]
            };

            // 5. Add each layer (conflict check per layer)
            let mut added = 0usize;
            let mut replaced = 0usize;
            let mut skipped = 0usize;
            // Re-read config for each pass to reflect prior writes
            for new_layer in &new_layers {
                let target_glob = new_layer.paths.first().cloned().unwrap_or_default();
                // Reload config to get current layers (may have changed in prior iteration)
                let current_config = match TomlConfigRepository.load_with_resolve(&config) {
                    Ok((c, _)) => c,
                    Err(e) => {
                        eprintln!("Error: failed to parse '{}': {}", config, e);
                        std::process::exit(3);
                    }
                };

                if let Some(conflict) =
                    add_layer::find_conflict(&current_config.layers, &target_glob)
                {
                    if !force {
                        eprintln!(
                            "  Skipping '{}': overlaps with layer '{}'",
                            new_layer.name, conflict.layer_name
                        );
                        skipped += 1;
                        continue;
                    }
                    // --force: replace via toml::Table
                    let raw = match fs::read_to_string(&config) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Error: failed to read '{}': {}", config, e);
                            std::process::exit(1);
                        }
                    };
                    let mut table: toml::Table = match raw.parse() {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("Error: failed to parse '{}': {}", config, e);
                            std::process::exit(3);
                        }
                    };
                    if let Err(e) = add_layer::replace_layer_in_table(
                        &mut table,
                        conflict.layer_index,
                        new_layer,
                    ) {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                    let content = toml::to_string_pretty(&table).unwrap_or_default();
                    if let Err(e) = fs::write(&config, &content) {
                        eprintln!("Error: failed to write '{}': {}", config, e);
                        std::process::exit(1);
                    }
                    replaced += 1;
                } else {
                    // Append
                    let layer_str = add_layer::layer_to_toml_string(new_layer);
                    let mut existing = match fs::read_to_string(&config) {
                        Ok(c) => c,
                        Err(e) => {
                            eprintln!("Error: failed to read '{}': {}", config, e);
                            std::process::exit(1);
                        }
                    };
                    existing.push_str(&layer_str);
                    if let Err(e) = fs::write(&config, &existing) {
                        eprintln!("Error: failed to write '{}': {}", config, e);
                        std::process::exit(1);
                    }
                    added += 1;
                }
            }

            // Summary
            if new_layers.len() == 1 && added == 1 {
                println!("Added layer '{}' to '{}'", new_layers[0].name, config);
            } else if new_layers.len() == 1 && replaced == 1 {
                println!("Replaced layer '{}' in '{}'", new_layers[0].name, config);
            } else {
                let mut parts = vec![];
                if added > 0 {
                    parts.push(format!("{} added", added));
                }
                if replaced > 0 {
                    parts.push(format!("{} replaced", replaced));
                }
                if skipped > 0 {
                    parts.push(format!("{} skipped", skipped));
                }
                println!("{} in '{}'", parts.join(", "), config);
            }

            if skipped > 0 && !force {
                std::process::exit(1);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Single-directory scanning for `mille add`
// ---------------------------------------------------------------------------

/// Scan a target directory with depth-based layer detection.
///
/// Works like `scan_project` but scoped to a subdirectory. Paths in the returned
/// `BTreeMap` are relative to the project root (prefixed with `target_rel`).
fn scan_target_with_depth(
    target_abs: &Path,
    parser: &DispatchingParser,
    depth: Option<usize>,
    target_rel: &str,
) -> BTreeMap<String, DirAnalysis> {
    // Collect source dirs relative to target_abs
    let mut source_dirs: BTreeSet<String> = BTreeSet::new();
    collect_source_dirs(target_abs, target_abs, &mut source_dirs);

    if source_dirs.is_empty() {
        return BTreeMap::new();
    }

    let target_depth = depth.unwrap_or_else(|| auto_detect_layer_depth(&source_dirs));

    // Compute layer dirs at the target depth, then prefix with target_rel
    let layer_dirs: BTreeSet<String> = source_dirs
        .iter()
        .filter_map(|d| ancestor_at_depth(d, target_depth))
        .collect();

    // Build analyses keyed by project-relative paths (target_rel/layer_dir)
    let mut analyses: BTreeMap<String, DirAnalysis> = BTreeMap::new();
    for dir in &layer_dirs {
        let project_rel = format!("{}/{}", target_rel, dir);
        analyses.insert(project_rel, DirAnalysis::default());
    }

    // Scan files, mapping them to the correct layer
    scan_target_imports(
        target_abs,
        target_abs,
        parser,
        target_depth,
        target_rel,
        &mut analyses,
    );

    analyses
}

fn scan_target_imports(
    root: &Path,
    dir: &Path,
    parser: &DispatchingParser,
    target_depth: usize,
    target_rel: &str,
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
            scan_target_imports(
                root,
                &path,
                parser,
                target_depth,
                target_rel,
                analyses,
            );
        } else if is_source_file(&name) {
            let Ok(source) = fs::read_to_string(&path) else {
                continue;
            };
            let file_str = path.to_string_lossy().to_string();

            let dir_rel = path
                .parent()
                .and_then(|p| p.strip_prefix(root).ok())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            // Roll up to target depth
            let layer_dir = if dir_rel.is_empty() {
                continue;
            } else {
                ancestor_at_depth(&dir_rel, target_depth).unwrap_or(dir_rel)
            };

            let project_key = format!("{}/{}", target_rel, layer_dir);
            let imports = SourceParser::parse_imports(parser, &source, &file_str);
            let analysis = analyses.entry(project_key).or_default();
            analysis.file_count += 1;

            for imp in &imports {
                if imp.kind == ImportKind::Mod {
                    continue;
                }
                match classify_import_for_init(&imp.path, &file_str, None) {
                    Some(InitImport::Internal(seg)) => {
                        analysis.internal_deps.insert(seg);
                    }
                    Some(InitImport::External(pkg)) => {
                        analysis.external_pkgs.insert(pkg);
                    }
                    Some(InitImport::TryInternal(seg)) => {
                        analysis.internal_deps.insert(seg);
                    }
                    None => {}
                }
            }
        }
    }
}

/// Scan a single directory (recursively) and build a DirAnalysis.
///
/// This is a simplified version of `scan_project` that treats the target directory
/// as a single layer. Internal deps are recorded as top-level import segments
/// (e.g. `crate::domain::...` → "domain"), and external packages are recorded as-is.
fn scan_single_dir(target: &Path) -> DirAnalysis {
    let parser = DispatchingParser::new();
    let mut analysis = DirAnalysis::default();
    scan_dir_recursive(target, &parser, &mut analysis);
    analysis
}

fn scan_dir_recursive(dir: &Path, parser: &DispatchingParser, analysis: &mut DirAnalysis) {
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
            scan_dir_recursive(&path, parser, analysis);
        } else if is_source_file(&name) {
            let Ok(source) = fs::read_to_string(&path) else {
                continue;
            };
            let file_str = path.to_string_lossy().to_string();
            let imports = SourceParser::parse_imports(parser, &source, &file_str);
            analysis.file_count += 1;

            for imp in &imports {
                if imp.kind == ImportKind::Mod {
                    continue;
                }
                match classify_import_for_init(&imp.path, &file_str, None) {
                    Some(InitImport::Internal(seg)) => {
                        analysis.internal_deps.insert(seg);
                    }
                    Some(InitImport::External(pkg)) => {
                        analysis.external_pkgs.insert(pkg);
                    }
                    Some(InitImport::TryInternal(seg)) => {
                        analysis.internal_deps.insert(seg);
                    }
                    None => {}
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Go module detection
// ---------------------------------------------------------------------------

/// Read `go.mod` in `root` and return the module path declared on the `module` line.
/// Returns `None` if `go.mod` is missing or no `module` line is found.
fn detect_go_module_name(root: &Path) -> Option<String> {
    let content = fs::read_to_string(root.join("go.mod")).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix("module ") {
            let mn = rest.trim().to_string();
            if !mn.is_empty() {
                return Some(mn);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Java/Kotlin module detection
// ---------------------------------------------------------------------------

/// Detect the Java module name from `pom.xml` (Maven) or `build.gradle` + `settings.gradle` (Gradle).
/// Returns `None` if neither file is present or parseable.
fn detect_java_module_name(root: &Path) -> Option<String> {
    use crate::infrastructure::resolver::java::{read_module_from_gradle, read_module_from_pom};
    let pom = root.join("pom.xml");
    if pom.exists() {
        if let Some(name) = read_module_from_pom(pom.to_str()?) {
            return Some(name);
        }
    }
    let gradle = root.join("build.gradle");
    if gradle.exists() {
        if let Some(name) = read_module_from_gradle(gradle.to_str()?, None) {
            return Some(name);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// JVM project scanning — package-declaration-based, depth-independent
// ---------------------------------------------------------------------------

/// Scan a JVM project (Java/Kotlin) and build `DirAnalysis` keyed by layer name.
///
/// Unlike the depth-based `scan_project`, this function uses the `package` declaration
/// in each source file to determine which layer it belongs to. This makes it independent
/// of the directory depth (e.g., Maven's `src/main/java/com/example/myapp/domain/` is
/// handled the same as a flat `src/domain/`).
///
/// Layer key = first package segment after `module_name` prefix.
/// e.g. `package com.example.myapp.domain;` with module `com.example.myapp` → key `"domain"`.
fn scan_jvm_project(
    root: &Path,
    parser: &DispatchingParser,
    module_name: Option<&str>,
) -> BTreeMap<String, DirAnalysis> {
    let mut analyses: BTreeMap<String, DirAnalysis> = BTreeMap::new();
    scan_jvm_dir(root, root, parser, module_name, &mut analyses);
    analyses
}

fn scan_jvm_dir(
    root: &Path,
    dir: &Path,
    parser: &DispatchingParser,
    module_name: Option<&str>,
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
            scan_jvm_dir(root, &path, parser, module_name, analyses);
        } else if name.ends_with(".java") || name.ends_with(".kt") {
            process_jvm_file(root, &path, parser, module_name, analyses);
        }
    }
}

fn process_jvm_file(
    root: &Path,
    file: &Path,
    parser: &DispatchingParser,
    module_name: Option<&str>,
    analyses: &mut BTreeMap<String, DirAnalysis>,
) {
    let Ok(source) = fs::read_to_string(file) else {
        return;
    };
    let file_rel = file.strip_prefix(root).unwrap_or(file);
    let file_rel_str = file_rel.to_string_lossy().to_string();

    // Determine layer from the package declaration in the source file.
    let Some(pkg) = extract_jvm_package(&source) else {
        return;
    };
    let Some(layer_key) = package_to_layer(&pkg, module_name) else {
        return;
    };

    // If module_name was not provided, derive it from this file's own package.
    // e.g. package="com.example.myapp.usecase", layer="usecase" → module="com.example.myapp"
    let derived_module: Option<String> = if module_name.map(|m| !m.is_empty()).unwrap_or(false) {
        None
    } else {
        let suffix = format!(".{}", layer_key);
        pkg.strip_suffix(&suffix).map(|s| s.to_string())
    };
    let effective_module: Option<&str> = module_name
        .filter(|m| !m.is_empty())
        .or(derived_module.as_deref());

    let imports = SourceParser::parse_imports(parser, &source, &file_rel_str);
    let analysis = analyses.entry(layer_key.clone()).or_default();
    analysis.file_count += 1;

    for imp in &imports {
        if imp.kind == ImportKind::Mod {
            continue;
        }
        match classify_java_import_for_init(&imp.path, effective_module) {
            Some(InitImport::Internal(seg)) | Some(InitImport::TryInternal(seg)) => {
                if seg != layer_key && !seg.is_empty() {
                    analysis.internal_deps.insert(seg);
                }
            }
            Some(InitImport::External(pkg)) => {
                analysis.external_pkgs.insert(pkg);
            }
            None => {}
        }
    }
}

/// Extract the package name from a Java or Kotlin source file.
///
/// Scans lines until the first `package` declaration is found.
/// Java: `package com.example.myapp.domain;`
/// Kotlin: `package com.example.myapp.domain`
fn extract_jvm_package(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("package ") {
            let pkg = rest.trim_end_matches(';').trim().to_string();
            if !pkg.is_empty() {
                return Some(pkg);
            }
        }
    }
    None
}

/// Derive the layer key from a package name given the project's module name.
///
/// - `com.example.myapp.domain` with module `com.example.myapp` → `"domain"`
/// - `com.example.myapp` (root package) → `None`
/// - Without module_name: uses the last segment heuristically
fn package_to_layer(package: &str, module_name: Option<&str>) -> Option<String> {
    if let Some(mn) = module_name.filter(|m| !m.is_empty()) {
        let prefix = format!("{}.", mn);
        let rest = package.strip_prefix(&prefix)?;
        let seg = rest.split('.').next()?.to_string();
        if seg.is_empty() {
            return None;
        }
        Some(seg)
    } else {
        // Heuristic: use the last segment of the package as the layer name.
        // e.g. "com.example.domain" → "domain"
        package.split('.').next_back().map(|s| s.to_string())
    }
}

/// Classify a Java/Kotlin import path for `mille init` layer inference.
fn classify_java_import_for_init(path: &str, module_name: Option<&str>) -> Option<InitImport> {
    // Java/javax stdlib — record as external so they appear in external_allow.
    // Using the full dotted path (e.g. "java.util.List") so external_allow matches exactly.
    if path.starts_with("java.")
        || path.starts_with("javax.")
        || path.starts_with("sun.")
        || path.starts_with("com.sun.")
    {
        return Some(InitImport::External(path.to_string()));
    }

    if let Some(mn) = module_name.filter(|m| !m.is_empty()) {
        let prefix = format!("{}.", mn);
        if path.starts_with(&prefix) || path == mn {
            let rest = path.strip_prefix(&prefix).unwrap_or("");
            let seg = rest.split('.').next()?.to_string();
            if seg.is_empty() {
                return None;
            }
            return Some(InitImport::TryInternal(seg));
        }
        // Has module_name context — anything else is external
        let pkg = path.split('.').next()?.to_string();
        return Some(InitImport::External(pkg));
    }

    // No module_name: heuristic — treat first segment as potentially internal
    let seg = path.split('.').next()?.to_string();
    if seg.is_empty() {
        return None;
    }
    Some(InitImport::TryInternal(seg))
}

// ---------------------------------------------------------------------------
// Project scanning — builds DirAnalysis per source directory
// ---------------------------------------------------------------------------

fn scan_project(
    root: &Path,
    parser: &DispatchingParser,
    depth: Option<usize>,
    go_module_name: Option<&str>,
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
    collect_dir_imports(
        root,
        root,
        parser,
        &layer_dirs,
        target_depth,
        &mut analyses,
        go_module_name,
    );

    analyses
}

fn is_source_file(name: &str) -> bool {
    matches!(
        name.rsplit('.').next().unwrap_or(""),
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "go"
            | "py"
            | "java"
            | "kt"
            | "php"
            | "c"
            | "h"
            | "yaml"
            | "yml"
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
    go_module_name: Option<&str>,
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
            collect_dir_imports(
                root,
                &path,
                parser,
                layer_dirs,
                target_depth,
                analyses,
                go_module_name,
            );
        } else if is_source_file(&name) {
            process_source_file(
                root,
                &path,
                parser,
                layer_dirs,
                target_depth,
                analyses,
                go_module_name,
            );
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
    go_module_name: Option<&str>,
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

    // Roll up the immediate dir to the target layer depth.
    // NOTE: files shallower than target_depth (e.g. src/main.py when depth=2)
    // use dir_rel itself as the layer dir so they are not silently dropped.
    let layer_dir = ancestor_at_depth(&dir_rel, target_depth).unwrap_or_else(|| dir_rel.clone());

    let imports = SourceParser::parse_imports(parser, &source, &file_rel_str);
    let analysis = analyses.entry(layer_dir.clone()).or_default();
    analysis.file_count += 1;

    for imp in &imports {
        // Skip Rust submodule declarations (mod foo;) — they are not cross-layer imports
        if imp.kind == ImportKind::Mod {
            continue;
        }
        match classify_import_for_init(&imp.path, &file_rel_str, go_module_name) {
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

#[derive(Debug)]
enum InitImport {
    /// Definitely internal (e.g. Rust `crate::X`): don't add as external if dir not found.
    Internal(String),
    /// Definitely external (e.g. Rust `serde::X`): always add to external_pkgs.
    External(String),
    /// Ambiguous (e.g. Go `pkg/X`, Python `X.Y`): try internal match first, fall back to external.
    TryInternal(String),
}

fn classify_import_for_init(
    path: &str,
    file_path: &str,
    go_module_name: Option<&str>,
) -> Option<InitImport> {
    if file_path.ends_with(".rs") {
        classify_rust_import(path)
    } else if file_path.ends_with(".ts")
        || file_path.ends_with(".tsx")
        || file_path.ends_with(".js")
        || file_path.ends_with(".jsx")
    {
        classify_ts_import(path, file_path)
    } else if file_path.ends_with(".go") {
        classify_go_import(path, go_module_name)
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

fn classify_go_import(path: &str, module_name: Option<&str>) -> Option<InitImport> {
    let first = path.split('/').next()?;

    // Internal: path starts with the project's module name (from go.mod)
    if let Some(mn) = module_name.filter(|m| !m.is_empty()) {
        if path == mn || path.starts_with(&format!("{}/", mn)) {
            // Strip module prefix and take the first remaining segment as the layer dir
            let rel = path.strip_prefix(&format!("{}/", mn)).unwrap_or(path);
            let seg = rel.split('/').next()?.to_string();
            return Some(InitImport::TryInternal(seg));
        }
        // We know the module name — anything else (including stdlib) is external.
        // Record with the full import path so external_allow can match exactly.
        let _ = first; // suppress unused-variable warning
        return Some(InitImport::External(path.to_string()));
    }

    // module_name not known: use heuristic.
    // No dot in first segment → likely stdlib; use full path so it appears in external_allow.
    if !first.contains('.') {
        return Some(InitImport::External(path.to_string()));
    }
    // External with dot in first segment: use full path for accurate matching.
    Some(InitImport::External(path.to_string()))
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
    // Absolute import — return full dotted path so resolve_to_known_dir can
    // try all slash-prefixes (e.g. "src.domain.entity" → try "src", "src/domain").
    if path.is_empty() {
        return None;
    }
    Some(InitImport::TryInternal(path.to_string()))
}

/// Return the ancestor of `dir` at `depth` path segments, or `None` if `dir` is shallower.
///
/// * `"src/domain/entity", 2` → `Some("src/domain")`
/// * `"src/domain", 2`        → `Some("src/domain")`
/// * `"src", 2`               → `None`
pub(crate) fn ancestor_at_depth(dir: &str, depth: usize) -> Option<String> {
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
pub(crate) fn auto_detect_layer_depth(all_source_dirs: &BTreeSet<String>) -> usize {
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

/// Find a known directory that matches a module path (dotted or slash).
///
/// Accepts a dotted Python path (`"src.domain.entity"`), a plain single
/// segment (`"domain"`), or a Rust/Go sub-segment (`"domain"`).
///
/// Strategy (in priority order):
/// 1. Slash-prefix exact match, sibling-first  — handles Python src-layout
///    e.g. `"src.domain.entity"` → tries `"src"`, `"src/domain"` → `"src/domain"`
/// 2. Slash-prefix exact match, any dir
/// 3. Base-name match, sibling-first           — handles Rust `crate::domain`
///    where the known dir is `"src/domain"` but the segment is just `"domain"`
/// 4. Base-name match, any dir
///
/// NOTE: steps 1–2 use full-path equality so a dotted path can't accidentally
/// match a dir in a completely different subtree.
fn resolve_to_known_dir(
    module_path: &str,
    current_dir: &str,
    known_dirs: &BTreeSet<String>,
) -> Option<String> {
    let current_parent = current_dir.rsplit_once('/').map(|(p, _)| p).unwrap_or("");

    // Build slash-prefixes from dotted path:
    // "src.domain.entity" → ["src", "src/domain", "src/domain/entity"]
    let segments: Vec<&str> = module_path.split('.').collect();
    let slash_prefixes: Vec<String> = (1..=segments.len())
        .map(|n| segments[..n].join("/"))
        .collect();

    // Strategy 1: prefix exact-match, sibling-first
    for prefix in &slash_prefixes {
        for dir in known_dirs {
            if dir == prefix && dir.as_str() != current_dir {
                let parent = dir.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
                if parent == current_parent {
                    return Some(dir.clone());
                }
            }
        }
    }

    // Strategy 2: prefix exact-match, any dir
    for prefix in &slash_prefixes {
        for dir in known_dirs {
            if dir == prefix && dir.as_str() != current_dir {
                return Some(dir.clone());
            }
        }
    }

    // Strategy 3 & 4: base-name match (for Rust/Go single segments like "domain"
    // that need to resolve to "src/domain").
    // NOTE: only the last segment of module_path is used as the base name.
    let base_seg = segments.last().copied().unwrap_or(module_path);

    // Strategy 3: base-name, sibling-first
    for dir in known_dirs {
        let base = dir.rsplit('/').next().unwrap_or(dir.as_str());
        if base == base_seg && dir.as_str() != current_dir {
            let parent = dir.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
            if parent == current_parent {
                return Some(dir.clone());
            }
        }
    }

    // Strategy 4: base-name, any dir
    for dir in known_dirs {
        let base = dir.rsplit('/').next().unwrap_or(dir.as_str());
        if base == base_seg && dir.as_str() != current_dir {
            return Some(dir.clone());
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Analyze output formatters
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Report external formatters
// ---------------------------------------------------------------------------

fn format_report_external_terminal(result: &report_external::ReportExternalResult) -> String {
    let mut buf = String::from("External Dependencies by Layer\n\n");
    if result.layers.is_empty() {
        buf.push_str("  (no layers configured)\n");
        return buf;
    }
    let max_name = result
        .layers
        .iter()
        .map(|l| l.layer_name.len())
        .max()
        .unwrap_or(0);
    for layer in &result.layers {
        let pkgs = if layer.packages.is_empty() {
            "(none)".to_string()
        } else {
            layer.packages.join(", ")
        };
        buf.push_str(&format!(
            "  {:<width$}  {}\n",
            layer.layer_name,
            pkgs,
            width = max_name,
        ));
    }
    buf
}

fn format_report_external_json(result: &report_external::ReportExternalResult) -> String {
    let entries: Vec<String> = result
        .layers
        .iter()
        .map(|l| {
            let pkgs: Vec<String> = l.packages.iter().map(|p| format!("\"{}\"", p)).collect();
            format!(
                "{{\"layer\":\"{}\",\"packages\":[{}]}}",
                l.layer_name,
                pkgs.join(",")
            )
        })
        .collect();
    format!("[{}]\n", entries.join(","))
}

fn format_analyze_terminal(result: &analyze::AnalyzeResult) -> String {
    let mut buf = String::new();
    buf.push_str(&format!(
        "Dependency Graph ({} layers)\n\n",
        result.nodes.len()
    ));
    if result.edges.is_empty() {
        buf.push_str("  (no cross-layer dependencies detected)\n");
    } else {
        let max_from = result.edges.iter().map(|e| e.from.len()).max().unwrap_or(0);
        let max_to = result.edges.iter().map(|e| e.to.len()).max().unwrap_or(0);
        for edge in &result.edges {
            buf.push_str(&format!(
                "  {:<from_w$} -> {:<to_w$}  ({})\n",
                edge.from,
                edge.to,
                edge.import_count,
                from_w = max_from,
                to_w = max_to,
            ));
        }
    }
    buf.push('\n');
    // Layer summary table
    let max_name = result.nodes.iter().map(|n| n.name.len()).max().unwrap_or(0);
    for node in &result.nodes {
        buf.push_str(&format!(
            "  {:<name_w$}  {} file{}\n",
            node.name,
            node.file_count,
            if node.file_count == 1 { "" } else { "s" },
            name_w = max_name,
        ));
    }
    buf
}

fn format_analyze_json(result: &analyze::AnalyzeResult) -> String {
    let nodes: Vec<String> = result
        .nodes
        .iter()
        .map(|n| format!(r#"{{"layer":"{}","file_count":{}}}"#, n.name, n.file_count))
        .collect();
    let edges: Vec<String> = result
        .edges
        .iter()
        .map(|e| {
            format!(
                r#"{{"from":"{}","to":"{}","import_count":{}}}"#,
                e.from, e.to, e.import_count
            )
        })
        .collect();
    format!(
        r#"{{"nodes":[{}],"edges":[{}]}}"#,
        nodes.join(","),
        edges.join(",")
    )
}

fn format_analyze_dot(result: &analyze::AnalyzeResult) -> String {
    let mut buf = String::from("digraph mille {\n  rankdir=TB;\n");
    for node in &result.nodes {
        let label = format!(
            "{}\\n{} file{}",
            node.name,
            node.file_count,
            if node.file_count == 1 { "" } else { "s" }
        );
        buf.push_str(&format!("  \"{}\" [label=\"{}\"];\n", node.name, label));
    }
    for edge in &result.edges {
        buf.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            edge.from, edge.to, edge.import_count
        ));
    }
    buf.push_str("}\n");
    buf
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

    // ------------------------------------------------------------------
    // classify_go_import
    // ------------------------------------------------------------------

    #[test]
    fn test_classify_go_import_internal_with_module_name() {
        // When module_name is known, matching imports → TryInternal with first sub-segment
        let result = classify_go_import(
            "github.com/example/myapp/domain",
            Some("github.com/example/myapp"),
        );
        match result {
            Some(InitImport::TryInternal(seg)) => {
                assert_eq!(seg, "domain", "first sub-segment after module prefix");
            }
            other => panic!("expected TryInternal(\"domain\"), got {:?}", other),
        }
    }

    #[test]
    fn test_classify_go_import_stdlib_with_module_name_is_external() {
        // When module_name is known, stdlib (no dot, no module match) → External with full path
        let result = classify_go_import("fmt", Some("github.com/example/myapp"));
        match result {
            Some(InitImport::External(pkg)) => {
                assert_eq!(pkg, "fmt");
            }
            other => panic!("expected External(\"fmt\"), got {:?}", other),
        }
    }

    #[test]
    fn test_classify_go_import_external_full_path_with_module_name() {
        // External package → full path stored (not last segment)
        let result = classify_go_import("github.com/cilium/ebpf", Some("github.com/example/myapp"));
        match result {
            Some(InitImport::External(pkg)) => {
                assert_eq!(
                    pkg, "github.com/cilium/ebpf",
                    "full path must be stored for accurate matching"
                );
            }
            other => panic!(
                "expected External(\"github.com/cilium/ebpf\"), got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_classify_go_import_no_module_name_stdlib_is_external() {
        // Without module_name, stdlib-like packages (no dot) → External with full path
        let result = classify_go_import("fmt", None);
        match result {
            Some(InitImport::External(pkg)) => {
                assert_eq!(pkg, "fmt");
            }
            other => panic!("expected External(\"fmt\"), got {:?}", other),
        }
    }

    #[test]
    fn test_classify_go_import_no_module_name_external_full_path() {
        // Without module_name, external packages → External with full path
        let result = classify_go_import("github.com/cilium/ebpf", None);
        match result {
            Some(InitImport::External(pkg)) => {
                assert_eq!(pkg, "github.com/cilium/ebpf");
            }
            other => panic!(
                "expected External(\"github.com/cilium/ebpf\"), got {:?}",
                other
            ),
        }
    }

    // ------------------------------------------------------------------
    // detect_go_module_name
    // ------------------------------------------------------------------

    #[test]
    fn test_detect_go_module_name_from_go_mod() {
        let tmp = std::env::temp_dir().join(format!("mille_go_mod_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(
            tmp.join("go.mod"),
            "module github.com/example/myapp\n\ngo 1.21\n",
        )
        .unwrap();

        let result = detect_go_module_name(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);

        assert_eq!(
            result,
            Some("github.com/example/myapp".to_string()),
            "module name should be extracted from go.mod"
        );
    }

    #[test]
    fn test_detect_go_module_name_missing_file_returns_none() {
        let tmp = std::env::temp_dir().join(format!("mille_go_mod_missing_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        // No go.mod created

        let result = detect_go_module_name(&tmp);
        let _ = std::fs::remove_dir_all(&tmp);

        assert!(result.is_none(), "missing go.mod should return None");
    }

    // ------------------------------------------------------------------
    // Bug 1: classify_py_import — absolute imports should return full dotted path
    // ------------------------------------------------------------------

    #[test]
    fn test_classify_py_import_returns_full_path() {
        // Absolute imports must return the full dotted path so
        // resolve_to_known_dir can try all slash-prefixes.
        match classify_py_import("src.domain.entity") {
            Some(InitImport::TryInternal(seg)) => {
                assert_eq!(seg, "src.domain.entity");
            }
            other => panic!(
                "expected TryInternal(\"src.domain.entity\"), got {:?}",
                other
            ),
        }
        match classify_py_import("domain.entity") {
            Some(InitImport::TryInternal(seg)) => {
                assert_eq!(seg, "domain.entity");
            }
            other => panic!("expected TryInternal(\"domain.entity\"), got {:?}", other),
        }
        // Relative imports (.domain) should still return single segment
        match classify_py_import(".domain") {
            Some(InitImport::TryInternal(seg)) => {
                assert_eq!(seg, "domain");
            }
            other => panic!(
                "expected TryInternal(\"domain\") for relative, got {:?}",
                other
            ),
        }
    }

    // ------------------------------------------------------------------
    // Bug 1: resolve_to_known_dir — dotted namespace paths
    // ------------------------------------------------------------------

    #[test]
    fn test_resolve_to_known_dir_dotted_namespace() {
        let dirs: BTreeSet<String> = ["src/domain", "src/infrastructure"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        // "src.domain.entity" from "src/infrastructure" → "src/domain"
        assert_eq!(
            resolve_to_known_dir("src.domain.entity", "src/infrastructure", &dirs),
            Some("src/domain".to_string()),
            "dotted path prefix src/domain should match layer dir"
        );

        // "src.infrastructure.db" from "src/domain" → "src/infrastructure"
        assert_eq!(
            resolve_to_known_dir("src.infrastructure.db", "src/domain", &dirs),
            Some("src/infrastructure".to_string()),
        );

        // "src.unknown.thing" → no match
        assert_eq!(
            resolve_to_known_dir("src.unknown.thing", "src/domain", &dirs),
            None,
            "unknown sub-package should return None"
        );
    }

    #[test]
    fn test_resolve_to_known_dir_flat_import_still_works() {
        // Flat layout (no src/ prefix): "domain.entity" from "infrastructure" → "domain"
        let dirs: BTreeSet<String> = ["domain", "infrastructure"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        assert_eq!(
            resolve_to_known_dir("domain.entity", "infrastructure", &dirs),
            Some("domain".to_string()),
            "flat layout backward compatibility must be preserved"
        );
    }

    // ------------------------------------------------------------------
    // Bug 2: ancestor_at_depth regression guard
    // ------------------------------------------------------------------

    #[test]
    fn test_ancestor_at_depth_shallower_returns_none() {
        // "src" at depth=2 → None (verified regression: Bug 2 was using this as skip)
        assert_eq!(ancestor_at_depth("src", 2), None);
    }

    // ------------------------------------------------------------------
    // Bug 2: scan_project must include files shallower than target_depth
    // ------------------------------------------------------------------

    #[test]
    fn test_scan_main_py_creates_layer() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();

        // src/main.py — shallower than target_depth=2
        let src_dir = root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.py"), "# no imports\n").unwrap();

        // src/domain/entity.py — at target_depth=2
        let domain_dir = src_dir.join("domain");
        std::fs::create_dir_all(&domain_dir).unwrap();
        std::fs::write(domain_dir.join("entity.py"), "# no imports\n").unwrap();

        // src/infrastructure/repo.py — at target_depth=2
        let infra_dir = src_dir.join("infrastructure");
        std::fs::create_dir_all(&infra_dir).unwrap();
        std::fs::write(infra_dir.join("repo.py"), "# no imports\n").unwrap();

        let parser = crate::infrastructure::parser::DispatchingParser::new();
        let analyses = scan_project(root, &parser, None, None);

        assert!(
            analyses.contains_key("src"),
            "analyses must contain a 'src' layer for src/main.py; got keys: {:?}",
            analyses.keys().collect::<Vec<_>>()
        );
        assert!(
            analyses["src"].file_count >= 1,
            "src layer file_count should be >= 1"
        );
    }
}
