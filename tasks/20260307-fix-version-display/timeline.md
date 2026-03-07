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
- `handleVersionFlag` が undefined でコンパイルエラー → RED 確認

### RED commit (--no-verify)
- `main_test.go` に `TestHandleVersionFlag_Version`, `TestHandleVersionFlag_ShortV`, `TestHandleVersionFlag_NotVersion` を追加

### GREEN 実装
- `packages/go/mille/main.go`: `getVersion()`（`runtime/debug.ReadBuildInfo()` 経由）と `handleVersionFlag()` を追加
- `packages/go/mille/mille.toml`: `io`, `runtime/debug`, `bytes` を `external_allow` に追加（dogfooding self-check が違反検出したため）
- `packages/npm/index.js`: `--version`/`-V` を `package.json` バージョンで返すよう修正
- `.github/workflows/release.yml`: `build`/`build-deb` ジョブに `Cargo.toml` version 同期ステップを追加

### 結果
- `go test ./...` → OK
- `cargo test` → 136 tests pass
- lefthook → pass
