pub mod go;
pub mod java;
pub mod kotlin;
pub mod python;
pub mod rust;
pub mod typescript;

use self::go::GoParser;
use self::java::JavaParser;
use self::kotlin::KotlinParser;
use self::python::PythonParser;
use self::rust::RustParser;
use self::typescript::TypeScriptParser;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;
use crate::domain::entity::name::RawName;
use crate::domain::repository::parser::Parser;

/// Dispatches to the appropriate parser based on file extension.
pub struct DispatchingParser {
    rust: RustParser,
    go: GoParser,
    python: PythonParser,
    typescript: TypeScriptParser,
    java: JavaParser,
    kotlin: KotlinParser,
}

impl DispatchingParser {
    pub fn new() -> Self {
        DispatchingParser {
            rust: RustParser,
            go: GoParser,
            python: PythonParser,
            typescript: TypeScriptParser,
            java: JavaParser,
            kotlin: KotlinParser,
        }
    }
}

impl Default for DispatchingParser {
    fn default() -> Self {
        Self::new()
    }
}

fn is_ts_js(file_path: &str) -> bool {
    file_path.ends_with(".ts")
        || file_path.ends_with(".tsx")
        || file_path.ends_with(".js")
        || file_path.ends_with(".jsx")
}

impl Parser for DispatchingParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        if file_path.ends_with(".go") {
            self.go.parse_imports(source, file_path)
        } else if file_path.ends_with(".py") {
            self.python.parse_imports(source, file_path)
        } else if is_ts_js(file_path) {
            self.typescript.parse_imports(source, file_path)
        } else if file_path.ends_with(".java") {
            self.java.parse_imports(source, file_path)
        } else if file_path.ends_with(".kt") {
            self.kotlin.parse_imports(source, file_path)
        } else {
            self.rust.parse_imports(source, file_path)
        }
    }

    fn parse_call_exprs(&self, source: &str, file_path: &str) -> Vec<RawCallExpr> {
        if file_path.ends_with(".go") {
            self.go.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".py") {
            self.python.parse_call_exprs(source, file_path)
        } else if is_ts_js(file_path) {
            self.typescript.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".java") {
            self.java.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".kt") {
            self.kotlin.parse_call_exprs(source, file_path)
        } else {
            self.rust.parse_call_exprs(source, file_path)
        }
    }

    fn parse_names(&self, source: &str, file_path: &str) -> Vec<RawName> {
        if file_path.ends_with(".go") {
            self.go.parse_names(source, file_path)
        } else if file_path.ends_with(".py") {
            self.python.parse_names(source, file_path)
        } else if is_ts_js(file_path) {
            self.typescript.parse_names(source, file_path)
        } else if file_path.ends_with(".java") {
            self.java.parse_names(source, file_path)
        } else if file_path.ends_with(".kt") {
            self.kotlin.parse_names(source, file_path)
        } else {
            self.rust.parse_names(source, file_path)
        }
    }
}
