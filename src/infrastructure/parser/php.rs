use tree_sitter::Node;

use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::name::{NameKind, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for PHP source files.
pub struct PhpParser;

impl Parser for PhpParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_php_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // PHP call expression analysis is not yet implemented.
        vec![]
    }

    fn parse_names(&self, source: &str, file_path: &str) -> Vec<RawName> {
        parse_php_names(source, file_path)
    }
}

/// Parse PHP source code and extract all `use` declarations.
///
/// Handles:
/// - Simple: `use App\Models\User;`
/// - Aliased: `use App\Models\User as UserModel;` (alias ignored, original path returned)
/// - Grouped: `use App\Services\{Auth, Logger};` (expands to one import per name)
/// - Function: `use function App\Helpers\format_date;`
/// - Constant: `use const App\Config\MAX_RETRIES;`
pub fn parse_php_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_php::language_php())
        .expect("Failed to load PHP grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_php_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

fn collect_php_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    if node.kind() == "namespace_use_declaration" {
        let line = node.start_position().row + 1;
        extract_namespace_use_declaration(&node, source, file_path, line, out);
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_php_imports(child, source, file_path, out);
        }
    }
}

/// Extract imports from a `namespace_use_declaration` node.
///
/// Two forms:
/// 1. `use [function|const] clause1, clause2, ...;`
///    → children include `namespace_use_clause` nodes
/// 2. `use [function|const] Prefix\ {Name1, Name2, ...};`
///    → children include a `namespace_name`, then `namespace_use_group`
fn extract_namespace_use_declaration(
    node: &Node,
    source: &[u8],
    file_path: &str,
    line: usize,
    out: &mut Vec<RawImport>,
) {
    // Check if this is a group use: look for namespace_use_group child
    let mut prefix: Option<String> = None;
    let mut has_group = false;

    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        match child.kind() {
            "namespace_name" => {
                // Group use prefix: `App\Services` in `use App\Services\{Auth, Logger}`
                prefix = extract_text(&child, source);
            }
            "namespace_use_group" => {
                has_group = true;
                let base = prefix.as_deref().unwrap_or("");
                extract_group_use(&child, source, file_path, line, base, out);
            }
            "namespace_use_clause" => {
                // Simple / aliased / function / const use
                if let Some(path) = extract_use_clause_path(&child, source) {
                    out.push(RawImport {
                        path,
                        line,
                        file: file_path.to_string(),
                        kind: ImportKind::Import,
                        named_imports: vec![],
                    });
                }
            }
            _ => {}
        }
    }

    let _ = has_group; // suppress unused warning
}

/// Extract the import path from a `namespace_use_clause` node.
///
/// Handles:
/// - `qualified_name` (e.g. `App\Models\User`)
/// - `name` (bare identifier)
/// - Optional `namespace_aliasing_clause` is ignored (we return the original path).
fn extract_use_clause_path(node: &Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        match child.kind() {
            "qualified_name" => return extract_qualified_name(&child, source),
            "name" => return extract_text(&child, source),
            _ => {}
        }
    }
    None
}

/// Extract the fully-qualified path from a `qualified_name` node.
///
/// Structure: `namespace_name_as_prefix` + `name`
/// where `namespace_name_as_prefix` contains a `namespace_name` child.
///
/// e.g. `App\Models\User`:
///   qualified_name
///     namespace_name_as_prefix
///       namespace_name → "App\Models"
///       \
///     name → "User"
fn extract_qualified_name(node: &Node, source: &[u8]) -> Option<String> {
    let mut prefix = String::new();
    let mut name = String::new();

    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        match child.kind() {
            "namespace_name_as_prefix" => {
                prefix = extract_namespace_name_as_prefix(&child, source).unwrap_or_default();
            }
            "name" => {
                name = extract_text(&child, source).unwrap_or_default();
            }
            _ => {}
        }
    }

    if name.is_empty() {
        return None;
    }
    if prefix.is_empty() {
        Some(name)
    } else {
        Some(format!("{}\\{}", prefix, name))
    }
}

/// Extract the namespace prefix from a `namespace_name_as_prefix` node.
///
/// The prefix is the `namespace_name` child's text (e.g. `App\Models`).
fn extract_namespace_name_as_prefix(node: &Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        if child.kind() == "namespace_name" {
            return extract_namespace_name(&child, source);
        }
    }
    None
}

/// Reconstruct a `namespace_name` node's text as a backslash-separated string.
///
/// The grammar represents `App\Models` as:
///   namespace_name
///     name "App"
///     \ "\\"
///     name "Models"
///
/// Walking children and joining non-`\` tokens produces the clean path.
fn extract_namespace_name(node: &Node, source: &[u8]) -> Option<String> {
    // The text of the node itself is the most reliable source.
    extract_text(node, source)
}

