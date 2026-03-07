# PR 10: GitHub Actions アノテーション出力 (`--format github-actions`)

## 概要

`mille check` に `--format` オプションを追加し、GitHub Actions / JSON / terminal の3形式に対応する。

## タスクリスト

- [ ] `src/presentation/cli/args.rs` に `--format` オプション追加（`Format` enum: `terminal` / `json` / `github-actions`）
- [ ] `src/presentation/formatter/github_actions.rs` 作成（`::error file=...,line=N::msg` 形式）
- [ ] `src/presentation/formatter/json.rs` 作成（JSON 形式）
- [ ] `src/presentation/formatter/mod.rs` を更新して新モジュールを公開
- [ ] `src/main.rs` で `--format` に応じてフォーマッターを切り替える
- [ ] CI ドキュメントに GitHub Actions 設定例を追記（`docs/github-actions-usage.md`）
- [ ] E2E テストの追加（`tests/e2e_format.rs`）

## 受け入れ条件

- `mille check --format github-actions` で `::error file=<path>,line=<n>::<msg>` が出力される
- `mille check --format json` で JSON が出力される
- `mille check` はデフォルトで terminal フォーマット
- 全テスト通過
