use tree_sitter::Node;

use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Python source files.
pub struct PythonParser;

impl Parser for PythonParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_python_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // Python call-expression analysis is not yet implemented.
        vec![]
    }
}

/// Parse Python source code and extract all `import` and `from ... import` statements.
pub fn parse_python_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::language())
        .expect("Failed to load Python grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_python_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

fn collect_python_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    match node.kind() {
        "import_statement" => {
            // import X   import X as Y   import X, Y
            let line = node.start_position().row + 1;
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    let path = match child.kind() {
                        "dotted_name" => extract_text(&child, source),
                        "aliased_import" => {
                            // aliased_import: dotted_name "as" identifier
                            child
                                .child(0)
                                .filter(|n| n.kind() == "dotted_name")
                                .and_then(|n| extract_text(&n, source))
                        }
                        _ => None,
                    };
                    if let Some(p) = path {
                        out.push(RawImport {
                            path: p,
                            line,
                            file: file_path.to_string(),
                            kind: ImportKind::Import,
                            named_imports: vec![],
                        });
                    }
                }
            }
        }
        "import_from_statement" => {
            // from X import Y   from . import Y   from .X import Y
            let line = node.start_position().row + 1;
            if let Some(path) = extract_from_module(&node, source) {
                let named = extract_python_named_imports(&node, source);
                out.push(RawImport {
                    path,
                    line,
                    file: file_path.to_string(),
                    kind: ImportKind::Import,
                    named_imports: named,
                });
            }
        }
        _ => {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    collect_python_imports(child, source, file_path, out);
                }
            }
        }
    }
}

/// Extract the module path from a `from … import …` statement.
///
/// Returns:
/// - `"."` for `from . import X` (relative, current package)
/// - `".sub"` for `from .sub import X`
/// - `"os"` for `from os import path`
fn extract_from_module(node: &Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        let child = node.child(i)?;
        match child.kind() {
            // Absolute import: "from os import path"
            "dotted_name" => return extract_text(&child, source),
            // Relative import: "from . import X" or "from .sub import X"
            "relative_import" => return extract_text(&child, source),
            _ => {}
        }
    }
    None
}

/// Extract the list of names imported by `from X import Y, Z`.
///
/// For `from domain.entity import User, Admin` → returns `["User", "Admin"]`.
/// For `from . import entity` → returns `["entity"]`.
/// For `from os import *` → returns `[]` (wildcard).
fn extract_python_named_imports(node: &Node, source: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut after_import_kw = false;

    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        if child.kind() == "import" {
            after_import_kw = true;
            continue;
        }
        if !after_import_kw {
            continue;
        }
        match child.kind() {
            "wildcard_import" => {
                // from X import * — cannot enumerate names
                return vec![];
            }
            "dotted_name" | "identifier" => {
                if let Some(text) = extract_text(&child, source) {
                    // Only take the last component for dotted names used as identifiers
                    let name = text.split('.').last().unwrap_or(&text).to_string();
                    names.push(name);
                }
            }
            "aliased_import" => {
                // from X import Y as Z — the bound name is Z
                for j in 0..child.child_count() {
                    let Some(inner) = child.child(j) else { continue };
                    if inner.kind() == "identifier" && j == 0 {
                        // first identifier is the original name
                        if let Some(text) = extract_text(&inner, source) {
                            names.push(text);
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    names
}

fn extract_text(node: &Node, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Vec<RawImport> {
        parse_python_imports(src, "test.py")
    }

    #[test]
    fn test_parse_python_simple_import() {
        let imports = parse("import os\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "os");
        assert_eq!(imports[0].line, 1);
    }

    #[test]
    fn test_parse_python_dotted_import() {
        let imports = parse("import os.path\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "os.path");
    }

    #[test]
    fn test_parse_python_aliased_import() {
        let imports = parse("import numpy as np\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "numpy");
    }

    #[test]
    fn test_parse_python_from_import() {
        let imports = parse("from os import path\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "os");
    }

    #[test]
    fn test_parse_python_from_dotted_import() {
        let imports = parse("from domain.entity import User\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "domain.entity");
    }

    #[test]
    fn test_parse_python_relative_import() {
        let imports = parse("from . import entity\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, ".");
    }

    #[test]
    fn test_parse_python_relative_dotted_import() {
        let imports = parse("from .entity import User\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, ".entity");
    }

    #[test]
    fn test_parse_python_multiple_imports() {
        let src = "import os\nimport sys\nfrom domain.entity import User\n";
        let imports = parse(src);
        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0].path, "os");
        assert_eq!(imports[1].path, "sys");
        assert_eq!(imports[2].path, "domain.entity");
    }

    #[test]
    fn test_parse_python_no_imports() {
        let imports = parse("x = 1\nprint(x)\n");
        assert!(imports.is_empty());
    }
}
