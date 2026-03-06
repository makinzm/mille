# Wasm ビルドガイド

## 概要

`mille` のコアロジック（Rust）を `wasm32-wasip1`（WASI Preview 1）ターゲットでビルドし、
Go ラッパー（wazero）に埋め込みます。
`go install` 後はネットワーク接続不要で動作します。

### .wasm ファイルのリポジトリ構造

| ファイル | 役割 |
|----------|------|
| `packages/go/mille/mille.wasm` | **唯一のコミット済み .wasm**。`packages/go/main.go` が `//go:embed` で直接埋め込む |
| `packages/npm/mille.wasm` | publish 時にのみ生成（`packages/go/mille/mille.wasm` からコピー）。リポジトリにはコミットしない |

`packages/go` は `mille.wasm` を直接 `//go:embed` で埋め込みます。
外部 Go モジュールへの依存はなく、`replace` ディレクティブもないため `go install` がそのまま動作します。

---

## ローカルでの npm ラッパー使用手順

### 前提

| ツール | 用途 |
|--------|------|
| Node.js ≥ 18 | `node:wasi` モジュール（WASI Preview 1 サポート） |
| mille.wasm | `packages/go/mille/mille.wasm`（リポジトリにコミット済み） |

### Step 1: mille.wasm を npm パッケージにコピー

```bash
# リポジトリ root で実行
cp packages/go/mille/mille.wasm packages/npm/mille.wasm
```

> ⚠️ `packages/npm/mille.wasm` はリポジトリにコミットしない（publish 時に生成される）。
> ローカル動作確認のためだけに手動でコピーする。

### Step 2: 動作確認

```bash
# 確認したいプロジェクトのディレクトリで実行
node /path/to/mille/packages/npm/index.js check

# リポジトリ root 自体を確認する例
node packages/npm/index.js check
```

> **注意**: `node` は devbox 経由（volta 管理）を使用すること。
> バージョン要件: Node.js ≥ 18.0.0（`node:wasi` の WASI Preview 1 サポートが必要）。

### 仕組み

`packages/npm/index.js` は Node.js の `node:wasi` モジュールを使って
`mille.wasm`（WASI Preview 1 コマンドモジュール）を実行します。

| 要素 | Node.js WASI 側の担当 |
|------|----------------------|
| ファイルシステム | `preopens: { '/': process.cwd() }` で CWD を WASI root にマウント |
| CLI 引数 | `args: ['mille', ...process.argv.slice(2)]` で伝達 |
| 標準入出力 | Node.js の stdin/stdout/stderr が自動的に接続される |
| 終了コード | `wasi.start()` が `proc_exit(n)` を受けて `process.exit(n)` を呼ぶ |

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
- `packages/go/mille/mille.wasm` — 唯一の canonical Wasm バイナリ（Git 管理）

### Step 2: Go ラッパーをビルド

```bash
cd packages/go/mille
go build -o mille_go .
```

`packages/go/mille/main.go` が `//go:embed mille.wasm` で直接埋め込みます。
外部モジュール依存はありません。

### Step 3: 動作確認

```bash
# packages/go/mille ディレクトリで実行（mille.toml が必要）
./mille_go check

# リポジトリ root の Rust プロジェクトを確認する場合
cd /path/to/your/project
/path/to/mille_go check
```

### テスト実行

```bash
cd packages/go/mille
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
test → build-wasm → dogfood-go  ┐
                 → dogfood-npm  ├→ update-wasm (main push 時のみ)
              dogfood-rust ─────┘
              dogfood-python ───┘
```

| ジョブ | 役割 | 実行タイミング |
|--------|------|--------------|
| `build-wasm` | devbox 経由で wasm をビルドし artifact にアップロード | PR・main push |
| `dogfood-go` | artifact の wasm を `packages/go/` に配置して go test / self-check | PR・main push |
| `dogfood-npm` | artifact の wasm を `packages/go/` → `packages/npm/` にコピーして Node.js WASI self-check | PR・main push |
| `update-wasm` | artifact の wasm で `packages/go/mille/mille.wasm` を上書きコミット | **main push のみ** |

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

### update-wasm ジョブの動作

main に push されたとき、全テストが通った後に自動実行されます。

```bash
git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
# 変化があればコミット（変化なければスキップ）
git add packages/go/mille/mille.wasm
git commit -m "[fix] mille.wasm を CI ビルドで更新 [skip ci]"
git push
```

`[skip ci]` をコミットメッセージに含めることで、bot commit による CI の再トリガーを防ぎます。

---

## .wasm ファイルの Git 管理

### なぜバイナリを Git に含めるのか（3MB の根拠）

`packages/go/mille/mille.wasm`（約 3MB）は Git で追跡されています。
これが **リポジトリ内で唯一の .wasm ファイル** です。

バイナリをリポジトリに含める理由と代替案の比較は以下のとおりです。

| 方式 | `go install` 時の動作 | 外部通信 | ランタイム依存 |
|------|----------------------|----------|---------------|
| **Git 管理（採用）** | そのまま動く | なし | なし |
| GitHub Releases 配布 | 初回に自動ダウンロード | あり | なし |
| 実行時ビルド | Rust + wasi-sdk が必要 | なし | あり |

`go install` が成功した後、エンドユーザーは **追加ツールなし・ネットワークなし** で
`mille check` を実行できる必要があります。
`packages/go/main.go` が `//go:embed mille.wasm` でバイナリに直接埋め込みます。

3MB というサイズについては：

- Rust の release ビルドには標準ライブラリ（`std`）が静的リンクされる
- tree-sitter の C ランタイムが含まれる
- `wasm-opt` による最適化を将来検討できるが、現時点では未適用

### 更新タイミング

Rust コア（`src/`）に変更があった場合は必ず `.wasm` を再ビルドしてコミットしてください。
CI の `build-wasm` ジョブがこれを自動検出します。

```bash
bash scripts/build-wasm.sh
git add packages/go/mille/mille.wasm
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
