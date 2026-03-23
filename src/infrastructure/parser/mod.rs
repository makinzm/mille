pub mod c;
pub mod go;
pub mod java;
pub mod kotlin;
pub mod php;
pub mod python;
pub mod rust;
pub mod typescript;

use self::c::CParser;
use self::go::GoParser;
use self::java::JavaParser;
use self::kotlin::KotlinParser;
use self::php::PhpParser;
use self::python::PythonParser;
use self::rust::RustParser;
use self::typescript::TypeScriptParser;
use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;
use crate::domain::entity::name::{NameKind, ParsedNames, RawName};
use crate::domain::repository::parser::Parser;

/// Partition a flat `Vec<RawName>` into a `ParsedNames` struct grouped by kind.
pub(crate) fn partition_names(names: Vec<RawName>) -> ParsedNames {
    let mut symbols = Vec::new();
    let mut variables = Vec::new();
    let mut comments = Vec::new();
    let mut string_literals = Vec::new();

    for name in names {
        match name.kind {
            NameKind::Symbol => symbols.push(name),
            NameKind::Variable => variables.push(name),
            NameKind::Comment => comments.push(name),
            NameKind::StringLiteral => string_literals.push(name),
            NameKind::File => {} // File-level checks are handled by the caller
        }
    }

    ParsedNames {
        symbols,
        variables,
        comments,
        string_literals,
    }
}

/// Strip surrounding quotes/delimiters from a string literal node text.
///
/// Handles `"..."`, `'...'`, `r#"..."#`, `` `...` ``, `"""..."""`, etc.
pub(crate) fn strip_string_delimiters(text: &str) -> String {
    let t = text.trim();
    // Rust raw string: r#"..."# or r##"..."##
    if t.starts_with('r') && t.contains('"') {
        if let Some(start) = t.find('"') {
            let after_open = start + 1;
            if let Some(end) = t.rfind('"') {
                if end > after_open {
                    return t[after_open..end].to_string();
                }
            }
        }
        return t.to_string();
    }
    // Triple-quoted strings (Python, Kotlin)
    if t.starts_with("\"\"\"") && t.ends_with("\"\"\"") && t.len() >= 6 {
        return t[3..t.len() - 3].to_string();
    }
    if t.starts_with("'''") && t.ends_with("'''") && t.len() >= 6 {
        return t[3..t.len() - 3].to_string();
    }
    // Single/double quoted
    if ((t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')))
        && t.len() >= 2
    {
        return t[1..t.len() - 1].to_string();
    }
    // Backtick (Go raw strings, JS template literals)
    if t.starts_with('`') && t.ends_with('`') && t.len() >= 2 {
        return t[1..t.len() - 1].to_string();
    }
    t.to_string()
}

/// Map a file extension to the language name used in `mille.toml`.
pub fn ext_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("rust"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "go" => Some("go"),
        "py" => Some("python"),
        "java" => Some("java"),
        "kt" => Some("kotlin"),
        "php" => Some("php"),
        "c" | "h" => Some("c"),
        _ => None,
    }
}

/// Dispatches to the appropriate parser based on file extension.
pub struct DispatchingParser {
    c: CParser,
    rust: RustParser,
    go: GoParser,
    python: PythonParser,
    typescript: TypeScriptParser,
    java: JavaParser,
    kotlin: KotlinParser,
    php: PhpParser,
}

impl DispatchingParser {
    pub fn new() -> Self {
        DispatchingParser {
            c: CParser,
            rust: RustParser,
            go: GoParser,
            python: PythonParser,
            typescript: TypeScriptParser,
            java: JavaParser,
            kotlin: KotlinParser,
            php: PhpParser,
        }
    }
}

impl Default for DispatchingParser {
    fn default() -> Self {
        Self::new()
    }
}

fn is_c(file_path: &str) -> bool {
    file_path.ends_with(".c") || file_path.ends_with(".h")
}

fn is_ts_js(file_path: &str) -> bool {
    file_path.ends_with(".ts")
        || file_path.ends_with(".tsx")
        || file_path.ends_with(".js")
        || file_path.ends_with(".jsx")
}

impl Parser for DispatchingParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        if is_c(file_path) {
            self.c.parse_imports(source, file_path)
        } else if file_path.ends_with(".go") {
            self.go.parse_imports(source, file_path)
        } else if file_path.ends_with(".py") {
            self.python.parse_imports(source, file_path)
        } else if is_ts_js(file_path) {
            self.typescript.parse_imports(source, file_path)
        } else if file_path.ends_with(".java") {
            self.java.parse_imports(source, file_path)
        } else if file_path.ends_with(".kt") {
            self.kotlin.parse_imports(source, file_path)
        } else if file_path.ends_with(".php") {
            self.php.parse_imports(source, file_path)
        } else {
            self.rust.parse_imports(source, file_path)
        }
    }

    fn parse_call_exprs(&self, source: &str, file_path: &str) -> Vec<RawCallExpr> {
        if is_c(file_path) {
            self.c.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".go") {
            self.go.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".py") {
            self.python.parse_call_exprs(source, file_path)
        } else if is_ts_js(file_path) {
            self.typescript.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".java") {
            self.java.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".kt") {
            self.kotlin.parse_call_exprs(source, file_path)
        } else if file_path.ends_with(".php") {
            self.php.parse_call_exprs(source, file_path)
        } else {
            self.rust.parse_call_exprs(source, file_path)
        }
    }

    fn parse_names(&self, source: &str, file_path: &str) -> ParsedNames {
        if is_c(file_path) {
            self.c.parse_names(source, file_path)
        } else if file_path.ends_with(".go") {
            self.go.parse_names(source, file_path)
        } else if file_path.ends_with(".py") {
            self.python.parse_names(source, file_path)
        } else if is_ts_js(file_path) {
            self.typescript.parse_names(source, file_path)
        } else if file_path.ends_with(".java") {
            self.java.parse_names(source, file_path)
        } else if file_path.ends_with(".kt") {
            self.kotlin.parse_names(source, file_path)
        } else if file_path.ends_with(".php") {
            self.php.parse_names(source, file_path)
        } else {
            self.rust.parse_names(source, file_path)
        }
    }
}
