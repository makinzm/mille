# Timeline: Wasm/WASI リファクタリング

## 2026-03-05

### 設計調査・計画

- 既存コードベースを調査:
  - `src/main.rs`: `clap` CLI → WASI "command" モジュールとして自然にマッピング可能
  - `src/usecase/check_architecture.rs`: `std::fs::read_to_string` を使用 → WASI preopen で透過解決
  - `packages/go/main.go`: GitHub Releases からバイナリをダウンロードするブートストラップ方式
  - `Cargo.toml`: `tree-sitter`（C ライブラリ）が最大のクロスコンパイル障壁
- wasi-sdk-30 が最新であることを確認（計画では v24 を想定していたが更新）
- **設計決定**: WASI "Command" モジュール方式 → Rust コア無変更

---

### [RED] テスト先行コミット

- `packages/go/main.go` を `//go:embed mille.wasm` + `runWasm()` 呼び出しに書き換え
- `packages/go/wasm_runner.go` に `panic("not implemented")` スタブを追加
- `packages/go/main_test.go` に 3 件のテストを追加
- `bash scripts/build-wasm.sh` で wasm32-wasip1 バイナリを生成・配置

**ERROR LOG (RED 確認)**:
```
=== RUN   TestRunWasm_MissingConfig
panic: not implemented (goroutine in wasm_runner.go:9)
exit status 2
```
→ commit 4f748bd (`--no-verify`)

---

### [GREEN] 実装コミット

- `packages/go/wasm_runner.go`: wazero + WASI Preview 1 で実装
  - `wasi_snapshot_preview1.MustInstantiate` で WASI syscall 提供
  - `WithDirMount(dir, "/")` でホスト CWD を WASI root にマウント
  - `*sys.ExitError` を捕捉して exit code を正確に伝播
- `packages/go/go.mod`: wazero v1.11.0 追加
- `packages/go/mille.toml`: external_allow に wazero 追加

**テスト結果 (GREEN 確認)**:
```
PASS: TestRunWasm_WasmBytesEmbedded (0.00s)
PASS: TestRunWasm_MissingConfig      (0.81s)
PASS: TestRunWasm_SelfCheck          (0.77s)
ok github.com/makinzm/mille/packages/go 1.598s
```
→ commit 2f8fd8b

---

### [REFACTOR] CI 更新

- `.github/workflows/ci.yml` に `build-wasm` ジョブを追加（wasi-sdk-30 で wasm ビルド）
- `dogfood-go` を wazero ラッパー対応に更新（ダウンロード・キャッシュシード廃止）
- `tasks/20260304-wasm/TODO.md` チェックボックス更新
