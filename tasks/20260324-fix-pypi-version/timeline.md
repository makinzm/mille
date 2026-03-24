# Timeline

## 2026-03-24

- 調査: `publish-pypi` ジョブがルート `Cargo.toml` をバージョンパッチしていないことを特定
- 原因: `packages/pypi/Cargo.toml` が `mille-core = { package = "mille", path = "../.." }` でルートクレートに依存。clap の `#[command(version)]` はルートの `CARGO_PKG_VERSION` を使うため、パッチ漏れで旧バージョンがコンパイル時に埋め込まれる
