use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for TypeScript and JavaScript source files.
/// Handles: .ts, .tsx, .js, .jsx
pub struct TypeScriptParser;

impl Parser for TypeScriptParser {
    fn parse_imports(&self, _source: &str, _file_path: &str) -> Vec<RawImport> {
        todo!("TypeScriptParser::parse_imports not implemented")
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        vec![]
    }
}
