use tree_sitter::Node;

use super::partition_names;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::name::{NameKind, ParsedNames, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Python source files.
pub struct PythonParser;

impl Parser for PythonParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_python_imports(source, file_path)
    }

    fn parse_call_exprs(&self, source: &str, file_path: &str) -> Vec<RawCallExpr> {
        parse_python_call_exprs(source, file_path)
    }

    fn parse_names(&self, source: &str, file_path: &str) -> ParsedNames {
        parse_python_names(source, file_path)
    }
}

/// Parse Python source code and extract named entities for naming convention checks.
///
/// Extracts:
/// - `Symbol`: function_definition, class_definition names
/// - `Variable`: assignment targets at module/function scope (simple identifiers)
/// - `Comment`: comment content
pub fn parse_python_names(source: &str, file_path: &str) -> ParsedNames {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::language())
        .expect("Failed to load Python grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut names = Vec::new();
    collect_python_names(root, source.as_bytes(), file_path, &mut names);
    partition_names(names)
}

fn collect_python_names(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawName>) {
    let kind = node.kind();
    let line = node.start_position().row + 1;

    match kind {
        "function_definition" | "async_function_definition" => {
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
        "class_definition" => {
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
        // Variables: simple assignments (x = ...) at module/function scope
        "assignment" => {
            if let Some(left) = node.child_by_field_name("left") {
                if left.kind() == "identifier" {
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
        // String literals
        "string" => {
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
            collect_python_names(child, source, file_path, out);
        }
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

/// Parse Python source code and extract attribute-access call expressions.
///
/// Extracts calls of the form `Receiver.method(...)`:
/// - `receiver_type = Some("Receiver")` (object/class name)
/// - `method = "method"`
///
/// Only immediate attribute calls are captured: `User.create("John")` →
/// receiver = "User", method = "create". Chained calls like `a.b.c()` are
/// not captured because static type inference is not available.
pub fn parse_python_call_exprs(source: &str, file_path: &str) -> Vec<RawCallExpr> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_python::language())
        .expect("Failed to load Python grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut calls = Vec::new();
    collect_python_call_exprs(root, source.as_bytes(), file_path, &mut calls);
    calls
}

fn collect_python_call_exprs(
    node: Node,
    source: &[u8],
    file_path: &str,
    out: &mut Vec<RawCallExpr>,
) {
    if node.kind() == "call" {
        let line = node.start_position().row + 1;
        if let Some(func) = node.child_by_field_name("function") {
            if func.kind() == "attribute" {
                // Receiver.method(...)
                if let (Some(obj), Some(attr)) = (
                    func.child_by_field_name("object"),
                    func.child_by_field_name("attribute"),
                ) {
                    // Only handle simple identifier receivers (e.g. `User.create`)
                    if obj.kind() == "identifier" {
                        let receiver = obj.utf8_text(source).unwrap_or("").to_string();
                        let method = attr.utf8_text(source).unwrap_or("").to_string();
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
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_python_call_exprs(child, source, file_path, out);
        }
    }
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
                    let Some(inner) = child.child(j) else {
                        continue;
                    };
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

    // ------------------------------------------------------------------
    // parse_python_names
    // ------------------------------------------------------------------

    use crate::domain::entity::name::NameKind;

    #[test]
    fn test_py_parse_names_function() {
        let source = "def aws_handler():\n    pass\n";
        let names = parse_python_names(source, "test.py").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "aws_handler" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "def aws_handler should be detected as Symbol, got: {:#?}",
            names
        );
        assert_eq!(found.unwrap().line, 1);
    }

    #[test]
    fn test_py_parse_names_class() {
        let source = "class AwsClient:\n    pass\n";
        let names = parse_python_names(source, "test.py").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "AwsClient" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "class AwsClient should be detected as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_py_parse_names_comment() {
        let source = "# connect to aws\nx = 1\n";
        let names = parse_python_names(source, "test.py").into_all();
        let found = names
            .iter()
            .find(|n| n.kind == NameKind::Comment && n.name.contains("aws"));
        assert!(
            found.is_some(),
            "comment with aws should be detected, got: {:#?}",
            names
        );
        assert_eq!(found.unwrap().line, 1);
    }
}
