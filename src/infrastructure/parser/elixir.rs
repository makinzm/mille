use tree_sitter::Node;

use super::partition_names;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::name::{NameKind, ParsedNames, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Elixir source files.
pub struct ElixirParser;

impl Parser for ElixirParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_elixir_imports(source, file_path)
    }

    fn parse_call_exprs(&self, source: &str, file_path: &str) -> Vec<RawCallExpr> {
        parse_elixir_call_exprs(source, file_path)
    }

    fn parse_names(&self, source: &str, file_path: &str) -> ParsedNames {
        parse_elixir_names(source, file_path)
    }
}

/// Parse Elixir source code and extract named entities for naming convention checks.
pub fn parse_elixir_names(source: &str, file_path: &str) -> ParsedNames {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_elixir::language())
        .expect("Failed to load Elixir grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut names = Vec::new();
    collect_elixir_names(root, source.as_bytes(), file_path, &mut names);
    partition_names(names)
}

fn collect_elixir_names(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawName>) {
    let kind = node.kind();
    let line = node.start_position().row + 1;

    match kind {
        // defmodule, def, defp — top-level call with identifier as first child
        "call" => {
            if let Some(id_node) = node.child(0) {
                if id_node.kind() == "identifier" {
                    let keyword = id_node.utf8_text(source).unwrap_or("");
                    match keyword {
                        "defmodule" | "def" | "defp" | "defmacro" | "defmacrop" => {
                            // Extract the name from arguments.
                            // `arguments` children include `(` `)` `,` as anonymous nodes;
                            // use `named_child(0)` to skip them and get the first named node.
                            if let Some(args) = node.child(1) {
                                if args.kind() == "arguments" {
                                    if let Some(first_named) = args.named_child(0) {
                                        // `def name(args)` → first_named is a `call` node:
                                        //   call → identifier("name"), arguments(...)
                                        // `defmodule Foo.Bar` → first_named is an `alias` node
                                        let name = if first_named.kind() == "call" {
                                            first_named
                                                .named_child(0)
                                                .filter(|n| n.kind() == "identifier")
                                                .and_then(|n| n.utf8_text(source).ok())
                                                .unwrap_or("")
                                        } else {
                                            first_named.utf8_text(source).unwrap_or("")
                                        };
                                        if !name.is_empty() {
                                            out.push(RawName {
                                                name: name.to_string(),
                                                line,
                                                kind: NameKind::Symbol,
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
        // String literals
        "string" | "charlist" => {
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
            collect_elixir_names(child, source, file_path, out);
        }
    }
}

/// Parse Elixir source code and extract all `alias`, `import`, `require`, `use` statements.
///
/// Elixir AST structure (from tree-sitter-elixir):
/// - `alias MyApp.Domain.User` → call(identifier("alias"), arguments(alias("MyApp.Domain.User")))
/// - `alias MyApp.Domain.User, as: U` → call(identifier("alias"), arguments(alias("MyApp.Domain.User"), keywords(...)))
/// - `import Enum` → call(identifier("import"), arguments(alias("Enum")))
/// - `require Logger` → call(identifier("require"), arguments(alias("Logger")))
/// - `use Ecto.Schema` → call(identifier("use"), arguments(alias("Ecto.Schema")))
pub fn parse_elixir_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_elixir::language())
        .expect("Failed to load Elixir grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut imports = Vec::new();
    collect_elixir_imports(root, source.as_bytes(), file_path, &mut imports);
    imports
}

/// Parse Elixir source code and extract call expressions.
///
/// Always returns an empty list. Elixir's dynamic dispatch makes static
/// receiver-type inference unreliable, so `allow_call_patterns` is not
/// supported for Elixir.
pub fn parse_elixir_call_exprs(_source: &str, _file_path: &str) -> Vec<RawCallExpr> {
    Vec::new()
}

fn collect_elixir_imports(node: Node, source: &[u8], file_path: &str, out: &mut Vec<RawImport>) {
    if node.kind() == "call" {
        if let Some(id_node) = node.child(0) {
            if id_node.kind() == "identifier" {
                let keyword = id_node.utf8_text(source).unwrap_or("");
                match keyword {
                    "alias" | "import" | "require" | "use" => {
                        let line = node.start_position().row + 1;
                        if let Some(path) = extract_first_alias_path(&node, source) {
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
        }
    }

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_elixir_imports(child, source, file_path, out);
        }
    }
}

/// Extract the module path from the first `alias` node in the `arguments` of a call.
///
/// For `alias MyApp.Domain.User, as: U`, returns `"MyApp.Domain.User"`.
/// The `as:` keyword option is ignored — we always use the original module path.
fn extract_first_alias_path(call_node: &Node, source: &[u8]) -> Option<String> {
    // child(1) is the `arguments` node
    let args = call_node.child(1)?;
    if args.kind() != "arguments" {
        return None;
    }

    // The first argument should be an `alias` node (module path)
    // Elixir AST: alias("MyApp.Domain.User") — the text is the full dotted module name
    for i in 0..args.child_count() {
        if let Some(arg) = args.child(i) {
            if arg.kind() == "alias" {
                return arg.utf8_text(source).ok().map(|s| s.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Vec<RawImport> {
        parse_elixir_imports(src, "test.ex")
    }

    #[test]
    fn test_parse_alias_simple() {
        let imports = parse("alias MyApp.Domain.User\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "MyApp.Domain.User");
        assert_eq!(imports[0].line, 1);
    }

    #[test]
    fn test_parse_import() {
        let imports = parse("import Enum\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "Enum");
    }

    #[test]
    fn test_parse_require() {
        let imports = parse("require Logger\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "Logger");
    }

    #[test]
    fn test_parse_use() {
        let imports = parse("use Ecto.Schema\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "Ecto.Schema");
    }

    #[test]
    fn test_parse_alias_with_as() {
        let imports = parse("alias MyApp.Domain.User, as: User\n");
        assert_eq!(imports.len(), 1, "alias with as: should still yield 1 import");
        assert_eq!(
            imports[0].path, "MyApp.Domain.User",
            "should use original module path, not the alias"
        );
    }

    #[test]
    fn test_parse_no_imports() {
        let imports = parse("defmodule MyApp.Domain.User do\n  defstruct [:id]\nend\n");
        assert!(imports.is_empty(), "no imports should yield empty list");
    }

    #[test]
    fn test_parse_multiple_imports() {
        let src = "alias MyApp.Domain.User\nimport Enum\nrequire Logger\n";
        let imports = parse(src);
        assert_eq!(imports.len(), 3);
        assert_eq!(imports[0].path, "MyApp.Domain.User");
        assert_eq!(imports[1].path, "Enum");
        assert_eq!(imports[2].path, "Logger");
    }

    // ------------------------------------------------------------------
    // parse_elixir_names
    // ------------------------------------------------------------------

    use crate::domain::entity::name::NameKind;

    #[test]
    fn test_names_defmodule() {
        let source = "defmodule MyApp.Domain.User do\nend\n";
        let names = parse_elixir_names(source, "test.ex").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "MyApp.Domain.User" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "defmodule should be detected as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_names_def_function() {
        let source = "defmodule MyApp do\n  def create(attrs) do\n    attrs\n  end\nend\n";
        let names = parse_elixir_names(source, "test.ex").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "create" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "def create(...) should extract 'create' as Symbol, not 'create(attrs)'; got: {:#?}",
            names
        );
    }

    #[test]
    fn test_names_defp_function() {
        let source = "defmodule MyApp do\n  defp helper(x) do\n    x\n  end\nend\n";
        let names = parse_elixir_names(source, "test.ex").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "helper" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "defp helper(x) should extract 'helper' as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_names_comment() {
        let source = "# connect to aws\nx = 1\n";
        let names = parse_elixir_names(source, "test.ex").into_all();
        let found = names
            .iter()
            .find(|n| n.kind == NameKind::Comment && n.name.contains("aws"));
        assert!(
            found.is_some(),
            "comment with aws should be detected, got: {:#?}",
            names
        );
    }
}
