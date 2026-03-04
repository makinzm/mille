use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;

/// Port for extracting imports and call expressions from source code.
/// Concrete implementations live in `infrastructure::parser`.
pub trait Parser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport>;
    fn parse_call_exprs(&self, source: &str, file_path: &str) -> Vec<RawCallExpr>;
}
