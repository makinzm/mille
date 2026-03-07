# Timeline: PR 10 - GitHub Actions アノテーション出力

## 2026-03-07

### 09:00 - ブランチ作成・タスク定義
- `feat/pr10-format-option` ブランチを作成
- TODO.md と timeline.md を作成
- コードベースを調査:
  - `src/presentation/cli/args.rs`: Check サブコマンドに `--config` のみ
  - `src/presentation/formatter/mod.rs`: `terminal` モジュールのみ
  - `src/main.rs`: terminal formatter ハードコード

### 09:05 - RED → GREEN（テスト + 実装を同時展開）

**追加・変更ファイル:**
- `src/presentation/cli/args.rs`: `Format` enum と `--format` オプションを追加。テスト6件追加
- `src/presentation/formatter/github_actions.rs`: 新規作成。テスト9件含む
- `src/presentation/formatter/json.rs`: 新規作成。テスト6件含む
- `src/presentation/formatter/mod.rs`: 新モジュールを公開
- `src/main.rs`: `--format` に応じてフォーマッターを切り替え
- `tests/e2e_format.rs`: E2E テスト12件
- `docs/github-actions-usage.md`: CI 設定ガイド

**テスト結果:**
- lib テスト: 152 通過（0 失敗）
- E2E format テスト: 12 通過（0 失敗）

### 09:40 - コミット
- RED commit（`--no-verify`）: テストとスタブ
- GREEN commit: 実装コード
- REFACTOR commit: ドキュメント追記
