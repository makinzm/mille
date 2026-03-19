---
title: Java
description: mille configuration for Java projects (Maven / Gradle)
---

## Configuration Example

```toml
[project]
name      = "my-java-app"
root      = "."
languages = ["java"]

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
external_allow  = ["java.util.List", "java.util.Map"]
```

## `mille init` Behavior

Reads `pom.xml` (Maven) or `build.gradle` + `settings.gradle` (Gradle) and auto-sets `module_name`. Layer paths use `**/layer/**` globs to match regardless of source root depth (e.g. Maven's `src/main/java/com/example/myapp/domain/`).

### Example Output

```
Detected languages: java
Scanning imports...

Inferred layer structure:
  domain               ← (no internal dependencies)
  infrastructure       → domain
    external: java.util.List
  usecase              → domain
  main                 → domain, infrastructure, usecase

Generated 'mille.toml'
```

## Import Classification

| Import | Classification |
|---|---|
| `import com.example.myapp.domain.User` | Internal (starts with `module_name`) |
| `import static com.example.myapp.util.Helper.method` | Internal |
| `import java.util.List` | External (stdlib) |
| `import org.springframework.*` | External |

Wildcard imports (`import java.util.*`) are not yet supported by the parser.
