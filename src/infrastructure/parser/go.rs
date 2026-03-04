use tree_sitter::Node;

use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Go source files.
pub struct GoParser;

impl Parser for GoParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_go_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // Go call-expression analysis is not yet implemented; return empty.
        vec![]
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
}
