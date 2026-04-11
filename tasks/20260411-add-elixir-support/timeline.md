# タイムライン: Elixir 言語サポート追加

## 2026-04-11

### 計画・調査フェーズ
- 既存パーサー（Python）・リゾルバー・DispatchingParser/Resolver を調査
- E2Eテストパターン（e2e_python.rs）を確認
- CI dogfooding ステップパターンを ci.yml で確認
- Website ドキュメント構造を確認（ja/en 両方存在）
- task ディレクトリ作成、TODO.md・timeline.md 作成

### CI dogfooding 追加
- `.github/workflows/ci.yml` の `dogfood-rust` ジョブに Elixir fixture ステップを追加

### E2E fixture 作成
- `tests/fixtures/elixir_sample/lib/domain/user.ex`
- `tests/fixtures/elixir_sample/lib/usecase/service.ex`
- `tests/fixtures/elixir_sample/lib/infrastructure/repo.ex`
- `tests/fixtures/elixir_sample/mille.toml`

### RED フェーズ
- `tests/e2e_elixir.rs` を作成（7テストケース）
- テスト実行結果:
  - 4 passed（言語未登録のため 0 files が検出される → 正常系が通る）
  - 3 failed（broken config でも違反が出ない → Elixir サポートがまだない）
  - これが期待通りの RED 状態

### 次のステップ
1. --no-verify でコミット
2. Cargo.toml に tree-sitter-elixir = "0.3.5" 追加
3. パーサー・リゾルバー実装
4. DispatchingParser/Resolver 登録
5. 全テスト通過確認
