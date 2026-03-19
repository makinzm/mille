---
title: Rust
description: mille configuration for Rust projects
---

## Configuration Example

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

## Import Classification

| Import | Classification |
|---|---|
| `use crate::domain::...` | Internal (starts with `<crate_name>::`) |
| `use super::...` | Internal |
| `use serde::Serialize` | External |

## Dogfooding

mille checks its own source code on every CI run. See [mille.toml](https://github.com/makinzm/mille/blob/main/mille.toml) for the self-referential configuration.
