use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;
use crate::domain::entity::name::RawName;

/// Port for extracting imports, call expressions, and names from source code.
/// Concrete implementations live in `infrastructure::parser`.
pub trait Parser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport>;
    fn parse_call_exprs(&self, source: &str, file_path: &str) -> Vec<RawCallExpr>;
    /// Extract named entities (symbols, variables, comments) for naming convention checks.
    /// File-level checks (NameKind::File) are handled by the caller using the file path directly.
    fn parse_names(&self, source: &str, file_path: &str) -> Vec<RawName>;
}
