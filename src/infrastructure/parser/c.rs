use tree_sitter::Node;

use super::partition_names;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::name::{NameKind, ParsedNames, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for C source files.
pub struct CParser;

impl Parser for CParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_c_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // C does not have static method dispatch (e.g. Class::method()).
        vec![]
    }

    fn parse_names(&self, source: &str, file_path: &str) -> ParsedNames {
        parse_c_names(source, file_path)
    }
}

/// Parse C source code and extract `#include` directives.
///
/// Handles:
/// - System includes: `#include <stdio.h>` → `ImportKind::Import`, path = `stdio.h`
/// - Local includes:  `#include "user.h"`  → `ImportKind::Use`, path = `user.h`
pub fn parse_c_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_c::language())
        .expect("Failed to load C grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_c_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

fn collect_c_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    if node.kind() == "preproc_include" {
        let line = node.start_position().row + 1;
        // The path child can be system_lib_string (<...>) or string_literal ("...")
        if let Some(path_node) = node.child_by_field_name("path") {
            let raw_text = path_node.utf8_text(source).unwrap_or("").to_string();
            let kind = if path_node.kind() == "system_lib_string" {
                ImportKind::Import // system include <...>
            } else {
                ImportKind::Use // local include "..."
            };
            // Strip surrounding < > or " "
            let path = raw_text
                .trim_start_matches('<')
                .trim_end_matches('>')
                .trim_start_matches('"')
                .trim_end_matches('"')
                .to_string();
            if !path.is_empty() {
                out.push(RawImport {
                    path,
                    line,
                    file: file_path.to_string(),
                    kind,
                    named_imports: vec![],
                });
            }
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_c_imports(child, source, file_path, out);
        }
    }
}

/// Parse C source code and extract named entities for naming convention checks.
///
/// Extracts:
/// - `Symbol`: function definitions, struct/enum/union/typedef declarations
/// - `Variable`: global variable declarations, parameter declarations
/// - `Comment`: line and block comments
pub fn parse_c_names(source: &str, file_path: &str) -> ParsedNames {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_c::language())
        .expect("Failed to load C grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut names = Vec::new();
    collect_c_names(root, source.as_bytes(), file_path, &mut names);
    partition_names(names)
}

