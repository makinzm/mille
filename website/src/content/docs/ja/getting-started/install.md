---
title: インストール
description: mille のインストール方法（cargo / npm / pip / go / バイナリ）
---

## cargo

```sh
cargo install mille
```

## npm

```sh
npm install -g @makinzm/mille
mille check
```

グローバルインストールなしで実行する場合:

```sh
npx @makinzm/mille check
```

Node.js ≥ 18 が必要です。`mille.wasm` を同梱しているため、ネイティブコンパイル不要です。

## go install

```sh
go install github.com/makinzm/mille/packages/go/mille@latest
```

[wazero](https://wazero.io/) 経由で `mille.wasm` を埋め込んだ完全自己完結型バイナリです。

## pip / uv

```sh
# uv（推奨）
uv add --dev mille
uv run mille check

# pip
pip install mille
mille check
```

## バイナリダウンロード

[GitHub Releases](https://github.com/makinzm/mille/releases) からプラットフォーム別のバイナリを取得できます:

| プラットフォーム | アーカイブ |
|---|---|
| Linux x86_64 | `mille-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux arm64 | `mille-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `mille-<version>-x86_64-apple-darwin.tar.gz` |
| macOS arm64 | `mille-<version>-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `mille-<version>-x86_64-pc-windows-msvc.zip` |
