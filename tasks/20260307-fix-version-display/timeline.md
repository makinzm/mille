# Timeline

## 2026-03-07

### 調査
- `mille --version` → `mille 0.0.1` (`go install @v0.0.9` 後も変わらない)
- `Cargo.toml` に `version = "0.0.1"` がハードコード
- `release.yml` の `build`/`build-deb` ジョブはビルド前に version を更新しない
- `publish-crates` のみ `sed` で更新するが、WASM・バイナリビルドは対象外
- Go wrapper は `mille.wasm` を embed しており、WASM の `--version` 出力をそのまま返す
- npm wrapper も同様

### TDD 進め方
- Go wrapper に `handleVersionFlag()` ヘルパーを抽出してテストを書く（RED）
- 実装して GREEN にする
