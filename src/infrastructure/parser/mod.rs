pub mod go;
pub mod python;
pub mod rust;

use self::go::GoParser;
use self::python::PythonParser;
use self::rust::RustParser;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;
use crate::domain::repository::parser::Parser;

/// Dispatches to the appropriate parser based on file extension.
pub struct DispatchingParser {
    rust: RustParser,
    go: GoParser,
    python: PythonParser,
}

impl DispatchingParser {
    pub fn new() -> Self {
        DispatchingParser {
            rust: RustParser,
            go: GoParser,
            python: PythonParser,
        }
    }
}

impl Default for DispatchingParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for DispatchingParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        if file_path.ends_with(".go") {
            self.go.parse_imports(source, file_path)
        } else if file_path.ends_with(".py") {
            self.python.parse_imports(source, file_path)
        } else {
            self.rust.parse_imports(source, file_path)
        }
    }

    fn parse_call_exprs(&self, source: &str, file_path: &str) -> Vec<RawCallExpr> {
        if file_path.ends_with(".go") {
            self.go.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".py") {
            self.python.parse_call_exprs(source, file_path)
        } else {
            self.rust.parse_call_exprs(source, file_path)
        }
    }
}
