# Timeline: TypeScript / JavaScript サポート

## 2026-03-06

### TODO 作成
- `tasks/20260306-ts-js-support/TODO.md` を作成
- branch `feat/ts-js-support` は作成済み

### RED commit
- `.gitignore` に node_modules / dist を追加
- `Cargo.toml` に tree-sitter-javascript 0.21 + tree-sitter-typescript 0.21 を追加
- fixtures: `tests/fixtures/typescript_sample/` と `tests/fixtures/javascript_sample/` を作成
- `src/infrastructure/parser/typescript.rs` スタブ（`todo!()`）
- `src/infrastructure/resolver/typescript.rs` スタブ（`todo!()`）
- `tests/e2e_typescript.rs` と `tests/e2e_javascript.rs` を作成（各 10 テスト）
- `--no-verify` で RED commit

### GREEN commit
- `TypeScriptParser` 実装: tree-sitter で import_statement を解析、.ts/.tsx/.js/.jsx で grammar を切り替え
- `TypeScriptResolver` 実装: `./` や `../` で始まる import → Internal + resolved_path 計算、その他 → External
- `normalize_path()` で `..` を含むパスを正規化（`usecase/../domain/user` → `domain/user`）
- dispatcher に TypeScriptParser / TypeScriptResolver を接続
- FsSourceFileRepository に .ts/.tsx/.js/.jsx を追加
- 全 206 テスト GREEN、lefthook 全通過
- GREEN commit

### dogfood & docs
- `packages/npm/mille.toml` 追加（bindings layer）
- `docs/e2e_checklist.md` に TypeScript / JavaScript 列を追加
- `README.md` に TS/JS サポート, mille.toml 例, `[resolve.typescript]` リファレンスを追記
- `ci.yml` に TS/JS fixture と npm package の dogfood ステップを追加
- `docs/TODO.md` の PR 9 を完了マーク
