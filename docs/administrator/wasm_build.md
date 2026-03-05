# Wasm ビルドガイド

## 概要

`mille` のコアロジック（Rust）を `wasm32-wasip1`（WASI Preview 1）ターゲットでビルドし、
Go ラッパー（wazero）に埋め込みます。
`go install` 後はネットワーク接続不要で動作します。

### .wasm ファイルのリポジトリ構造

| ファイル | 役割 |
|----------|------|
| `packages/wasm/mille.wasm` | **唯一のコミット済み .wasm**。`//go:embed` でエクスポート |
| `packages/go/` | `packages/wasm` を Go モジュール依存として import。.wasm のコピーなし |

`packages/wasm` は独立した Go モジュール (`github.com/makinzm/mille/packages/wasm`) として
`var Wasm []byte` をエクスポートします。`packages/go` はこれを通常の Go 依存として参照します。

ローカル開発・CI では `go.work`（Go Workspaces）が `packages/wasm` をローカルパスで解決します。
npm/pypi などの将来のパッケージは `packages/wasm/mille.wasm` を publish 時にバンドルします
（コピーをリポジトリには持たない）。

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
- `packages/wasm/mille.wasm` — 唯一の canonical Wasm バイナリ（Git 管理）

### Step 2: Go ラッパーをビルド

```bash
# リポジトリ root で実行（go.work が参照される）
cd packages/go
go build -o mille_go .
```

`packages/wasm` モジュールが `go.work` 経由でローカル解決され、
`packages/wasm/mille.wasm` が Go バイナリに埋め込まれます。

### Step 3: 動作確認

```bash
# packages/go ディレクトリで実行（mille.toml が必要）
./mille_go check

# リポジトリ root の Rust プロジェクトを確認する場合
cd /path/to/your/project
/path/to/mille_go check
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

- **`build-wasm`**: wasi-sdk-30 で `cargo build --target wasm32-wasip1 --release` を実行し、
  **コミット済みの `.wasm` と CI ビルド結果を `git diff` で比較する**。
  差異があれば CI を失敗させ、開発者に再コミットを促す。
- **`dogfood-go`**: `build-wasm` が通った場合のみ実行（checkout 済みの `.wasm` を使用）。

### CI が失敗した場合

`build-wasm` で以下のエラーが出たとき：

```
ERROR: Committed packages/wasm/mille.wasm is out of sync with the current Rust source.
```

Rust コアを変更したのに `.wasm` を更新していないことを意味します。
ローカルで以下を実行して再プッシュしてください：

```bash
bash scripts/build-wasm.sh
git add packages/wasm/mille.wasm
git commit -m "[fix] mille.wasm を更新 because of <変更内容>"
git push
```

---

## .wasm ファイルの Git 管理

### なぜバイナリを Git に含めるのか（3MB の根拠）

`packages/wasm/mille.wasm`（約 3MB）は Git で追跡されています。
これが **リポジトリ内で唯一の .wasm ファイル** です（`packages/go/` にはコピーを持ちません）。

バイナリをリポジトリに含める理由と代替案の比較は以下のとおりです。

| 方式 | `go install` 時の動作 | 外部通信 | ランタイム依存 |
|------|----------------------|----------|---------------|
| **Git 管理（採用）** | そのまま動く | なし | なし |
| GitHub Releases 配布 | 初回に自動ダウンロード | あり | なし |
| 実行時ビルド | Rust + wasi-sdk が必要 | なし | あり |

`go install` が成功した後、エンドユーザーは **追加ツールなし・ネットワークなし** で
`mille check` を実行できる必要があります。
これを実現するために `packages/wasm` モジュールが `//go:embed mille.wasm` で
バイナリに埋め込み、`packages/go` がそれを Go 依存として参照します。

3MB という サイズについては：

- Rust の release ビルドには標準ライブラリ（`std`）が静的リンクされる
- tree-sitter の C ランタイムが含まれる
- `wasm-opt` による最適化を将来検討できるが、現時点では未適用

### 更新タイミング

Rust コア（`src/`）に変更があった場合は必ず `.wasm` を再ビルドしてコミットしてください。
CI の `build-wasm` ジョブがこれを自動検出します。

```bash
bash scripts/build-wasm.sh
git add packages/wasm/mille.wasm
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
npm/pypi は publish 時のビルドフックで `packages/wasm/mille.wasm` をパッケージにバンドルします。
リポジトリには **コピーを持たない** ため .wasm ファイルが言語パッケージ数だけ増えません。
