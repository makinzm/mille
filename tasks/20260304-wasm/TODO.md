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
- [x] `rust-toolchain.toml` に `targets = ["wasm32-wasip1"]` 追加
- [x] `devbox.json` に wget 追加（wasi-sdk ダウンロード用）
- [x] `scripts/build-wasm.sh` 作成（wasi-sdk-30 自動取得 + wasm ビルド）
- [x] `packages/wasm/` ディレクトリ作成

### TDD: Go ラッパー
- [x] [RED] `packages/go/main_test.go` 作成（`--no-verify` コミット）
- [x] wasm32-wasip1 ビルド実行 → `packages/wasm/mille.wasm` 生成・コミット
- [x] [GREEN] `packages/go/main.go` を wazero で書き換え
- [x] [GREEN] `packages/go/wasm_runner.go` を wazero 実装に更新
- [x] [GREEN] `packages/go/go.mod` に wazero v1.11.0 追加
- [x] `go test ./...` パス確認（3件 PASS）

### CI
- [x] `build-wasm` ジョブ追加（wasi-sdk-30 + cargo build）
- [x] `dogfood-go` ジョブ更新（ダウンロード廃止）
- [x] `build-wasm` ジョブに stale .wasm 検知ステップ追加

### ドキュメント
- [x] `docs/administrator/wasm_build.md` 追加（ローカルビルド手順・.wasm Git 管理の根拠）

### REFACTOR: packages/wasm を Go モジュール化（single canonical copy）
- 背景: packages/go/mille.wasm と packages/wasm/mille.wasm の 2 コピー問題を解消。
  npm/pypi を追加しても .wasm が増えない構造にする。
- [x] `packages/wasm/go.mod` 作成（module github.com/makinzm/mille/packages/wasm）
- [x] `packages/wasm/wasm.go` 作成（//go:embed mille.wasm → var Wasm []byte）
- [x] `go.work` 作成（use ./packages/wasm を追加）
- [x] `packages/go/go.mod` に packages/wasm v0.0.1 require 追加
- [x] `packages/go/main.go` を millewasm.Wasm 使用に書き換え（//go:embed 削除）
- [x] `packages/go/main_test.go` を millewasm.Wasm 使用に書き換え
- [x] `packages/go/go.mod` に replace ディレクティブ追加（local dev 用）
- [x] `git rm --cached packages/go/mille.wasm`（git 追跡解除）
- [x] `.gitignore` に `packages/go/mille.wasm` 追加
- [x] `scripts/build-wasm.sh` を packages/wasm のみコピーに修正
- [x] `go mod tidy` 実行（go.sum 更新）
- [x] `go test ./...` パス確認
- [x] CI `build-wasm` ジョブ更新（packages/go/mille.wasm への参照を削除）
- [x] `docs/administrator/wasm_build.md` 更新（single copy・go.work 説明）
- [x] リファクタリングコミット

---

## 将来対応（このPRでは実装しない）

- npm: Node.js WASI runner（`node:wasi` モジュール使用）
- pypi: Python wasmtime runner（`wasmtime-py` 使用）
- 共通構造: npm/pypi は packages/wasm/mille.wasm を publish 時にバンドル（コミットしない）
- packages/wasm v0.0.1 を Go module proxy へ公開後、go.mod の replace ディレクティブ削除
