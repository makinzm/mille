use tree_sitter::Node;

use super::partition_names;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::name::{NameKind, ParsedNames, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Go source files.
pub struct GoParser;

impl Parser for GoParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_go_imports(source, file_path)
    }

    fn parse_call_exprs(&self, source: &str, file_path: &str) -> Vec<RawCallExpr> {
        parse_go_call_exprs(source, file_path)
    }

    fn parse_names(&self, source: &str, file_path: &str) -> ParsedNames {
        parse_go_names(source, file_path)
    }
}

/// Parse Go source code and extract named entities for naming convention checks.
///
/// Extracts:
/// - `Symbol`: function_declaration, method_declaration, type_declaration names
/// - `Variable`: var_declaration, const_declaration, short_var_declaration identifiers
/// - `Comment`: comment content
pub fn parse_go_names(source: &str, file_path: &str) -> ParsedNames {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::language())
        .expect("Failed to load Go grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut names = Vec::new();
    collect_go_names(root, source.as_bytes(), file_path, &mut names);
    partition_names(names)
}

fn collect_go_names(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawName>) {
    let kind = node.kind();
    let line = node.start_position().row + 1;

    match kind {
        "function_declaration" | "method_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node.utf8_text(source).unwrap_or("").to_string();
                if !name.is_empty() {
                    out.push(RawName {
                        name,
                        line,
                        kind: NameKind::Symbol,
                        file: file_path.to_string(),
                    });
                }
            }
        }
        "type_declaration" => {
            // type_declaration may contain type_spec children
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "type_spec" {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = name_node.utf8_text(source).unwrap_or("").to_string();
                            if !name.is_empty() {
                                out.push(RawName {
                                    name,
                                    line,
                                    kind: NameKind::Symbol,
                                    file: file_path.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        "var_declaration" => {
            // var_declaration contains var_spec children
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "var_spec" {
                        collect_go_spec_names(&child, source, file_path, out, NameKind::Variable);
                    }
                }
            }
        }
        "const_declaration" => {
            // const_declaration contains const_spec children
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "const_spec" {
                        collect_go_spec_names(&child, source, file_path, out, NameKind::Variable);
                    }
                }
            }
        }
        "short_var_declaration" => {
            // left side is identifier_list
            if let Some(left) = node.child_by_field_name("left") {
                let name = left.utf8_text(source).unwrap_or("").to_string();
                if !name.is_empty() {
                    out.push(RawName {
                        name,
                        line,
                        kind: NameKind::Variable,
                        file: file_path.to_string(),
                    });
                }
            }
        }
        "comment" => {
            let text = node.utf8_text(source).unwrap_or("").to_string();
            if !text.is_empty() {
                out.push(RawName {
                    name: text,
                    line,
                    kind: NameKind::Comment,
                    file: file_path.to_string(),
                });
            }
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_go_names(child, source, file_path, out);
        }
    }
}

