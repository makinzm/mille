---
title: Installation
description: How to install mille (cargo / npm / pip / go / binary)
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

Or without global install:

```sh
npx @makinzm/mille check
```

Requires Node.js ≥ 18. Bundles `mille.wasm` — no native compilation needed.

## go install

```sh
go install github.com/makinzm/mille/packages/go/mille@latest
```

Embeds `mille.wasm` via [wazero](https://wazero.io/) — fully self-contained binary.

## pip / uv

```sh
# uv (recommended)
uv add --dev mille
uv run mille check

# pip
pip install mille
mille check
```

## Binary Download

Pre-built binaries are available on [GitHub Releases](https://github.com/makinzm/mille/releases):

| Platform | Archive |
|---|---|
| Linux x86_64 | `mille-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux arm64 | `mille-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `mille-<version>-x86_64-apple-darwin.tar.gz` |
| macOS arm64 | `mille-<version>-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `mille-<version>-x86_64-pc-windows-msvc.zip` |
