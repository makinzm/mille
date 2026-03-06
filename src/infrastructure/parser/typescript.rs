use tree_sitter::Node;

use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for TypeScript and JavaScript source files.
/// Handles: .ts, .tsx, .js, .jsx
pub struct TypeScriptParser;

impl Parser for TypeScriptParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_ts_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // TypeScript/JavaScript call-expression analysis is not yet implemented.
        vec![]
    }
}

/// Parse TypeScript/JavaScript source code and extract all static import statements.
///
/// Handles:
/// - `import X from "path"`
/// - `import { X } from "path"`
/// - `import * as X from "path"`
/// - `import "path"` (side-effect only)
pub fn parse_ts_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    let language = select_language(file_path);
    parser
        .set_language(&language)
        .expect("Failed to load TypeScript/JavaScript grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_ts_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

/// Select the appropriate tree-sitter grammar based on file extension.
fn select_language(file_path: &str) -> tree_sitter::Language {
    if file_path.ends_with(".tsx") {
        tree_sitter_typescript::language_tsx()
    } else if file_path.ends_with(".ts") {
        tree_sitter_typescript::language_typescript()
    } else {
        // .js and .jsx both use the JavaScript grammar (JSX is supported by default)
        tree_sitter_javascript::language()
    }
}

fn collect_ts_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    if node.kind() == "import_statement" {
        let line = node.start_position().row + 1;
        if let Some(path) = extract_import_source(&node, source) {
            let named = extract_ts_named_imports(&node, source);
            out.push(RawImport {
                path,
                line,
                file: file_path.to_string(),
                kind: ImportKind::Import,
                named_imports: named,
            });
        }
        // Do not recurse into import_statement children to avoid double-counting.
        return;
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_ts_imports(child, source, file_path, out);
        }
    }
}

/// Extract the module specifier string from an `import_statement` node.
///
/// The grammar structure for `import X from "path"` is:
/// ```text
/// (import_statement
///   "import"
///   (import_clause ...)
///   "from"
///   (string "\"path\""))
/// ```
/// For side-effect imports `import "path"`:
/// ```text
/// (import_statement
///   "import"
///   (string "\"path\""))
/// ```
fn extract_import_source(node: &Node, source: &[u8]) -> Option<String> {
    for i in 0..node.child_count() {
        let child = node.child(i)?;
        if child.kind() == "string" {
            return extract_string_content(&child, source);
        }
    }
    None
}

/// Extract the named symbols brought into scope by a TS/JS import statement.
///
/// - `import { User, Admin } from "..."` → `["User", "Admin"]`
/// - `import User from "..."` → `["User"]`  (default import)
/// - `import * as ns from "..."` → `[]`  (namespace import — no specific name)
/// - `import "..."` → `[]`  (side-effect only)
fn extract_ts_named_imports(node: &Node, source: &[u8]) -> Vec<String> {
    for i in 0..node.child_count() {
        let Some(child) = node.child(i) else { continue };
        if child.kind() != "import_clause" {
            continue;
        }
        return collect_import_clause_names(&child, source);
    }
    vec![]
}

fn collect_import_clause_names(clause: &Node, source: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    for i in 0..clause.child_count() {
        let Some(child) = clause.child(i) else { continue };
        match child.kind() {
            // Default import: `import User from "..."`
            "identifier" => {
                if let Ok(text) = child.utf8_text(source) {
                    names.push(text.to_string());
                }
            }
            // Named imports: `import { User, Admin } from "..."`
            "named_imports" => {
                for j in 0..child.child_count() {
                    let Some(spec) = child.child(j) else { continue };
                    if spec.kind() == "import_specifier" {
                        // The "name" field is the local (imported) name.
                        // For `import { Foo as Bar }`, tree-sitter uses "name" for Foo and "alias" for Bar.
                        // We want the original name Foo for type matching.
                        if let Some(name_node) = spec.child_by_field_name("name") {
                            if let Ok(text) = name_node.utf8_text(source) {
                                names.push(text.to_string());
                            }
                        }
                    }
                }
            }
            // Namespace import: `import * as ns from "..."` — no specific type name
            "namespace_import" => {}
            _ => {}
        }
    }
    names
}

/// Extract the content of a string literal node, stripping surrounding quotes.
fn extract_string_content(node: &Node, source: &[u8]) -> Option<String> {
    let raw = node.utf8_text(source).ok()?;
    // Strip surrounding quotes (" or ')
    let inner = raw.trim_matches(|c| c == '"' || c == '\'');
    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ts(src: &str) -> Vec<RawImport> {
        parse_ts_imports(src, "test.ts")
    }

    fn parse_js(src: &str) -> Vec<RawImport> {
        parse_ts_imports(src, "test.js")
    }

    #[test]
    fn test_ts_default_import() {
        let imports = parse_ts("import User from \"../domain/user\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "../domain/user");
        assert_eq!(imports[0].line, 1);
    }

    #[test]
    fn test_ts_named_import() {
        let imports = parse_ts("import { User } from \"../domain/user\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "../domain/user");
    }

    #[test]
    fn test_ts_namespace_import() {
        let imports = parse_ts("import * as fs from \"node:fs\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "node:fs");
    }

    #[test]
    fn test_ts_side_effect_import() {
        let imports = parse_ts("import \"./polyfills\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "./polyfills");
    }

    #[test]
    fn test_ts_relative_import() {
        let imports = parse_ts("import { User } from \"./user\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "./user");
    }

    #[test]
    fn test_ts_external_package() {
        let imports = parse_ts("import { validate } from \"some-lib\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "some-lib");
    }

    #[test]
    fn test_ts_multiple_imports() {
        let src =
            "import { User } from \"../domain/user\";\nimport { validate } from \"some-lib\";\n";
        let imports = parse_ts(src);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].path, "../domain/user");
        assert_eq!(imports[1].path, "some-lib");
    }

    #[test]
    fn test_ts_no_imports() {
        let imports = parse_ts("const x = 1;\nconsole.log(x);\n");
        assert!(imports.is_empty());
    }

    #[test]
    fn test_js_default_import() {
        let imports = parse_js("import User from \"../domain/user\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "../domain/user");
    }

    #[test]
    fn test_js_named_import() {
        let imports = parse_js("import { User } from \"../domain/user\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "../domain/user");
    }

    #[test]
    fn test_js_external_package() {
        let imports = parse_js("import fs from \"node:fs\";\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "node:fs");
    }

    #[test]
    fn test_tsx_import() {
        let src = "import React from \"react\";\nimport { User } from \"../domain/user\";\n";
        let imports = parse_ts_imports(src, "test.tsx");
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].path, "react");
        assert_eq!(imports[1].path, "../domain/user");
    }
}
