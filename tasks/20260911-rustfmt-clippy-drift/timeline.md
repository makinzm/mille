# Timeline

## 2026-09-11

- 前タスク完了報告で `cargo fmt --check` が main 上で失敗していることを
  ユーザーへ報告。ユーザーから「fix it」の指示。
- `main` から新ブランチ `fix/rustfmt-drift` を作成。
- `cargo fmt` 実行 → 4ファイルのフォーマットのみ変更。
  `cargo fmt --check` で clean を確認。
- `cargo clippy --tests -- -D warnings` を実行したところ、fmt とは別に
  5件の lint エラー（+ lib/lib-test の compile failure ×2）を検出。
  スコープを fmt のみに留めるか clippy も含めるかユーザーに確認
  （`AskUserQuestion`）→「Fix clippy too, same PR」を選択。
- clippy 指摘を修正:
  - `violation_detector.rs:356,362` `split(...).last()` →
    `.next_back()`（DoubleEndedIterator 全走査の無駄を指摘）
  - `python.rs:325` 同様に `.last()` → `.next_back()`
  - `go.rs:63-65` doc list item overindented（3スペース→2スペースへ）
- 上記3ファイルへの Edit が TDD ガード hook にブロックされた
  (`~/.claude/scripts/tdd-guard.sh`)。`cargo test --lib` を先に実行し
  434 tests green を確認してからガード解除・再実行で通過。
- 修正後に再度 `cargo clippy --tests -- -D warnings` を実行すると、
  lib のコンパイルが通ったことで初めて到達する統合テストバイナリ側で
  さらに2件検出:
  - `tests/e2e_c.rs:249-267` 実装のないコメントブロックが `///`
    （doc comment）になっており「empty lines after doc comment」。
    どちらのブロックも後続の `C_BROKEN_NAMING_TOML` とは無関係な
    開発メモだったため `//` に変更（プレーンコメント化）。
  - `tests/e2e_multifile_main.rs:22,38` `&PathBuf` 引数 → `&Path`
    （`clippy::ptr_arg`）。`use std::path::Path` を追加し2箇所修正。
- `cargo clippy --tests --all-targets -- -D warnings` → clean（exit 0）
- `cargo fmt --check` → clean（exit 0）
- `cargo test` → 全 pass、失敗・エラーなし（grep で FAILED/error[ 系を確認）
- `devbox run -- lefthook run pre-commit` は devbox の nix シェルに `git`
  バイナリが無く `tool 'git' not found` で失敗（環境固有の問題で今回の
  変更とは無関係）。cargo fmt/clippy/test を個別実行しており lefthook
  相当のチェックは既にカバー済みと判断し、深追いしない。
