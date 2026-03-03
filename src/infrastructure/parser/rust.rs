use tree_sitter::Node;

use crate::domain::entity::import::{ImportKind, RawImport};

/// Parse Rust source code and extract all `use` and external `mod` declarations.
pub fn parse_rust_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Failed to load Rust grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

fn collect_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    match node.kind() {
        "use_declaration" => {
            if let Some(arg) = node.child_by_field_name("argument") {
                let path = arg.utf8_text(source).unwrap_or("").to_string();
                out.push(RawImport {
                    path,
                    line: node.start_position().row + 1,
                    file: file_path.to_string(),
                    kind: ImportKind::Use,
                });
            }
        }
        "mod_item" => {
            // Only capture external mod declarations — those without an inline body.
            let has_body = (0..node.child_count())
                .filter_map(|i| node.child(i))
                .any(|c| c.kind() == "declaration_list");
            if !has_body {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let path = name_node.utf8_text(source).unwrap_or("").to_string();
                    out.push(RawImport {
                        path,
                        line: node.start_position().row + 1,
                        file: file_path.to_string(),
                        kind: ImportKind::Mod,
                    });
                }
            }
        }
        _ => {}
    }

    // Always recurse so nested declarations (e.g. inside inline mods) are also captured.
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_imports(child, source, file_path, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::ImportKind;

    #[test]
    fn test_simple_use_declaration() {
        let source = "use crate::domain::entity::config;";
        let imports = parse_rust_imports(source, "test.rs");

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "crate::domain::entity::config");
        assert_eq!(imports[0].kind, ImportKind::Use);
        assert_eq!(imports[0].line, 1);
        assert_eq!(imports[0].file, "test.rs");
    }

    #[test]
    fn test_pub_use_declaration() {
        let source = "pub use crate::domain::entity::config;";
        let imports = parse_rust_imports(source, "test.rs");

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "crate::domain::entity::config");
        assert_eq!(imports[0].kind, ImportKind::Use);
    }

    #[test]
    fn test_external_mod_declaration() {
        let source = "mod domain;";
        let imports = parse_rust_imports(source, "test.rs");

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "domain");
        assert_eq!(imports[0].kind, ImportKind::Mod);
        assert_eq!(imports[0].line, 1);
    }

    #[test]
    fn test_pub_mod_declaration() {
        let source = "pub mod domain;";
        let imports = parse_rust_imports(source, "test.rs");

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "domain");
        assert_eq!(imports[0].kind, ImportKind::Mod);
    }

    #[test]
    fn test_inline_mod_not_captured_as_import() {
        // Inline mod bodies define new scopes, not external file imports.
        let source = "pub mod domain { pub struct Foo; }";
        let imports = parse_rust_imports(source, "test.rs");

        assert!(
            imports.iter().all(|i| i.kind != ImportKind::Mod),
            "inline mod body should not be captured as a Mod import"
        );
    }

    #[test]
    fn test_grouped_use_declaration() {
        let source = "use crate::domain::{entity, repository};";
        let imports = parse_rust_imports(source, "test.rs");

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "crate::domain::{entity, repository}");
        assert_eq!(imports[0].kind, ImportKind::Use);
    }

    #[test]
    fn test_wildcard_use_declaration() {
        let source = "use super::*;";
        let imports = parse_rust_imports(source, "test.rs");

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "super::*");
        assert_eq!(imports[0].kind, ImportKind::Use);
    }

    #[test]
    fn test_multiple_declarations_line_numbers() {
        let source = "use std::io;\nuse crate::domain::entity::config::MilleConfig;\nmod tests;";
        let imports = parse_rust_imports(source, "test.rs");

        assert_eq!(imports.len(), 3);

        let use_imports: Vec<_> = imports
            .iter()
            .filter(|i| i.kind == ImportKind::Use)
            .collect();
        let mod_imports: Vec<_> = imports
            .iter()
            .filter(|i| i.kind == ImportKind::Mod)
            .collect();
        assert_eq!(use_imports.len(), 2);
        assert_eq!(mod_imports.len(), 1);

        assert_eq!(use_imports[0].line, 1);
        assert_eq!(use_imports[1].line, 2);
        assert_eq!(mod_imports[0].line, 3);
    }

    #[test]
    fn test_use_inside_inline_mod_is_captured() {
        // use declarations nested inside inline mods should still be extracted.
        let source = "mod tests {\n    use super::*;\n}";
        let imports = parse_rust_imports(source, "test.rs");

        let use_imports: Vec<_> = imports
            .iter()
            .filter(|i| i.kind == ImportKind::Use)
            .collect();
        assert_eq!(use_imports.len(), 1);
        assert_eq!(use_imports[0].path, "super::*");
    }

    // -----------------------------------------------------------------
    // Dogfooding: parse mille's own source files
    // -----------------------------------------------------------------

    #[test]
    fn test_dogfood_main_rs() {
        let source = std::fs::read_to_string("src/main.rs").expect("src/main.rs should exist");
        let imports = parse_rust_imports(&source, "src/main.rs");

        let mod_names: Vec<&str> = imports
            .iter()
            .filter(|i| i.kind == ImportKind::Mod)
            .map(|i| i.path.as_str())
            .collect();

        assert!(
            mod_names.contains(&"domain"),
            "`pub mod domain` should be detected in main.rs, got: {:?}",
            mod_names
        );
        assert!(
            mod_names.contains(&"infrastructure"),
            "`pub mod infrastructure` should be detected in main.rs, got: {:?}",
            mod_names
        );
    }

    #[test]
    fn test_dogfood_toml_config_repository() {
        let source =
            std::fs::read_to_string("src/infrastructure/repository/toml_config_repository.rs")
                .expect("toml_config_repository.rs should exist");
        let imports = parse_rust_imports(
            &source,
            "src/infrastructure/repository/toml_config_repository.rs",
        );

        let use_paths: Vec<&str> = imports
            .iter()
            .filter(|i| i.kind == ImportKind::Use)
            .map(|i| i.path.as_str())
            .collect();

        assert!(
            use_paths.iter().any(|p| p.contains("MilleConfig")),
            "should detect `use crate::domain::entity::config::MilleConfig`, got: {:?}",
            use_paths
        );
        assert!(
            use_paths.iter().any(|p| p.contains("ConfigRepository")),
            "should detect `use crate::domain::repository::config_repository::ConfigRepository`, got: {:?}",
            use_paths
        );
    }
}
