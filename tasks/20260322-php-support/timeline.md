# Timeline: PHP Support

## 2026-03-22

### Branch created
`feat/php-support` branched from `main`.

### Tasks created
- `TODO.md` and `timeline.md` initialized.
- Implementation plan reviewed and approved by user.

### Phase: RED (complete)
All parser tests fail with `not yet implemented` (todo!()). Resolver tests pass (no stubs needed).
Committed with `--no-verify`: `[test] add PHP parser and resolver tests (stubs) because of PHP support`

Error log (parser):
```
failures:
    test_parse_php_aliased_use, test_parse_php_const_use, test_parse_php_function_use,
    test_parse_php_group_use, test_parse_php_multiple_use, test_parse_php_names_class,
    test_parse_php_names_comment, test_parse_php_names_function, test_parse_php_no_imports,
    test_parse_php_simple_use
panicked at: not yet implemented (todo!())
```

### Phase: GREEN (complete)
Implemented `parse_php_imports` and `parse_php_names` using tree-sitter-php 0.22.8.
Key technique: ran a temporary AST dump test to observe exact node types before implementing.

Node types used:
- `namespace_use_declaration` → top-level use statement
- `namespace_use_clause` → simple/aliased/function/const imports
- `namespace_use_group` + `namespace_use_group_clause` → grouped imports `{Auth, Logger}`
- `class_declaration`, `function_definition` → Symbol names
- `comment` → Comment names

All 348 tests pass (no regressions). Committed: `[fix] implement PHP parser and resolver because of PHP support`

### Phase: REFACTOR (complete)
Updated README.md (`[resolve.php]` section, language table, Laravel example).
Updated docs/TODO.md (PHP support entry in サマリー + completed PR entry).
Updated tasks/20260322-php-support/TODO.md checkbox states.
