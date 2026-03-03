# 開発タスク

- [x] `docs/TODO.md` を作成し、スクラム開発での縦切りPR粒度でのロードマップを定義する
- [x] PR 1（パッケージ名予約のためのダミーCD構築とリリース） を実施する
  - [x] `cargo init` によるRustプロジェクトの初期化と最小 `Cargo.toml` 設定
  - [x] GitHub Actions (`.github/workflows/cd-reserve.yml`等) で空パッケージをpublishするCDパイプライン構築 (crates.io)
  - [x] npm パッケージの自動 publish パイプライン
  - [x] PyPI パッケージの自動 publish パイプライン
  - [x] Java (Maven Central等) / Dart (pub.dev) などの検討・対応
  - [x] `rust-toolchain.toml` の設定(pinned v1.85.0) と `lefthook` の導入 (devbox経由)
  - [x] `RED -> GREEN -> REFACTOR` ワークフローの準備
