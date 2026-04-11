# タスク: Elixir 言語サポート追加

## 概要

Elixir 言語の構造チェック機能を追加する。既存の9言語（Rust, Go, Python等）と同じパターンで実装。

## 実施順序（CLAUDE.md 言語追加チェックリスト厳守）

- [x] CI dogfooding ステップ追加（`.github/workflows/ci.yml`）← 最初に追加
- [x] E2E fixture 作成（`tests/fixtures/elixir_sample/`）
- [x] E2E テスト作成（`tests/e2e_elixir.rs`）← テストファースト・--no-verifyでコミット
- [x] `Cargo.toml` に `tree-sitter-elixir = "0.2.0"` 追加
- [x] Parser 実装（`src/infrastructure/parser/elixir.rs`）
- [x] Resolver 実装（`src/infrastructure/resolver/elixir.rs`）
- [x] DispatchingParser 登録（`src/infrastructure/parser/mod.rs`）
- [x] DispatchingResolver 登録（`src/infrastructure/resolver/mod.rs`）
- [x] Website ドキュメント（`website/src/content/docs/guides/languages/elixir.md`）
- [x] Website en ドキュメント（`website/src/content/docs/en/guides/languages/elixir.md`）
- [x] astro.config.mjs のサイドバーに Elixir を追加
- [x] index.mdx の対応言語リストに Elixir を追加
- [x] README.md フィーチャーマトリックス更新
- [x] docs/TODO.md 更新

## TDD コミット順序

1. `[test] elixir E2E テスト・ユニットテストのスケルトンを追加 because of テストファースト`（--no-verify）
2. `[fix] Elixir パーサー・リゾルバー実装 because of E2E テスト通過`
3. `[refactor] ドキュメント・README 更新 because of Elixir サポート完成`

## Fixture 設計

### `tests/fixtures/elixir_sample/`

```
lib/
  domain/
    user.ex
  usecase/
    service.ex
  infrastructure/
    repo.ex
mille.toml
```

### mille.toml

- domain: opt-in, allow=[]
- usecase: opt-in, allow=["domain"]
- infrastructure: opt-out, deny=[]
- app_name = "MyApp"

## テストケース一覧

### ユニットテスト（parser）
- test_parse_alias_simple: `alias MyApp.Domain.User` → path="MyApp.Domain.User"
- test_parse_import: `import Enum` → path="Enum"
- test_parse_require: `require Logger` → path="Logger"
- test_parse_use: `use Ecto.Schema` → path="Ecto.Schema"
- test_parse_alias_with_as: `alias MyApp.Domain.User, as: User` → path="MyApp.Domain.User"
- test_parse_no_imports: インポートなし → 空リスト

### ユニットテスト（resolver）
- test_internal_module: path="MyApp.Domain.User", app_name="MyApp" → Internal
- test_external_module: path="Ecto.Repo", app_name="MyApp" → External
- test_internal_resolved_path: path="MyApp.Domain.User" → resolved_path="lib/domain/user.ex"

### E2E テスト
- test_elixir_valid_config_exits_zero: 正常fixture → exit 0
- test_elixir_valid_config_summary_shows_zero_errors: 正常fixture → "0 error(s)" 含む
- test_elixir_valid_config_all_layers_clean: 正常fixture → domain/usecase/infrastructure 含む
- test_elixir_broken_usecase_exits_one: usecase allow=[] → exit 1
- test_elixir_broken_usecase_mentions_usecase: 同上 → "usecase" 含む
- test_elixir_broken_infra_deny_domain_exits_one: deny=["domain"] → exit 1
- test_elixir_broken_external_deny_exits_one: domain external_deny=["Ecto"] → exit 1
