# CI: update-wasm push race / past audit toolchain error

## 背景

ユーザー報告のログから2件の CI エラーを確認:

1. **直近のエラー**: `Update mille.wasm (main only)` ジョブが `git push` で
   `[rejected] main -> main (fetch first)` により失敗（run 34601820960, PR #117）
2. **過去の audit エラー**: `cargo install cargo-audit --locked` が
   `cargo-audit 0.22.2` は rustc 1.88+ を要求するが、当時 pin していた
   rustc は 1.85.0 だったため失敗

## 調査結果

### 1. update-wasm push race

- PR #117 (`83dadd6`) と PR #118 (`d53b33a`) が約3分差でほぼ連続して merge された
- それぞれが独立した `CI` workflow run（push イベント）をトリガーし、
  両方の run で `update-wasm` ジョブが実行された
- `actions/checkout@v7` はデフォルトで **トリガー時点の `github.sha` に固定**
  チェックアウトする（ジョブ実行時点の最新 `main` を再フェッチしない）
- そのため #117 の run は古い base のまま `mille.wasm` をコミットしようとし、
  既に #118 の run が push 済みの `main` に対して non-fast-forward で
  reject された
- #118 の run 自体は成功し、`a3a9a07` として `main` に反映済み（データ損失なし）
- ただしこのレースは push が短時間に重なるたびに再発し得る

### 2. cargo-audit toolchain エラー

- 既に PR #117 (`rust-toolchain.toml` を 1.98.0 に更新) と
  PR #118 (`security-audit.yml` 含む関連ファイルを 1.98.0 に追従) で解決済み
- `security-audit.yml` 上の `dtolnay/rust-toolchain@1.98.0` (1.88+ 要件を満たす) を確認
- 追加のコード変更は不要。次回のスケジュール実行 (毎日 UTC 06:00) で
  green になることを確認する

## タスク

- [x] 原因調査（git log, gh run view, gh run view --log-failed）
- [x] `update-wasm` ジョブで push 前に `origin/main` の最新を取り込むよう修正
- [x] `update-wasm` ジョブに `concurrency` グループを追加し、並行実行時の
      push レースを完全に防ぐ
- [x] `actionlint` で workflow 構文を確認（exit 0, 指摘なし）
- [ ] docs/TODO.md 更新（該当なし。CI infra のみの変更で公開機能追加なし）
- [ ] commit → push → CI green 確認
