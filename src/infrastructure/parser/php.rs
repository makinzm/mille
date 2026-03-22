use tree_sitter::Node;

use crate::domain::entity::call_expr::RawCallExpr;
use crate::domain::entity::import::{ImportKind, RawImport};
use crate::domain::entity::name::{NameKind, RawName};
use crate::domain::repository::parser::Parser;

/// Concrete implementation of the `Parser` port for PHP source files.
pub struct PhpParser;

impl Parser for PhpParser {
    fn parse_imports(&self, source: &str, file_path: &str) -> Vec<RawImport> {
        parse_php_imports(source, file_path)
    }

    fn parse_call_exprs(&self, _source: &str, _file_path: &str) -> Vec<RawCallExpr> {
        // PHP call expression analysis is not yet implemented.
        vec![]
    }

    fn parse_names(&self, source: &str, file_path: &str) -> Vec<RawName> {
        parse_php_names(source, file_path)
    }
}

/// Parse PHP source code and extract all `use` statements.
pub fn parse_php_imports(source: &str, file_path: &str) -> Vec<RawImport> {
    todo!()
}

/// Parse PHP source code and extract named entities for naming convention checks.
pub fn parse_php_names(source: &str, file_path: &str) -> Vec<RawName> {
    todo!()
}

fn extract_text(node: &Node, source: &[u8]) -> Option<String> {
    node.utf8_text(source).ok().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::name::NameKind;

    fn parse(src: &str) -> Vec<RawImport> {
        parse_php_imports(src, "test.php")
    }

    // ------------------------------------------------------------------
    // parse_php_imports
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_php_simple_use() {
        let imports = parse("<?php\nuse App\\Models\\User;\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "App\\Models\\User");
        assert_eq!(imports[0].line, 2);
        assert_eq!(imports[0].file, "test.php");
    }

    #[test]
    fn test_parse_php_aliased_use() {
        // Alias is ignored — we record the original class path.
        let imports = parse("<?php\nuse App\\Models\\User as UserModel;\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "App\\Models\\User");
    }

    #[test]
    fn test_parse_php_group_use() {
        // `use App\Services\{Auth, Logger};` expands to two imports.
        let imports = parse("<?php\nuse App\\Services\\{Auth, Logger};\n");
        assert_eq!(imports.len(), 2);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(
            paths.contains(&"App\\Services\\Auth"),
            "expected App\\Services\\Auth, got {:?}",
            paths
        );
        assert!(
            paths.contains(&"App\\Services\\Logger"),
            "expected App\\Services\\Logger, got {:?}",
            paths
        );
    }

    #[test]
    fn test_parse_php_function_use() {
        let imports = parse("<?php\nuse function App\\Helpers\\format_date;\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "App\\Helpers\\format_date");
    }

    #[test]
    fn test_parse_php_const_use() {
        let imports = parse("<?php\nuse const App\\Config\\MAX_RETRIES;\n");
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "App\\Config\\MAX_RETRIES");
    }

    #[test]
    fn test_parse_php_multiple_use() {
        let src = "<?php\nuse App\\Models\\User;\nuse App\\Services\\Auth;\nuse Illuminate\\Http\\Request;\n";
        let imports = parse(src);
        assert_eq!(imports.len(), 3);
        let paths: Vec<&str> = imports.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"App\\Models\\User"));
        assert!(paths.contains(&"App\\Services\\Auth"));
        assert!(paths.contains(&"Illuminate\\Http\\Request"));
    }

    #[test]
    fn test_parse_php_no_imports() {
        let imports = parse("<?php\nclass Foo {}\n");
        assert!(imports.is_empty());
    }

    // ------------------------------------------------------------------
    // parse_php_names
    // ------------------------------------------------------------------

    #[test]
    fn test_parse_php_names_class() {
        let source = "<?php\nclass UserController {}\n";
        let names = parse_php_names(source, "test.php");
        let found = names
            .iter()
            .find(|n| n.name == "UserController" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "class UserController should be detected as Symbol, got: {:#?}",
            names
        );
        assert_eq!(found.unwrap().line, 2);
    }

    #[test]
    fn test_parse_php_names_function() {
        let source = "<?php\nfunction getUserById() {}\n";
        let names = parse_php_names(source, "test.php");
        let found = names
            .iter()
            .find(|n| n.name == "getUserById" && n.kind == NameKind::Symbol);
        assert!(
            found.is_some(),
            "function getUserById should be detected as Symbol, got: {:#?}",
            names
        );
    }

    #[test]
    fn test_parse_php_names_comment() {
        let source = "<?php\n// connect to db\n$x = 1;\n";
        let names = parse_php_names(source, "test.php");
        let found = names
            .iter()
            .find(|n| n.kind == NameKind::Comment && n.name.contains("connect to db"));
        assert!(
            found.is_some(),
            "comment with 'connect to db' should be detected, got: {:#?}",
            names
        );
        assert_eq!(found.unwrap().line, 2);
    }
}
