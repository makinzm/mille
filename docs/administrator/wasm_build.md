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
# リポジトリ root で実行（devbox shell 内、または devbox run -- 経由）
devbox run -- bash scripts/build-wasm.sh
```

devbox が `rust-toolchain.toml` の Rust バージョン（1.85.0）を使ってビルドします。
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

`.github/workflows/ci.yml` のジョブ構成:

```
test → build-wasm → dogfood-go ┐
                               ├→ update-wasm (main push 時のみ)
              dogfood-rust ────┘
```

| ジョブ | 役割 | 実行タイミング |
|--------|------|--------------|
| `build-wasm` | devbox 経由で wasm をビルドし artifact にアップロード | PR・main push |
| `dogfood-go` | artifact の wasm を使って go test / self-check | PR・main push |
| `update-wasm` | artifact の wasm でコミット済みファイルを上書きコミット | **main push のみ** |

### なぜバイナリ比較による stale 検知をしないか

Rust の wasm ビルドはビット単位の再現性（reproducibility）が保証されません。
同一ソース・同一 Rust バージョン（1.85.0）・同一 wasi-sdk-30 でも、
LLVM の最適化パス順序や host 環境の差異で異なるバイナリが生成されます。

代わりに以下の方針を採用します：
- `dogfood-go` が CI ビルドの wasm でテストを実行し、**動作の正しさ**を検証する
- main マージ後は `update-wasm` が CI ビルド結果で自動上書きコミットし、
  リポジトリの wasm を常に CI 環境で生成したものに保つ

### なぜ devbox 経由で実行するか

`dtolnay/rust-toolchain@stable` で直接 Rust をインストールすると、その時点の `stable`
（例: 1.93.1）が使われ、`rust-toolchain.toml` の固定バージョン（1.85.0）が無視されます。

CI で実行されるコマンド（`build-wasm` ジョブの核心部分）:

```bash
# devbox が rust-toolchain.toml を参照して Rust 1.85.0 を使用する
devbox run -- bash scripts/build-wasm.sh
```

ローカルでの等価コマンド:

```bash
# devbox shell 外から実行する場合
devbox run -- bash scripts/build-wasm.sh
```

### update-wasm ジョブの動作

main に push されたとき、全テストが通った後に自動実行されます。

```bash
# CI 内で実行される処理のイメージ
git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
# 変化があればコミット（変化なければスキップ）
git add packages/wasm/mille.wasm
git commit -m "[fix] mille.wasm を CI ビルドで更新 [skip ci]"
git push
```

`[skip ci]` をコミットメッセージに含めることで、bot commit による CI の再トリガーを防ぎます。

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

## CompilationCache（起動高速化）

wazero は起動のたびに `.wasm` を機械語へ変換（JIT コンパイル）します。
`CompilationCache` を使うと変換結果をファイルに保存し、2 回目以降の起動でキャッシュを再利用できます。

### キャッシュディレクトリ

| OS | パス |
|----|------|
| Linux | `~/.cache/mille/wazero/` |
| macOS | `~/Library/Caches/mille/wazero/` |
| Windows | `%LOCALAPPDATA%\mille\wazero\` |

`packages/go` の Go ラッパーは起動時に自動でキャッシュを作成・利用します。
ディレクトリが存在しない場合は自動で作成されます。
キャッシュの作成に失敗した場合はキャッシュなしで動作します（フォールバック）。

### キャッシュのクリア

起動が遅くなった・キャッシュが壊れた場合は削除してください：

```bash
# Linux
rm -rf ~/.cache/mille/wazero/

# macOS
rm -rf ~/Library/Caches/mille/wazero/
```

### 実装

`packages/go/wasm_cache.go` の `compilationCacheDir()` と `newRuntime()` が担当します。
`newRuntime()` は `CompilationCache` の作成に失敗しても必ず `wazero.Runtime` を返すため、
キャッシュの有無に関わらず動作します。

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
