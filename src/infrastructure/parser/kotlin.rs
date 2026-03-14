use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Kotlin source files.
pub struct KotlinParser;

impl Parser for KotlinParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_kotlin_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        vec![]
    }
}

/// Parse Kotlin source code and extract all `import` declarations.
pub fn parse_kotlin_imports(_source: &str, _file_path: &str) -> Vec<RawImport> {
    todo!("KotlinParser not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::ImportKind;

    #[test]
    fn test_parse_kotlin_single_import() {
        let source = "package com.example\n\nimport com.example.usecase.UserService\n\nclass Foo\n";
        let imports = parse_kotlin_imports(source, "Foo.kt");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "com.example.usecase.UserService");
        assert_eq!(imports[0].kind, ImportKind::Import);
        assert_eq!(imports[0].line, 3);
        assert_eq!(imports[0].file, "Foo.kt");
    }

    #[test]
    fn test_parse_kotlin_wildcard_import() {
        let source = "package com.example\n\nimport com.example.domain.*\n\nclass Foo\n";
        let imports = parse_kotlin_imports(source, "Foo.kt");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "com.example.domain");
        assert_eq!(imports[0].kind, ImportKind::Import);
    }

    #[test]
    fn test_parse_kotlin_multiple_imports() {
        let source = "package com.example\n\nimport com.example.domain.User\nimport com.example.usecase.UserService\nimport kotlin.collections.List\n\nclass Foo\n";
        let imports = parse_kotlin_imports(source, "Foo.kt");
        assert_eq!(imports.len(), 3);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"com.example.domain.User"));
        assert!(paths.contains(&"com.example.usecase.UserService"));
        assert!(paths.contains(&"kotlin.collections.List"));
    }

    #[test]
    fn test_parse_kotlin_no_imports() {
        let source = "package com.example\n\nclass Foo\n";
        let imports = parse_kotlin_imports(source, "Foo.kt");
        assert!(imports.is_empty());
    }

    #[test]
    fn test_parse_kotlin_call_exprs_empty() {
        let parser = KotlinParser;
        let source = "package com.example\n\nclass Foo { fun bar() {} }\n";
        let calls = parser.parse_call_exprs(source, "Foo.kt");
        assert!(calls.is_empty());
    }
}
