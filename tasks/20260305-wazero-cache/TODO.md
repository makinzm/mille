# PR N+1: wazero CompilationCache の導入（Go ラッパー起動高速化）

## 背景

wazero は起動のたびに `.wasm` を機械語へ変換するため、ネイティブバイナリと比べて
起動が遅い（コールドスタートで数百ms〜数秒）。
`CompilationCache` を使えば変換結果をファイルに保存し、2 回目以降の起動でキャッシュを再利用できる。

キャッシュディレクトリ: `os.UserCacheDir()` 配下の `mille/wazero/`
- Linux: `~/.cache/mille/wazero/`
- macOS: `~/Library/Caches/mille/wazero/`

---

## 作業手順

1. **TODO + AGENTS.md**（本ファイル）
2. **RED commit**: `compilationCacheDir()` を呼ぶテストを追加（関数未存在でテスト失敗）
3. **GREEN commit**: 実装して全テスト通過
4. **docs 更新**: `docs/administrator/wasm_build.md` にキャッシュの仕組みを追記

---

## チェックリスト

- [x] `tasks/20260305-wazero-cache/TODO.md` 作成（本ファイル）
- [ ] RED: `compilationCacheDir()` の動作を検証するテスト追加
- [ ] GREEN: `wasm_runner.go` に CompilationCache 実装
- [ ] `docs/administrator/wasm_build.md` にキャッシュの仕組みと場所を記載
- [ ] `docs/TODO.md` の PR N+1 チェックボックスを更新
