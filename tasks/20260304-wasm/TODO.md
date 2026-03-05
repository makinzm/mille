# Wasm/WASI リファクタリング

## 概要

Rust コアロジックを `wasm32-wasip1`（WASI Preview 1）ターゲットでビルドし、
Go（wazero）・Node.js・Python の各環境からポータブルに呼び出せる構成にリファクタリングする。

### 設計決定: WASI "Command" モジュール方式

既存の `src/main.rs` を **そのまま** `wasm32-wasip1` でクロスコンパイルする。

- Rust コアロジック（ViolationDetector, Parser, Resolver）は無変更
- ホスト（wazero/Node.js/Python）が WASI 環境を提供し、CWD をディレクトリ preopen でマウント
- stdin/stdout/stderr・CLI args・exit code は WASI 経由で透過的にブリッジ
- `wasm-bindgen` 不使用、純粋な `wasm32-wasip1` バイナリ

### なぜ独自 FFI (extern "C") ではなく WASI か

| 観点 | WASI Command | extern "C" + JSON ABI |
|------|-------------|------------------------|
| Rust 変更量 | **ゼロ** | 大（FS 依存を全排除、メモリ管理追加） |
| FS 操作 | preopen で透過 | ホストから全ソースを渡す必要あり |
| 対応ランタイム | wazero / Node.js WASI / wasmtime-py | 任意 |
| 実装コスト | 低 | 高 |

---

## チェックリスト

### インフラ整備
- [ ] `rust-toolchain.toml` に `targets = ["wasm32-wasip1"]` 追加
- [ ] `devbox.json` に wasi-sdk 追加（tree-sitter C コンパイル用）
- [ ] `scripts/build-wasm.sh` 作成
- [ ] `packages/wasm/` ディレクトリ作成

### TDD: Go ラッパー
- [ ] [RED] `packages/go/main_test.go` 作成（`--no-verify` コミット）
- [ ] wasm32-wasip1 ビルド実行 → `packages/go/mille.wasm` 生成・コミット
- [ ] [GREEN] `packages/go/main.go` を wazero で書き換え
- [ ] [GREEN] `packages/go/go.mod` に wazero 追加・`go mod tidy`
- [ ] `go test ./...` パス確認

### CI
- [ ] `build-wasm` ジョブ追加（wasi-sdk-30 + cargo build → artifact）
- [ ] `dogfood-go` ジョブ更新（ダウンロード廃止、artifact 使用）

### ドキュメント
- [ ] `docs/administrator/wasm_build.md` 追加（ローカルビルド手順）
- [ ] `README.md` 更新（インストール方法の変更）

---

## 将来対応（このPRでは実装しない）

- npm: Node.js WASI runner（`node:wasi` モジュール使用）
- pypi: Python wasmtime runner（`wasmtime-py` 使用）
- 共通構造: `packages/wasm/mille.wasm` を各パッケージにコピーするビルドフロー
