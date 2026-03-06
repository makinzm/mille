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

## 2026-03-06 allow_call_patterns 全言語対応

### RED
- `RawImport` に `named_imports: Vec<String>` フィールドを追加（Python/TS の named import 追跡用）
- 全構築サイトに `named_imports: vec![]` を追加
- Go/Python/TS/JS の fixture に main レイヤーと call_pattern テスト用ソースを追加
- 各言語の E2E テスト (`allow_methods=[]` で CallPatternViolation を期待) を追加
- `--no-verify` でコミット（パーサー未実装のため FAIL）

### GREEN
- **Go**: `parse_go_call_exprs` → `selector_expression` (pkg.Func()) を抽出
- **Python**: `parse_python_call_exprs` → `attribute` call (Class.method()) を抽出
  - `extract_python_named_imports` で `from X import Y` の named_imports を記録
- **TypeScript/JS**: `parse_ts_call_exprs` → `member_expression` (Class.method()) を抽出
  - `extract_ts_named_imports` で `import { Y } from X` の named_imports を記録
- `type_name_from_import` を `/` セパレータに対応（Go package 名抽出）
- `detect_call_patterns` が `named_imports` も参照するよう更新
- lefthook 全通過、210 テスト通過

PR #31 タイトル・説明を更新
