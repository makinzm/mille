use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::RawImport;
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for Go source files.
pub struct GoParser;

impl Parser for GoParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_go_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // Go call-expression analysis is not yet implemented; return empty.
        vec![]
    }
}

/// Parse Go source code and extract all `import` declarations.
pub fn parse_go_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    todo!("GoParser not yet implemented: source={:?}, file_path={:?}", source, file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::import::ImportKind;

    #[test]
    fn test_parse_go_single_import() {
        let source = r#"package main

import "fmt"
"#;
        let imports = parse_go_imports(source, "main.go");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "fmt");
        assert_eq!(imports[0].kind, ImportKind::Import);
        assert_eq!(imports[0].line, 3);
        assert_eq!(imports[0].file, "main.go");
    }

    #[test]
    fn test_parse_go_grouped_imports() {
        let source = r#"package main

import (
    "fmt"
    "net/http"
)
"#;
        let imports = parse_go_imports(source, "server.go");
        assert_eq!(imports.len(), 2);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"fmt"));
        assert!(paths.contains(&"net/http"));
        assert!(imports.iter().all(|i| i.kind == ImportKind::Import));
    }

    #[test]
    fn test_parse_go_internal_import() {
        let source = r#"package usecase

import "github.com/example/myapp/domain"
"#;
        let imports = parse_go_imports(source, "usecase/user_usecase.go");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "github.com/example/myapp/domain");
        assert_eq!(imports[0].kind, ImportKind::Import);
    }

    #[test]
    fn test_parse_go_aliased_import() {
        let source = r#"package main

import (
    "fmt"
    myfmt "fmt"
)
"#;
        // Both imports should be captured (path without alias)
        let imports = parse_go_imports(source, "main.go");
        assert_eq!(imports.len(), 2);
        // Both should have path "fmt"
        assert!(imports.iter().all(|i| i.path == "fmt"));
    }

    #[test]
    fn test_parse_go_blank_import() {
        let source = r#"package main

import _ "database/sql/driver"
"#;
        let imports = parse_go_imports(source, "main.go");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "database/sql/driver");
    }

    #[test]
    fn test_parse_go_no_imports() {
        let source = "package main\n\nfunc main() {}\n";
        let imports = parse_go_imports(source, "main.go");
        assert!(imports.is_empty());
    }
}
