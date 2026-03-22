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

- [ ] Create branch `feat/php-support`
- [ ] RED: Write parser and resolver tests with stubs, commit `--no-verify`
- [ ] GREEN: Implement `src/infrastructure/parser/php.rs`
- [ ] GREEN: Implement `src/infrastructure/resolver/php.rs`
- [ ] GREEN: Wire up `parser/mod.rs`, `resolver/mod.rs`, `usecase/init.rs`, `Cargo.toml`
- [ ] REFACTOR: Update `README.md` and `docs/TODO.md`
- [ ] Create PR

## Acceptance Criteria
- All new tests pass
- Existing tests continue to pass (no regression)
- lefthook passes
- PHP appears in language support table in README.md
