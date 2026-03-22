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

### Phase: GREEN (in progress)
Implementing `parse_php_imports` and `parse_php_names` using tree-sitter-php 0.22.
