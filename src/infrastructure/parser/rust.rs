use tree_sitter::Node;

use crate::domain::entity::call_expr::RawCallExpr;
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

/// Parse Rust source code and extract static call expressions (`Type::method()`).
/// Instance method calls (`var.method()`) are extracted with `receiver_type = None`
/// because their type cannot be determined without type inference.
pub fn parse_rust_call_exprs(source: &str, file_path: &str) -> Vec<RawCallExpr> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_rust::language())
        .expect("Failed to load Rust grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut calls = Vec::new();
    collect_call_exprs(root, source.as_bytes(), file_path, &mut calls);
    calls
}

fn collect_call_exprs(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawCallExpr>) {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            let line = node.start_position().row + 1;
            match func.kind() {
                "scoped_identifier" => {
                    // Static call: Foo::method() or some::path::Foo::method()
                    if let Some(name_node) = func.child_by_field_name("name") {
                        let method = name_node.utf8_text(source).unwrap_or("").to_string();
                        let receiver_type = root_type_of_scoped_id(&func, source);
                        if !method.is_empty() && !receiver_type.is_empty() {
                            out.push(RawCallExpr {
                                file: file_path.to_string(),
                                line,
                                receiver_type: Some(receiver_type),
                                method,
                            });
                        }
                    }
                }
                "field_expression" => {
                    // Instance call: var.method()
                    if let Some(field) = func.child_by_field_name("field") {
                        let method = field.utf8_text(source).unwrap_or("").to_string();
                        if !method.is_empty() {
                            out.push(RawCallExpr {
                                file: file_path.to_string(),
                                line,
                                receiver_type: None,
                                method,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_call_exprs(child, source, file_path, out);
        }
    }
}

/// Walk `scoped_identifier` path chain to return the leftmost (root) type name.
/// For `UserRepo::new` → "UserRepo"; for `infra::Repo::new` → "infra".
fn root_type_of_scoped_id(node: &Node, source: &[u8]) -> String {
    let Some(path) = node.child_by_field_name("path") else {
        return String::new();
    };
    match path.kind() {
        "identifier" | "type_identifier" => path.utf8_text(source).unwrap_or("").to_string(),
        "scoped_identifier" => root_type_of_scoped_id(&path, source),
        _ => String::new(),
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

    // ------------------------------------------------------------------
    // parse_rust_call_exprs
    // ------------------------------------------------------------------

    #[test]
    fn test_static_call_new() {
        let source = "fn main() { let r = Repo::new(); }";
        let calls = parse_rust_call_exprs(source, "src/main.rs");
        let found = calls
            .iter()
            .find(|c| c.receiver_type.as_deref() == Some("Repo") && c.method == "new");
        assert!(
            found.is_some(),
            "should detect Repo::new() as a static call, got: {:#?}",
            calls
        );
    }

    #[test]
    fn test_static_call_other_method() {
        let source = "fn f() { UserRepo::find_user(1); }";
        let calls = parse_rust_call_exprs(source, "src/main.rs");
        let found = calls
            .iter()
            .find(|c| c.receiver_type.as_deref() == Some("UserRepo") && c.method == "find_user");
        assert!(
            found.is_some(),
            "should detect UserRepo::find_user() as a static call, got: {:#?}",
            calls
        );
    }

    #[test]
    fn test_instance_call_has_no_receiver_type() {
        let source = "fn f() { repo.save(&user); }";
        let calls = parse_rust_call_exprs(source, "src/main.rs");
        let found = calls.iter().find(|c| c.method == "save");
        assert!(
            found.is_some(),
            "should detect repo.save() as a call, got: {:#?}",
            calls
        );
        assert_eq!(
            found.unwrap().receiver_type,
            None,
            "instance call receiver_type should be None"
        );
    }

    #[test]
    fn test_call_line_number() {
        let source = "fn f() {\n    Repo::new();\n}";
        let calls = parse_rust_call_exprs(source, "src/main.rs");
        let found = calls
            .iter()
            .find(|c| c.receiver_type.as_deref() == Some("Repo") && c.method == "new");
        assert!(found.is_some(), "should find Repo::new()");
        assert_eq!(found.unwrap().line, 2, "should be on line 2");
    }

    #[test]
    fn test_multiple_calls_in_file() {
        let source = "fn main() { let r = Repo::new(); let u = Usecase::new(r); r.execute(); }";
        let calls = parse_rust_call_exprs(source, "src/main.rs");
        let static_calls: Vec<_> = calls.iter().filter(|c| c.receiver_type.is_some()).collect();
        assert!(
            static_calls.len() >= 2,
            "should detect at least 2 static calls, got: {:#?}",
            calls
        );
    }

    #[test]
    fn test_dogfood_call_exprs_main_rs() {
        let source = std::fs::read_to_string("src/main.rs").expect("src/main.rs should exist");
        let calls = parse_rust_call_exprs(&source, "src/main.rs");
        // main.rs should have at least one call expression (Cli::parse())
        assert!(
            !calls.is_empty(),
            "main.rs should contain at least one call expression"
        );
    }
}
