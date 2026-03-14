use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Java source files.
pub struct JavaParser;

impl Parser for JavaParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_java_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // Java call expression analysis is not yet implemented.
        // Return an empty Vec consistent with the Go parser approach.
        vec![]
    }
}

/// Parse Java source code and extract all `import` declarations.
pub fn parse_java_imports(_source: &str, _file_path: &str) -> Vec<RawImport> {
    todo!("JavaParser::parse_java_imports not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::ImportKind;

    #[test]
    fn test_parse_java_single_import() {
        let source = "package com.example;\n\nimport java.util.List;\n\npublic class Foo {}\n";
        let imports = parse_java_imports(source, "Foo.java");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "java.util.List");
        assert_eq!(imports[0].kind, ImportKind::Import);
        assert_eq!(imports[0].line, 3);
        assert_eq!(imports[0].file, "Foo.java");
    }

    #[test]
    fn test_parse_java_static_import() {
        let source =
            "package com.example;\n\nimport static com.example.Foo.bar;\n\npublic class Baz {}\n";
        let imports = parse_java_imports(source, "Baz.java");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "com.example.Foo.bar");
        assert_eq!(imports[0].kind, ImportKind::Import);
    }

    #[test]
    fn test_parse_java_multiple_imports() {
        let source = "package com.example;\n\nimport java.util.List;\nimport java.util.Map;\nimport com.example.domain.User;\n\npublic class Foo {}\n";
        let imports = parse_java_imports(source, "Foo.java");
        assert_eq!(imports.len(), 3);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"java.util.List"));
        assert!(paths.contains(&"java.util.Map"));
        assert!(paths.contains(&"com.example.domain.User"));
    }

    #[test]
    fn test_parse_java_no_imports() {
        let source = "package com.example;\n\npublic class Foo {}\n";
        let imports = parse_java_imports(source, "Foo.java");
        assert!(imports.is_empty());
    }

    #[test]
    fn test_parse_java_call_exprs_empty() {
        let parser = JavaParser;
        let source = "package com.example;\n\npublic class Foo { public void bar() {} }\n";
        let calls = parser.parse_call_exprs(source, "Foo.java");
        assert!(calls.is_empty());
    }
}