/// Expand a `namespace_use_group` node into individual imports.
///
/// `use App\Services\{Auth, Logger}` → `App\Services\Auth`, `App\Services\Logger`
///
/// Each `namespace_use_group_clause` contains a `namespace_name` child.
fn extract_group_use(
    node: &Node,
    source: &[u8],
    file_path: &str,
    line: usize,
    base_prefix: &str,
    out: &mut Vec<RawImport>,
) {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        if child.kind() == "namespace_use_group_clause" {
            if let Some(suffix) = extract_group_clause_name(&child, source) {
                let path = if base_prefix.is_empty() {
                    suffix
                } else {
                    format!("{}\\{}", base_prefix, suffix)
                };
                out.push(RawImport {
                    path,
                    line,
                    file: file_path.to_string(),
                    kind: ImportKind::Import,
                    named_imports: vec![],
                });
            }
        }
    }
}

/// Extract the name from a `namespace_use_group_clause` node.
///
/// Clause contains optional `function`/`const` keyword, then a `namespace_name`.
fn extract_group_clause_name(node: &Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        if child.kind() == "namespace_name" {
            return extract_namespace_name(&child, source);
        }
    }
    None
}

/// Parse PHP source code and extract named entities for naming convention checks.
///
/// Extracts:
/// - `Symbol`: class, interface, trait, enum declarations; function declarations
/// - `Comment`: `comment` nodes (both `//` and `/* */` styles)
pub fn parse_php_names(source: &str, file_path: &str) -> Vec<RawName> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_php::language_php())
        .expect("Failed to load PHP grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut names = Vec::new();
    collect_php_names(root, source.as_bytes(), file_path, &mut names);
    names
}

fn collect_php_names(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawName>) {
    let kind = node.kind();
    let line = node.start_position().row + 1;

    match kind {
        // Symbols: class, interface, trait, enum declarations
        "class_declaration"
        | "interface_declaration"
        | "trait_declaration"
        | "enum_declaration" => {
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
        // Symbols: top-level and method function declarations
        "function_definition" | "method_declaration" => {
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
        _ => {}
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_php_names(child, source, file_path, out);
        }
    }
}

fn extract_text(node: &Node, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::name::NameKind;

    fn parse(src: &str) -> Vec<RawImport> {
        parse_php_imports(src, "test.php")
    }

    // ------------------------------------------------------------------
    // parse_php_imports
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_php_simple_use() {
        let imports = parse("<?php\nuse App\\Models\\User;\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "App\\Models\\User");
        assert_eq!(imports[0].line, 2);
        assert_eq!(imports[0].file, "test.php");
    }

    #[test]
    fn test_parse_php_aliased_use() {
        // Alias is ignored — we record the original class path.
        let imports = parse("<?php\nuse App\\Models\\User as UserModel;\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "App\\Models\\User");
    }

    #[test]
    fn test_parse_php_group_use() {
        // `use App\Services\{Auth, Logger};` expands to two imports.
        let imports = parse("<?php\nuse App\\Services\\{Auth, Logger};\n");
        assert_eq!(imports.len(), 2);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(
            paths.contains(&"App\\Services\\Auth"),
            "expected App\\Services\\Auth, got {:?}",
            paths
        );
        assert!(
            paths.contains(&"App\\Services\\Logger"),
            "expected App\\Services\\Logger, got {:?}",
            paths
        );
    }

    #[test]
    fn test_parse_php_function_use() {
        let imports = parse("<?php\nuse function App\\Helpers\\format_date;\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "App\\Helpers\\format_date");
    }

    #[test]
    fn test_parse_php_const_use() {
        let imports = parse("<?php\nuse const App\\Config\\MAX_RETRIES;\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "App\\Config\\MAX_RETRIES");
    }

    #[test]
    fn test_parse_php_multiple_use() {
        let src = "<?php\nuse App\\Models\\User;\nuse App\\Services\\Auth;\nuse Illuminate\\Http\\Request;\n";
        let imports = parse(src);
        assert_eq!(imports.len(), 3);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"App\\Models\\User"));
        assert!(paths.contains(&"App\\Services\\Auth"));
        assert!(paths.contains(&"Illuminate\\Http\\Request"));
    }

    #[test]
    fn test_parse_php_no_imports() {
        let imports = parse("<?php\nclass Foo {}\n");
        assert!(imports.is_empty());
    }

    // ------------------------------------------------------------------
    // parse_php_names
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_php_names_class() {
        let source = "<?php\nclass UserController {}\n";
        let names = parse_php_names(source, "test.php");
        let found = names
            .iter()
            .find(|n| n.name == "UserController" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "class UserController should be detected as Symbol, got: {:#?}",
            names
        );
        assert_eq!(found.unwrap().line, 2);
    }

    #[test]
    fn test_parse_php_names_function() {
        let source = "<?php\nfunction getUserById() {}\n";
        let names = parse_php_names(source, "test.php");
        let found = names
            .iter()
            .find(|n| n.name == "getUserById" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "function getUserById should be detected as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_parse_php_names_comment() {
        let source = "<?php\n// connect to db\n$x = 1;\n";
        let names = parse_php_names(source, "test.php");
        let found = names
            .iter()
            .find(|n| n.kind == NameKind::Comment && n.name.contains("connect to db"));
        assert!(
            found.is_some(),
            "comment with 'connect to db' should be detected, got: {:#?}",
            names
        );
        assert_eq!(found.unwrap().line, 2);
    }
}
