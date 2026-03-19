---
title: TypeScript / JavaScript
description: TypeScript・JavaScript プロジェクトでの mille 設定例
---

## TypeScript の設定例

```toml
[project]
name      = "my-ts-app"
root      = "."
languages = ["typescript"]

[resolve.typescript]
tsconfig = "./tsconfig.json"

[[layers]]
name            = "domain"
paths           = ["domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "usecase"
paths           = ["usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-in"
external_allow  = ["zod"]

[[layers]]
name            = "infrastructure"
paths           = ["infrastructure/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []
```

## JavaScript の設定例

```toml
[project]
languages = ["javascript"]
# [resolve.typescript] は不要
```

## インポートの分類

| インポート | 分類 |
|---|---|
| `import X from "./module"` | 内部 |
| `import X from "../module"` | 内部 |
| `import X from "@/module"` （tsconfig パスエイリアス） | 内部 |
| `import X from "react"` | 外部 |
| `import fs from "node:fs"` | 外部 |

## サブパスインポート

`external_allow = ["vitest"]` と設定すると `"vitest"` も `"vitest/config"` も許可されます。
スコープドパッケージ（`@scope/name/sub`）は `"@scope/name"` で一致します。
