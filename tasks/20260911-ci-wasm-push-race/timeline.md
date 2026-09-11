# Timeline

## 2026-09-11

- ユーザーからユーザーが貼った CI ログ2件を受け取る:
  1. `Update mille.wasm (main only)` の `git push` が `[rejected] (fetch first)`
  2. `Security Audit` の `cargo install cargo-audit --locked` が rustc バージョン不足で失敗
- `git log --oneline -20` で直近のマージを確認。
  `e5e4abf`（rust-toolchain を 1.85.0 に pin）の後、`83dadd6`(#117) と
  `d53b33a`(#118) で 1.98.0 へ更新済みと判明。
- `gh run list --branch main` / `gh run list --workflow=security-audit.yml` で
  実際の run 履歴を確認:
  - #117 の CI run (34601820960): `update-wasm` ジョブが exit code 1 で失敗
  - #118 の CI run (34602100688): 全ジョブ success。`a3a9a07` として
    `mille.wasm` の更新コミットが main に反映済み
  - Security Audit の scheduled run は 09-07〜09-10 まで rustc バージョン
    不足で failure が続いていたが、09-11 の run は手動キャンセル
    (`@makinzm` による)。toolchain 更新後の次回 schedule (毎日 06:00 UTC)
    で green になる見込み。追加コード変更は不要と判断。
- `gh run view <job-id>/logs` で失敗ジョブのログを確認し、
  `actions/checkout` がジョブ起動時点の `github.sha` に固定チェックアウトする
  ため、後発の run が先行 run の push を認識できずに non-fast-forward で
  reject されると特定。
- 修正方針:
  1. `update-wasm` ジョブに `concurrency: group: update-wasm-main` を追加し、
     並行 run を直列化
  2. push 前に `git fetch origin main && git reset --hard origin/main` を
     追加し、直列化後も必ず最新 main を base にする
- `.github/workflows/ci.yml` の `update-wasm` ジョブを編集。
- `devbox run -- actionlint` で構文検証 → exit 0、指摘なし。
- CI infra のみの変更のため、Rust テストコードの追加は対象外
  （プロジェクトの TDD 原則は Rust ソースコード変更が対象。workflow YAML の
  検証は actionlint が相当する）。
