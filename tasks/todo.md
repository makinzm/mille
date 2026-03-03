# 開発タスク

- [x] `docs/TODO.md` を作成し、スクラム開発での縦切りPR粒度でのロードマップを定義する
- [ ] PR 1（パッケージ名予約のためのダミーCD構築とリリース） を実施する
  - [ ] `cargo init` によるRustプロジェクトの初期化と最小 `Cargo.toml` 設定
  - [ ] GitHub Actions (`.github/workflows/cd-reserve.yml`等) で空パッケージをpublishするCDパイプライン構築 (crates.io)
  - [ ] npm パッケージの自動 publish パイプライン
  - [ ] PyPI パッケージの自動 publish パイプライン
  - [ ] Java (Maven Central等) / Dart (pub.dev) などの検討・対応
  - [ ] `rust-toolchain.toml` の設定と `lefthook` (v1) の導入
  - [ ] `RED -> GREEN -> REFACTOR` ワークフローの準備
