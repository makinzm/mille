---
title: Rust
description: Rust プロジェクトでの mille 設定例
---

## mille.toml の設定例

```toml
[project]
name      = "my-app"
root      = "."
languages = ["rust"]

[[layers]]
name            = "domain"
paths           = ["src/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-in"
external_allow  = []

[[layers]]
name            = "usecase"
paths           = ["src/usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-in"
external_allow  = []

[[layers]]
name            = "infrastructure"
paths           = ["src/infrastructure/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "main"
paths           = ["src/main.rs"]
dependency_mode = "opt-in"
allow           = ["domain", "infrastructure", "usecase"]
external_mode   = "opt-in"
external_allow  = ["clap"]

  [[layers.allow_call_patterns]]
  callee_layer  = "infrastructure"
  allow_methods = ["new", "build", "create", "init", "setup"]
```

## インポートの分類

| インポート | 分類 |
|---|---|
| `use crate::domain::...` | 内部（`<crate_name>::` で始まるもの） |
| `use super::...` | 内部 |
| `use serde::Serialize` | 外部 |

## `mille init` の動作

Cargo.toml からクレート名を自動検出し、`crate::` プレフィックスを Internal に分類します。

```sh
mille init
# → mille.toml を自動生成
```

## dogfooding

mille 自身も mille でアーキテクチャを検査しています。設定例は [mille.toml](https://github.com/makinzm/mille/blob/main/mille.toml) を参照してください。
