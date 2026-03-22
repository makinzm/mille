# PHP Support

## Goal
Add PHP file analysis support to mille.

## Scope
- Parse `use` statements (simple, aliased, grouped, function, const)
- Parse namespace declarations (for context, not as imports)
- Resolver: classify imports as Internal / External / Stdlib
- Auto-detect base namespace from `composer.json` `autoload.psr-4`
- PHP stdlib classes (DateTime, PDO, Exception, etc.)

## Tasks

- [x] Create branch `feat/php-support`
- [x] RED: Write parser and resolver tests with stubs, commit `--no-verify`
- [x] GREEN: Implement `src/infrastructure/parser/php.rs`
- [x] GREEN: Implement `src/infrastructure/resolver/php.rs`
- [x] GREEN: Wire up `parser/mod.rs`, `resolver/mod.rs`, `usecase/init.rs`, `Cargo.toml`
- [x] REFACTOR: Update `README.md` and `docs/TODO.md`
- [ ] Create PR

## Acceptance Criteria
- All new tests pass
- Existing tests continue to pass (no regression)
- lefthook passes
- PHP appears in language support table in README.md