fn collect_go_spec_names(
    node: &Node,
    source: &[u8],
    file_path: &str,
    out: &mut Vec<RawName>,
    kind: NameKind,
) {
    // var_spec / const_spec: first child is identifier_list or identifier
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "identifier" => {
                    let name = child.utf8_text(source).unwrap_or("").to_string();
                    let line = child.start_position().row + 1;
                    if !name.is_empty() {
                        out.push(RawName {
                            name,
                            line,
                            kind,
                            file: file_path.to_string(),
                        });
                    }
                }
                "identifier_list" => {
                    for j in 0..child.child_count() {
                        if let Some(id) = child.child(j) {
                            if id.kind() == "identifier" {
                                let name = id.utf8_text(source).unwrap_or("").to_string();
                                let line = id.start_position().row + 1;
                                if !name.is_empty() {
                                    out.push(RawName {
                                        name,
                                        line,
                                        kind,
                                        file: file_path.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Parse Go source code and extract all `import` declarations.
pub fn parse_go_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::language())
        .expect("Failed to load Go grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_go_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

fn collect_go_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    if node.kind() == "import_declaration" {
        let line = node.start_position().row + 1;
        // Child is either import_spec (single) or import_spec_list (grouped)
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "import_spec" => {
                        if let Some(path) = extract_import_path(&child, source) {
                            out.push(RawImport {
                                path,
                                line,
                                file: file_path.to_string(),
                                kind: ImportKind::Import,
                                named_imports: vec![],
                            });
                        }
                    }
                    "import_spec_list" => {
                        for j in 0..child.child_count() {
                            if let Some(spec) = child.child(j) {
                                if spec.kind() == "import_spec" {
                                    let spec_line = spec.start_position().row + 1;
                                    if let Some(path) = extract_import_path(&spec, source) {
                                        out.push(RawImport {
                                            path,
                                            line: spec_line,
                                            file: file_path.to_string(),
                                            kind: ImportKind::Import,
                                            named_imports: vec![],
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        return; // import_declaration children already handled above
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_go_imports(child, source, file_path, out);
        }
    }
}

/// Parse Go source code and extract package-level call expressions.
///
/// Extracts `selector_expression` calls of the form `pkg.Func(...)`:
/// - `receiver_type = "pkg"` (the package identifier)
/// - `method = "Func"` (the function/method name)
///
/// These are the only statically-typed calls Go supports at the package level.
/// Instance method calls like `obj.Method()` where `obj` is a variable are also
/// extracted but with `receiver_type = Some(obj_name)` — the ViolationDetector
/// matches by import, so only package-level calls will trigger CallPatternViolation.
pub fn parse_go_call_exprs(source: &str, file_path: &str) -> Vec<RawCallExpr> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_go::language())
        .expect("Failed to load Go grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut calls = Vec::new();
    collect_go_call_exprs(root, source.as_bytes(), file_path, &mut calls);
    calls
}

fn collect_go_call_exprs(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawCallExpr>) {
    if node.kind() == "call_expression" {
        let line = node.start_position().row + 1;
        if let Some(func) = node.child_by_field_name("function") {
            if func.kind() == "selector_expression" {
                // pkg.Func(...)
                if let (Some(operand), Some(field)) = (
                    func.child_by_field_name("operand"),
                    func.child_by_field_name("field"),
                ) {
                    let receiver = operand.utf8_text(source).unwrap_or("").to_string();
                    let method = field.utf8_text(source).unwrap_or("").to_string();
                    if !receiver.is_empty() && !method.is_empty() {
                        out.push(RawCallExpr {
                            file: file_path.to_string(),
                            line,
                            receiver_type: Some(receiver),
                            method,
                        });
                    }
                }
            }
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_go_call_exprs(child, source, file_path, out);
        }
    }
}

/// Extract the import path string from an `import_spec` node.
/// The path is in an `interpreted_string_literal` child; strip surrounding `"`.
fn extract_import_path(spec: &Node, source: &[u8]) -> Option<String> {
    for i in 0..spec.child_count() {
        if let Some(child) = spec.child(i) {
            if child.kind() == "interpreted_string_literal" {
                let raw = child.utf8_text(source).unwrap_or("");
                // Strip surrounding double quotes
                let path = raw.trim_matches('"').to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::ImportKind;

    #[test]
    fn test_parse_go_single_import() {
        let source = "package main\n\nimport \"fmt\"\n";
        let imports = parse_go_imports(source, "main.go");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "fmt");
        assert_eq!(imports[0].kind, ImportKind::Import);
        assert_eq!(imports[0].line, 3);
        assert_eq!(imports[0].file, "main.go");
    }

    #[test]
    fn test_parse_go_grouped_imports() {
        let source = "package main\n\nimport (\n    \"fmt\"\n    \"net/http\"\n)\n";
        let imports = parse_go_imports(source, "server.go");
        assert_eq!(imports.len(), 2);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"fmt"));
        assert!(paths.contains(&"net/http"));
        assert!(imports.iter().all(|i| i.kind == ImportKind::Import));
    }

    #[test]
    fn test_parse_go_internal_import() {
        let source = "package usecase\n\nimport \"github.com/example/myapp/domain\"\n";
        let imports = parse_go_imports(source, "usecase/user_usecase.go");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "github.com/example/myapp/domain");
        assert_eq!(imports[0].kind, ImportKind::Import);
    }

    #[test]
    fn test_parse_go_aliased_import() {
        let source = "package main\n\nimport (\n    \"fmt\"\n    myfmt \"fmt\"\n)\n";
        // Both imports should be captured (path without alias)
        let imports = parse_go_imports(source, "main.go");
        assert_eq!(imports.len(), 2);
        // Both should have path "fmt"
        assert!(imports.iter().all(|i| i.path == "fmt"));
    }

    #[test]
    fn test_parse_go_blank_import() {
        let source = "package main\n\nimport _ \"database/sql/driver\"\n";
        let imports = parse_go_imports(source, "main.go");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "database/sql/driver");
    }

    #[test]
    fn test_parse_go_no_imports() {
        let source = "package main\n\nfunc main() {}\n";
        let imports = parse_go_imports(source, "main.go");
        assert!(imports.is_empty());
    }

    // ------------------------------------------------------------------
    // parse_go_names
    // ------------------------------------------------------------------

    use crate::domain::entity::name::NameKind;

    #[test]
    fn test_go_parse_names_function() {
        let source = "package main\n\nfunc AwsHandler() {}\n";
        let names = parse_go_names(source, "test.go").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "AwsHandler" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "func AwsHandler should be detected as Symbol, got: {:#?}",
            names
        );
        assert_eq!(found.unwrap().line, 3);
    }

    #[test]
    fn test_go_parse_names_var() {
        let source = "package main\n\nvar awsUrl string\n";
        let names = parse_go_names(source, "test.go").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "awsUrl" && n.kind == NameKind::Variable);
        assert!(
            found.is_some(),
            "var awsUrl should be detected as Variable, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_go_parse_names_line_comment() {
        let source = "package main\n\n// connect to aws\nfunc f() {}\n";
        let names = parse_go_names(source, "test.go").into_all();
        let found = names
            .iter()
            .find(|n| n.kind == NameKind::Comment && n.name.contains("aws"));
        assert!(
            found.is_some(),
            "line comment with aws should be detected, got: {:#?}",
            names
        );
    }
}
