# rustfmt / clippy drift after toolchain bump to 1.98.0

## 背景

前タスク ([[20260911-ci-wasm-push-race]]) の DA 相当チェックで、
`main` 上で `cargo fmt --check` が失敗していることを発見。ユーザーへ報告し
「fix it」の指示を受けて着手。

調査の結果、`cargo clippy --tests -- -D warnings` も別途失敗しており
（rustfmt/clippy 双方とも rust-toolchain 1.85.0 → 1.98.0 更新 (#117/#118) で
新しいルールが有効化されたことが原因）、同一根本原因のため同じ PR に含める
ことをユーザーに確認して承認を得た。

## タスク

- [x] `cargo fmt` を実行し fmt drift を解消（4ファイル、フォーマットのみ）
- [x] `cargo clippy --tests -- -D warnings` の指摘を修正
  - [x] `Iterator::last` → `next_back`（3箇所: violation_detector.rs x2, python.rs x1）
  - [x] doc list item overindented（go.rs のインデント修正）
  - [x] empty lines after doc comment（e2e_c.rs: 未使用の `///` を `//` に変更）
  - [x] `&PathBuf` → `&Path`（e2e_multifile_main.rs、2箇所）
- [x] `cargo clippy --tests --all-targets -- -D warnings` で他に指摘が
      残っていないことを確認（clean）
- [x] `cargo fmt --check` clean 確認
- [x] `cargo test` 全 pass 確認（回帰なし）
- [ ] commit → push → PR 作成 → CI green 確認
