---
title: Kotlin
description: mille configuration for Kotlin projects
---

## Configuration Example

```toml
[project]
name      = "my-kotlin-app"
root      = "."
languages = ["kotlin"]

[resolve.java]
module_name = "com.example.myapp"

[[layers]]
name            = "domain"
paths           = ["src/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-out"

[[layers]]
name            = "usecase"
paths           = ["src/usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-out"

[[layers]]
name            = "infrastructure"
paths           = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-in"
external_allow  = ["kotlinx.coroutines"]
```

## Difference from Java

Use `languages = ["kotlin"]`, but import resolution shares the `[resolve.java]` section.

## Import Classification

| Import | Classification |
|---|---|
| `import com.example.myapp.domain.User` | Internal (starts with `module_name`) |
| `import kotlinx.coroutines.launch` | External |
| `import java.util.UUID` | External (stdlib) |

## Gradle Deep Layout

For deep Gradle layouts, use `**/layer/**` globs:

```toml
[[layers]]
name  = "domain"
paths = ["**/domain/**"]   # matches src/main/kotlin/com/example/domain/ too
```
