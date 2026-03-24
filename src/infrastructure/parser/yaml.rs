use tree_sitter::Node;

use super::partition_names;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;
use crate::domain::entity::name::{NameKind, ParsedNames, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for YAML files.
///
/// YAML is a naming-only language — it has no imports or method calls.
/// Only `parse_names()` produces meaningful results.
pub struct YamlParser;

impl Parser for YamlParser {
    fn parse_imports(&self, _source: &str, _file_path: &str) -> Vec<RawImport> {
        vec![]
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        vec![]
    }

    fn parse_names(&self, source: &str, file_path: &str) -> ParsedNames {
        parse_yaml_names(source, file_path)
    }
}

/// Parse YAML source and extract named entities for naming convention checks.
///
/// Extracts:
/// - `Symbol`: mapping keys (block_mapping_pair key, flow_pair key)
/// - `StringLiteral`: scalar values (plain_scalar, double_quote_scalar, single_quote_scalar
///   in value position)
/// - `Comment`: comment nodes
pub fn parse_yaml_names(source: &str, file_path: &str) -> ParsedNames {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_yaml::language())
        .expect("Failed to load YAML grammar");

    let tree = parser.parse(source, None).expect("Failed to parse source");
    let root = tree.root_node();

    let mut names = Vec::new();
    collect_yaml_names(root, source.as_bytes(), file_path, &mut names, false);
    partition_names(names)
}

/// Recursively collect names from a YAML AST.
///
/// `in_key` tracks whether we are inside a mapping key (for Symbol classification).
fn collect_yaml_names(
    node: Node,
    source: &[u8],
    file_path: &str,
    out: &mut Vec<RawName>,
    in_key: bool,
) {
    let kind = node.kind();
    let line = node.start_position().row + 1;

    match kind {
        // Mapping pair: key → Symbol, value → StringLiteral
        "block_mapping_pair" | "flow_pair" => {
            if let Some(key_node) = node.child_by_field_name("key") {
                collect_yaml_names(key_node, source, file_path, out, true);
            }
            if let Some(value_node) = node.child_by_field_name("value") {
                collect_yaml_names(value_node, source, file_path, out, false);
            }
            return;
        }

        // Scalar nodes: classify based on whether they appear as key or value
        "plain_scalar" | "string_scalar" | "boolean_scalar" | "integer_scalar" | "float_scalar"
        | "null_scalar" | "timestamp_scalar" => {
            let text = node.utf8_text(source).unwrap_or("").to_string();
            if !text.is_empty() {
                let name_kind = if in_key {
                    NameKind::Symbol
                } else {
                    NameKind::StringLiteral
                };
                out.push(RawName {
                    name: text,
                    line,
                    kind: name_kind,
                    file: file_path.to_string(),
                });
            }
            return;
        }

        // Quoted scalars: strip delimiters for the name, classify by position
        "double_quote_scalar" | "single_quote_scalar" => {
            let raw = node.utf8_text(source).unwrap_or("");
            let content = super::strip_string_delimiters(raw);
            if !content.is_empty() {
                let name_kind = if in_key {
                    NameKind::Symbol
                } else {
                    NameKind::StringLiteral
                };
                out.push(RawName {
                    name: content,
                    line,
                    kind: name_kind,
                    file: file_path.to_string(),
                });
            }
            return;
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
            return;
        }

        _ => {}
    }

    // Recurse into children, preserving in_key context
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_yaml_names(child, source, file_path, out, in_key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::name::NameKind;

    #[test]
    fn test_yaml_mapping_key_is_symbol() {
        let source = "aws_region: us-east-1\n";
        let names = parse_yaml_names(source, "test.yaml").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "aws_region" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "mapping key 'aws_region' should be detected as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_yaml_plain_scalar_value_is_string_literal() {
        let source = "region: us-east-1\n";
        let names = parse_yaml_names(source, "test.yaml").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "us-east-1" && n.kind == NameKind::StringLiteral);
        assert!(
            found.is_some(),
            "plain scalar value 'us-east-1' should be StringLiteral, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_yaml_quoted_value_is_string_literal() {
        let source = "image: \"my-app:latest\"\n";
        let names = parse_yaml_names(source, "test.yaml").into_all();
        let found = names
            .iter()
            .find(|n| n.name == "my-app:latest" && n.kind == NameKind::StringLiteral);
        assert!(
            found.is_some(),
            "quoted value 'my-app:latest' should be StringLiteral, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_yaml_comment() {
        let source = "# This is a comment\nkey: value\n";
        let names = parse_yaml_names(source, "test.yaml").into_all();
        let found = names
            .iter()
            .find(|n| n.kind == NameKind::Comment && n.name.contains("This is a comment"));
        assert!(
            found.is_some(),
            "comment should be detected, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_yaml_nested_keys() {
        let source = "spec:\n  replicas: 3\n  template:\n    name: app\n";
        let names = parse_yaml_names(source, "test.yaml").into_all();
        let symbols: Vec<_> = names
            .iter()
            .filter(|n| n.kind == NameKind::Symbol)
            .map(|n| n.name.as_str())
            .collect();
        assert!(
            symbols.contains(&"spec"),
            "nested key 'spec' should be Symbol, got: {:?}",
            symbols
        );
        assert!(
            symbols.contains(&"replicas"),
            "nested key 'replicas' should be Symbol, got: {:?}",
            symbols
        );
        assert!(
            symbols.contains(&"template"),
            "nested key 'template' should be Symbol, got: {:?}",
            symbols
        );
        assert!(
            symbols.contains(&"name"),
            "nested key 'name' should be Symbol, got: {:?}",
            symbols
        );
    }

    #[test]
    fn test_yaml_list_values() {
        let source = "ports:\n  - 80\n  - 443\n";
        let names = parse_yaml_names(source, "test.yaml").into_all();
        let values: Vec<_> = names
            .iter()
            .filter(|n| n.kind == NameKind::StringLiteral)
            .map(|n| n.name.as_str())
            .collect();
        assert!(
            values.contains(&"80"),
            "list value '80' should be StringLiteral, got: {:?}",
            values
        );
        assert!(
            values.contains(&"443"),
            "list value '443' should be StringLiteral, got: {:?}",
            values
        );
    }
}
