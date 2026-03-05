# Wasm ビルドガイド

## 概要

`mille` のコアロジック（Rust）を `wasm32-wasip1`（WASI Preview 1）ターゲットでビルドし、
`packages/go/mille.wasm` として埋め込みます。
Go ラッパーは `//go:embed mille.wasm` でバイナリに同梱するため、
`go install` 後はネットワーク接続不要で動作します。

---

## ローカルでの Go ラッパービルド手順

### 前提

| ツール | 用途 | 入手方法 |
|--------|------|----------|
| Rust 1.85.0 | コアのビルド | `rustup` (devbox で管理) |
| wasm32-wasip1 target | wasm クロスコンパイル | 後述のスクリプトが自動追加 |
| wasi-sdk-30 | tree-sitter の C コードを wasm 向けにコンパイル | 後述のスクリプトが自動ダウンロード |
| Go 1.24+ | Go ラッパーのビルド | devbox で管理 |
| curl | wasi-sdk のダウンロード | OS 標準またはdevbox |

### Step 1: mille.wasm をビルド

```bash
# リポジトリ root で実行
bash scripts/build-wasm.sh
```

初回は `.wasi-sdk/` ディレクトリに wasi-sdk-30 を自動ダウンロードします（約 130MB）。
2 回目以降はキャッシュを再利用します（`WASI_SDK_PATH` 環境変数で上書きも可）。

**出力ファイル:**
- `packages/wasm/mille.wasm` — canonical の Wasm バイナリ
- `packages/go/mille.wasm` — Go ラッパー埋め込み用コピー

### Step 2: Go ラッパーをビルド

```bash
cd packages/go
go build -o mille_wasm-cli .
```

`//go:embed mille.wasm` により Step 1 の成果物がバイナリに埋め込まれます。

### Step 3: 動作確認

```bash
# packages/go ディレクトリで実行（mille.toml が必要）
./mille_wasm-cli check

# リポジトリ root の Rust プロジェクトを確認する場合
cd /path/to/your/project
/path/to/mille-cli check
```

### テスト実行

```bash
cd packages/go
go test -v -timeout 60s ./...
```

---

## wasm ビルドの仕組み

### なぜ wasi-sdk が必要か

`tree-sitter` は C ライブラリを含み、`cc` クレートでコンパイルされます。
通常の clang（Nix-wrapped 等）は glibc sysroot を注入するため `wasm32-wasip1` に向けられません。
wasi-sdk は自己完結型の clang + WASI sysroot を提供するため、これを使用します。

### tree-sitter の `dup()` 問題

`tree-sitter 0.22.6` の `ts_tree_print_dot_graph()` が `dup()`（POSIX 関数）を呼びますが、
WASI Preview 1 には `dup()` が存在しません。

**対策**: `-Wno-implicit-function-declaration` で C コンパイルエラーを抑制し、
release ビルドの `--gc-sections`（DCE: Dead Code Elimination）で除去します。
`ts_tree_print_dot_graph` は mille の実行パスから一切到達不能なため、
Wasm バイナリに含まれません。

### WASI "Command" モジュール方式

既存の `src/main.rs` をそのまま `wasm32-wasip1` でクロスコンパイルします。

| 要素 | ホスト（Go/wazero）側の担当 |
|------|---------------------------|
| ファイルシステム | `WithDirMount(cwd, "/")` で CWD を WASI root にマウント |
| CLI 引数 | `WithArgs("mille", <user-args>...)` で伝達 |
| 標準入出力 | `WithStdin/Stdout/Stderr` で接続 |
| 終了コード | `*sys.ExitError` から取得 |

---

## CI での Wasm ビルド

`.github/workflows/ci.yml` の `build-wasm` ジョブが担当します。

```
test → build-wasm → dogfood-go
```

- `build-wasm`: wasi-sdk-30 をダウンロードし `cargo build --target wasm32-wasip1 --release`
- 成果物は `actions/upload-artifact` で `mille-wasm` として保存
- `dogfood-go`: artifact をダウンロードして `go test ./...` + `go build` + self-check

---

## .wasm ファイルの Git 管理

`packages/go/mille.wasm` と `packages/wasm/mille.wasm` は Git で追跡されています。

- **理由**: `go install github.com/makinzm/mille/packages/go@latest` が動作するには
  `mille.wasm` がモジュール内に存在する必要があります
- **更新タイミング**: Rust コアに変更があった場合、`bash scripts/build-wasm.sh` を再実行して
  生成された `.wasm` を `git add` してコミットしてください

```bash
bash scripts/build-wasm.sh
git add packages/wasm/mille.wasm packages/go/mille.wasm
git commit -m "[fix] mille.wasm を更新 because of <変更内容>"
```

---

## 将来の拡張: Node.js / Python への展開

同じ `mille.wasm` を Node.js と Python でも再利用できます。

### Node.js (WASI)

```js
const { WASI } = require('node:wasi');
const fs = require('fs');

const wasm = fs.readFileSync('mille.wasm');
const wasi = new WASI({ version: 'preview1', args: ['mille', 'check'], preopens: { '/': '.' } });
const { instance } = await WebAssembly.instantiate(wasm, wasi.getImportObject());
wasi.start(instance);
```

### Python (wasmtime)

```python
from wasmtime import Store, Module, Linker, WasiConfig

store = Store()
wasi = WasiConfig()
wasi.argv = ['mille', 'check']
wasi.preopen_dir('.', '/')
store.set_wasi(wasi)

linker = Linker(store.engine)
linker.define_wasi()
module = Module.from_file(store.engine, 'mille.wasm')
instance = linker.instantiate(store, module)
start = instance.exports(store)['_start']
start(store)
```

`.wasm` ファイルは `packages/wasm/mille.wasm` が正規ソースです。
各パッケージディレクトリにコピーして `//go:embed` または組み込みで利用してください。
