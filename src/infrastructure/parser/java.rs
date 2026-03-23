use tree_sitter::Node;

use super::partition_names;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::name::{NameKind, ParsedNames, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Java source files.
pub struct JavaParser;

impl Parser for JavaParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_java_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // Java call expression analysis is not yet implemented.
        // Return an empty Vec consistent with the Go parser approach.
        vec![]
    }

    fn parse_names(&self, source: &str, file_path: &str) -> ParsedNames {
        parse_java_names(source, file_path)
    }
}

/// Parse Java source code and extract named entities for naming convention checks.
///
/// Extracts:
/// - `Symbol`: class, interface, enum, method declarations
/// - `Variable`: field declarations, local variable declarations
/// - `Comment`: line_comment, block_comment
///
/// NOTE: tree-sitter-java comment node types are `line_comment` and `block_comment`.
pub fn parse_java_names(source: &str, file_path: &str) -> ParsedNames {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_java::language())
        .expect("Failed to load Java grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut names = Vec::new();
    collect_java_names(root, source.as_bytes(), file_path, &mut names);
    partition_names(names)
}

fn collect_java_names(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawName>) {
    let kind = node.kind();
    let line = node.start_position().row + 1;

    match kind {
        // Symbols: class, interface, enum, annotation declarations
        "class_declaration"
        | "interface_declaration"
        | "enum_declaration"
        | "annotation_type_declaration"
        | "record_declaration" => {
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
        "method_declaration" | "constructor_declaration" => {
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
        // Variables: field declarations, local variable declarations
        "field_declaration" | "local_variable_declaration" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "variable_declarator" {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = name_node.utf8_text(source).unwrap_or("").to_string();
                            let var_line = child.start_position().row + 1;
                            if !name.is_empty() {
                                out.push(RawName {
                                    name,
                                    line: var_line,
                                    kind: NameKind::Variable,
                                    file: file_path.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        // Comments
        "line_comment" | "block_comment" => {
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
            collect_java_names(child, source, file_path, out);
        }
    }
}

/// Parse Java source code and extract all `import` declarations.
pub fn parse_java_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_java::language())
        .expect("Failed to load Java grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_java_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

fn collect_java_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    if node.kind() == "import_declaration" {
        let line = node.start_position().row + 1;

        // Both regular and static imports use the same `ImportKind::Import`.
        // Grammar: 'import' optional('static') $._name optional(seq('.', asterisk)) ';'
        // The import path is extracted from the `scoped_identifier` or `identifier` child.
        if let Some(path) = extract_java_import_path(&node, source) {
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
            collect_java_imports(child, source, file_path, out);
        }
    }
}

/// Extract the dotted import path from an `import_declaration` node.
///
/// Grammar: `'import' optional('static') $._name optional(seq('.', asterisk)) ';'`
/// The name is a `scoped_identifier` (e.g. `com.example.Foo`) or `identifier`.
fn extract_java_import_path(node: &Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            match child.kind() {
                "scoped_identifier" | "identifier" => {
                    let text = child.utf8_text(source).unwrap_or("").to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
                _ => {}
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
    fn test_parse_java_single_import() {
        let source = "package com.example;\n\nimport java.util.List;\n\npublic class Foo {}\n";
        let imports = parse_java_imports(source, "Foo.java");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "java.util.List");
        assert_eq!(imports[0].kind, ImportKind::Import);
        assert_eq!(imports[0].line, 3);
        assert_eq!(imports[0].file, "Foo.java");
    }

    #[test]
    fn test_parse_java_static_import() {
        let source =
            "package com.example;\n\nimport static com.example.Foo.bar;\n\npublic class Baz {}\n";
        let imports = parse_java_imports(source, "Baz.java");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "com.example.Foo.bar");
        assert_eq!(imports[0].kind, ImportKind::Import);
    }

    #[test]
    fn test_parse_java_multiple_imports() {
        let source = "package com.example;\n\nimport java.util.List;\nimport java.util.Map;\nimport com.example.domain.User;\n\npublic class Foo {}\n";
        let imports = parse_java_imports(source, "Foo.java");
        assert_eq!(imports.len(), 3);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"java.util.List"));
        assert!(paths.contains(&"java.util.Map"));
        assert!(paths.contains(&"com.example.domain.User"));
    }

    #[test]
    fn test_parse_java_no_imports() {
        let source = "package com.example;\n\npublic class Foo {}\n";
        let imports = parse_java_imports(source, "Foo.java");
        assert!(imports.is_empty());
    }

    #[test]
    fn test_parse_java_call_exprs_empty() {
        let parser = JavaParser;
        let source = "package com.example;\n\npublic class Foo { public void bar() {} }\n";
        let calls = parser.parse_call_exprs(source, "Foo.java");
        assert!(calls.is_empty());
    }
}
