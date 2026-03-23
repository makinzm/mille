use tree_sitter::Node;

use super::partition_names;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::name::{NameKind, ParsedNames, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Kotlin source files.
pub struct KotlinParser;

impl Parser for KotlinParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_kotlin_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        vec![]
    }

    fn parse_names(&self, source: &str, file_path: &str) -> ParsedNames {
        parse_kotlin_names(source, file_path)
    }
}

/// Parse Kotlin source code and extract named entities for naming convention checks.
///
/// Extracts:
/// - `Symbol`: class, interface, object, function declarations
/// - `Variable`: property declarations, local variable declarations
/// - `Comment`: multiline_comment, line_comment
///
/// NOTE: tree-sitter-kotlin comment node types are `multiline_comment` and `line_comment`.
pub fn parse_kotlin_names(source: &str, file_path: &str) -> ParsedNames {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_kotlin::language())
        .expect("Failed to load Kotlin grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut names = Vec::new();
    collect_kotlin_names(root, source.as_bytes(), file_path, &mut names);
    partition_names(names)
}

fn collect_kotlin_names(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawName>) {
    let kind = node.kind();
    let line = node.start_position().row + 1;

    match kind {
        // Symbols: class, interface, object declarations
        "class_declaration" | "interface_declaration" | "object_declaration" => {
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
        "function_declaration" => {
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
        // Variables: property declarations
        "property_declaration" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = name_node.utf8_text(source).unwrap_or("").to_string();
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
        // Comments
        "multiline_comment" | "line_comment" => {
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
        // Identifier: navigation expression (e.g. `obj.prop` → extract `prop`)
        "navigation_expression" => {
            // navigation_expression children: object, navigation_suffix
            // navigation_suffix contains the identifier after the dot
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "navigation_suffix" {
                        // The identifier inside navigation_suffix
                        if let Some(id) = child.child_by_field_name("name") {
                            let name = id.utf8_text(source).unwrap_or("").to_string();
                            if !name.is_empty() {
                                out.push(RawName {
                                    name,
                                    line: id.start_position().row + 1,
                                    kind: NameKind::Identifier,
                                    file: file_path.to_string(),
                                });
                            }
                        } else {
                            // Fallback: try simple_identifier child
                            for j in 0..child.child_count() {
                                if let Some(inner) = child.child(j) {
                                    if inner.kind() == "simple_identifier" {
                                        let name =
                                            inner.utf8_text(source).unwrap_or("").to_string();
                                        if !name.is_empty() {
                                            out.push(RawName {
                                                name,
                                                line: inner.start_position().row + 1,
                                                kind: NameKind::Identifier,
                                                file: file_path.to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Recurse into first child (the object) to capture nested navigation
            if let Some(obj) = node.child(0) {
                if obj.kind() != "navigation_suffix" {
                    collect_kotlin_names(obj, source, file_path, out);
                }
            }
            return;
        }
        // String literals
        "line_string_literal" | "multi_line_string_literal" => {
            let text = node.utf8_text(source).unwrap_or("");
            let content = super::strip_string_delimiters(text);
            if !content.is_empty() {
                out.push(RawName {
                    name: content,
                    line,
                    kind: NameKind::StringLiteral,
                    file: file_path.to_string(),
                });
            }
            return;
        }
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_kotlin_names(child, source, file_path, out);
        }
    }
}

/// Parse Kotlin source code and extract all `import` declarations.
pub fn parse_kotlin_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_kotlin::language())
        .expect("Failed to load Kotlin grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_kotlin_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

fn collect_kotlin_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    if node.kind() == "import_header" {
        let line = node.start_position().row + 1;
        if let Some(path) = extract_kotlin_import_path(&node, source) {
            out.push(RawImport {
                path,
                line,
                file: file_path.to_string(),
                kind: ImportKind::Import,
                named_imports: vec![],
            });
        }
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_kotlin_imports(child, source, file_path, out);
        }
    }
}

/// Extract the dotted import path from an `import_header` node.
///
/// Grammar: `'import' $identifier optional(seq('.', '*'))`
/// The identifier child contains the full dotted path (wildcard stripped).
fn extract_kotlin_import_path(node: &Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            if child.kind() == "identifier" {
                let text = child.utf8_text(source).unwrap_or("").to_string();
                if !text.is_empty() {
                    return Some(text);
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
    fn test_parse_kotlin_single_import() {
        let source = "package com.example\n\nimport com.example.usecase.UserService\n\nclass Foo\n";
        let imports = parse_kotlin_imports(source, "Foo.kt");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "com.example.usecase.UserService");
        assert_eq!(imports[0].kind, ImportKind::Import);
        assert_eq!(imports[0].line, 3);
        assert_eq!(imports[0].file, "Foo.kt");
    }

    #[test]
    fn test_parse_kotlin_wildcard_import() {
        let source = "package com.example\n\nimport com.example.domain.*\n\nclass Foo\n";
        let imports = parse_kotlin_imports(source, "Foo.kt");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "com.example.domain");
        assert_eq!(imports[0].kind, ImportKind::Import);
    }

    #[test]
    fn test_parse_kotlin_multiple_imports() {
        let source = "package com.example\n\nimport com.example.domain.User\nimport com.example.usecase.UserService\nimport kotlin.collections.List\n\nclass Foo\n";
        let imports = parse_kotlin_imports(source, "Foo.kt");
        assert_eq!(imports.len(), 3);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"com.example.domain.User"));
        assert!(paths.contains(&"com.example.usecase.UserService"));
        assert!(paths.contains(&"kotlin.collections.List"));
    }

    #[test]
    fn test_parse_kotlin_no_imports() {
        let source = "package com.example\n\nclass Foo\n";
        let imports = parse_kotlin_imports(source, "Foo.kt");
        assert!(imports.is_empty());
    }

    #[test]
    fn test_parse_kotlin_call_exprs_empty() {
        let parser = KotlinParser;
        let source = "package com.example\n\nclass Foo { fun bar() {} }\n";
        let calls = parser.parse_call_exprs(source, "Foo.kt");
        assert!(calls.is_empty());
    }
}
