# fix: PyPI パッケージの `mille --version` が1つ前のバージョンを表示する

## 問題

`publish-pypi` ジョブがルート `Cargo.toml` のバージョンをパッチしていないため、
`uv pip install mille` でインストールした `mille --version` が1つ前のバージョンを表示する。

- `packages/pypi/Cargo.toml` → パッチ済み ✅
- `packages/pypi/pyproject.toml` → パッチ済み ✅
- ルート `Cargo.toml` → **未パッチ** ❌ ← clap がここから `CARGO_PKG_VERSION` を取得

## タスク

- [ ] `release.yml` の `publish-pypi` ジョブに ルート `Cargo.toml` の sed パッチを追加
- [ ] timeline.md 記録
