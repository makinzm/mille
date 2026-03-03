# PR 3 & CD Workflow Update

## Checklist

### 1. CD Workflow Tag Push Activation
- [x] Create `.github/workflows/cd.yml` mapping release tags (`v*`) to Publish Jobs.
- [x] Migrate steps from `cd-reserve.yml` checking registry environment config.
- [x] Remove `.github/workflows/cd-reserve.yml`.
- [x] Document release usage in `README.md`.

### 2. PR 3: tree-sitter AST Import Parsing (Rust)
- [ ] Add `tree-sitter` and `tree-sitter-rust` dependencies.
- [ ] Define `domain::entity::import::RawImport`.
- [ ] Define `domain::repository::parser::Parser` trait.
- [ ] Add `tree_sitter_rust::language()` bindings.
- [ ] Implement `infrastructure::parser::rust::RustParser::parse`.
- [ ] Write unit tests for importing `use_declaration`, `mod_item`, and nested blocks.

### 3. Review & Consolidation
- [ ] Review implementation against expected test logic.
- [ ] Run `cargo clippy` and `cargo fmt`.
