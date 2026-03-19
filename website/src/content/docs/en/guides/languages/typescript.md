---
title: TypeScript / JavaScript
description: mille configuration for TypeScript and JavaScript projects
---

## TypeScript Configuration

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

## JavaScript Configuration

```toml
[project]
languages = ["javascript"]
# No [resolve.typescript] needed
```

## Import Classification

| Import | Classification |
|---|---|
| `import X from "./module"` | Internal |
| `import X from "../module"` | Internal |
| `import X from "@/module"` (tsconfig path alias) | Internal |
| `import X from "react"` | External |
| `import fs from "node:fs"` | External |

## Subpath Imports

`external_allow = ["vitest"]` allows both `"vitest"` and `"vitest/config"`.
Scoped packages (`@scope/name/sub`) are matched by `"@scope/name"`.