fn collect_c_names(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawName>) {
    let kind = node.kind();
    let line = node.start_position().row + 1;

    match kind {
        // Symbols: function definitions
        "function_definition" => {
            if let Some(declarator) = node.child_by_field_name("declarator") {
                // The declarator may be a function_declarator; the name is inside it
                extract_function_name(declarator, source, file_path, line, out);
            }
        }
        // Symbols: struct, enum, union specifiers with a name
        "struct_specifier" | "enum_specifier" | "union_specifier" => {
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
        // Symbols: typedef (extract the alias name from the last declarator)
        "type_definition" => {
            if let Some(declarator) = node.child_by_field_name("declarator") {
                let name = if declarator.kind() == "type_identifier" {
                    declarator.utf8_text(source).unwrap_or("").to_string()
                } else {
                    String::new()
                };
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
        // Variables: top-level declarations (global variables, constants)
        "declaration" => {
            if let Some(declarator) = node.child_by_field_name("declarator") {
                extract_variable_name(declarator, source, file_path, line, out);
            }
        }
        // Comments
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
        // Identifier: field expression (e.g. `s.field` or `p->field` → extract `field`)
        "field_expression" => {
            if let Some(field_node) = node.child_by_field_name("field") {
                let name = field_node.utf8_text(source).unwrap_or("").to_string();
                if !name.is_empty() {
                    out.push(RawName {
                        name,
                        line: field_node.start_position().row + 1,
                        kind: NameKind::Identifier,
                        file: file_path.to_string(),
                    });
                }
            }
            // Recurse into argument to capture nested field access
            if let Some(arg) = node.child_by_field_name("argument") {
                collect_c_names(arg, source, file_path, out);
            }
            return;
        }
        // String literals
        "string_literal" => {
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
            collect_c_names(child, source, file_path, out);
        }
    }
}

/// Extract function name from a declarator node.
fn extract_function_name(
    node: Node,
    source: &[u8],
    file_path: &str,
    line: usize,
    out: &mut Vec<RawName>,
) {
    match node.kind() {
        "function_declarator" => {
            if let Some(name_node) = node.child_by_field_name("declarator") {
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
        "pointer_declarator" => {
            // e.g. `int *get_ptr()` — the function_declarator is a child
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    extract_function_name(child, source, file_path, line, out);
                }
            }
        }
        _ => {}
    }
}

/// Extract variable name from a declarator node.
fn extract_variable_name(
    node: Node,
    source: &[u8],
    file_path: &str,
    line: usize,
    out: &mut Vec<RawName>,
) {
    match node.kind() {
        "identifier" => {
            let name = node.utf8_text(source).unwrap_or("").to_string();
            if !name.is_empty() {
                out.push(RawName {
                    name,
                    line,
                    kind: NameKind::Variable,
                    file: file_path.to_string(),
                });
            }
        }
        "init_declarator" => {
            if let Some(decl) = node.child_by_field_name("declarator") {
                extract_variable_name(decl, source, file_path, line, out);
            }
        }
        "pointer_declarator" => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() == "identifier" {
                        extract_variable_name(child, source, file_path, line, out);
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::name::NameKind;

    fn parse(src: &str) -> Vec<RawImport> {
        parse_c_imports(src, "test.c")
    }

    #[test]
    fn test_system_include() {
        let imports = parse("#include <stdio.h>\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "stdio.h");
        assert_eq!(imports[0].kind, ImportKind::Import);
    }

    #[test]
    fn test_local_include() {
        let imports = parse("#include \"user.h\"\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "user.h");
        assert_eq!(imports[0].kind, ImportKind::Use);
    }

    #[test]
    fn test_multiple_includes() {
        let src = r#"
#include <stdio.h>
#include <stdlib.h>
#include "domain/user.h"
"#;
        let imports = parse(src);
        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0].path, "stdio.h");
        assert_eq!(imports[1].path, "stdlib.h");
        assert_eq!(imports[2].path, "domain/user.h");
    }

    // parse_c_names
    #[test]
    fn test_parse_c_names_function() {
        let source = "int main(int argc, char *argv[]) { return 0; }";
        let names = parse_c_names(source, "test.c").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "main" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "function 'main' should be detected as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_parse_c_names_struct() {
        let source = "struct User { int id; char *name; };";
        let names = parse_c_names(source, "test.c").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "User" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "struct 'User' should be detected as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_parse_c_names_variable() {
        let source = "int aws_count = 42;";
        let names = parse_c_names(source, "test.c").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "aws_count" && n.kind == NameKind::Variable);
        assert!(
            found.is_some(),
            "variable 'aws_count' should be detected as Variable, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_parse_c_names_comment() {
        let source = "// connect to database\nint x = 1;";
        let names = parse_c_names(source, "test.c").into_all();
        let found = names
            .iter()
            .find(|n| n.kind == NameKind::Comment && n.name.contains("connect to database"));
        assert!(
            found.is_some(),
            "comment should be detected, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_parse_c_names_typedef() {
        let source = "typedef unsigned int uint32;";
        let names = parse_c_names(source, "test.c").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "uint32" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "typedef 'uint32' should be detected as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_c_parse_names_field_identifier() {
        let source = "void f() { int x = config.gcp.bucket; }";
        let names = parse_c_names(source, "test.c").into_all();
        let gcp = names
            .iter()
            .find(|n| n.name == "gcp" && n.kind == NameKind::Identifier);
        assert!(
            gcp.is_some(),
            "field access 'gcp' should be detected as Identifier, got: {:#?}",
            names
        );
        let bucket = names
            .iter()
            .find(|n| n.name == "bucket" && n.kind == NameKind::Identifier);
        assert!(
            bucket.is_some(),
            "field access 'bucket' should be detected as Identifier, got: {:#?}",
            names
        );
    }
}
