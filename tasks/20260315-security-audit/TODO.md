# Security Audit CI ワークフロー

## 概要

`cargo audit` を使った自動脆弱性スキャンを CI に組み込む。
脆弱性が検出された場合、GitHub Issue を自動作成して通知する。

## タスク

- [ ] `.github/workflows/security-audit.yml` を作成する
  - スケジュール実行（毎日 UTC 06:00）
  - push / PR 時に Cargo.lock 変更があればトリガー
  - `cargo audit` で依存パッケージをスキャン
  - 脆弱性検出時にスケジュール実行では GitHub Issue を自動作成
  - PR 上では annotations として表示
- [ ] `actionlint` が通ることを確認する
- [ ] PR 作成

## 設計メモ

- Rust toolchain: 1.85.0（rust-toolchain.toml 準拠）
- `cargo audit` は `cargo install cargo-audit` でインストール
- Issue 作成には `actions/github-script` を使用
- PR / push 実行ではワークフローを fail させる（CI ゲート）
- スケジュール実行でのみ Issue を作成（PR ノイズ回避）
