---
title: Layer Configuration
description: "All [[layers]] options and the opt-in / opt-out model"
---

import { Aside } from '@astrojs/starlight/components';

## `[[layers]]`

| Key | Description |
|---|---|
| `name` | Layer name |
| `paths` | Glob patterns for files in this layer |
| `dependency_mode` | `"opt-in"` (only `allow`-listed layers permitted) or `"opt-out"` (all except `deny`-listed) |
| `allow` | Permitted dependency layers (`dependency_mode = "opt-in"`) |
| `deny` | Forbidden dependency layers (`dependency_mode = "opt-out"`) |
| `external_mode` | `"opt-in"` or `"opt-out"` for external libraries |
| `external_allow` | Permitted external packages (`external_mode = "opt-in"`) |
| `external_deny` | Forbidden external packages (`external_mode = "opt-out"`) |

## opt-in / opt-out Model

| Mode | Default | What to write | Best for |
|---|---|---|---|
| `opt-in` | All denied | List permitted items in `allow` / `external_allow` | domain, usecase, presentation |
| `opt-out` | All allowed | List forbidden items in `deny` / `external_deny` | infrastructure |

### Internal dependency example

```toml
[[layers]]
name            = "domain"
dependency_mode = "opt-in"
allow           = []               # depends on nothing

[[layers]]
name            = "usecase"
dependency_mode = "opt-in"
allow           = ["domain"]       # only domain allowed

[[layers]]
name            = "infrastructure"
dependency_mode = "opt-out"        # all internal layers OK
deny            = []
```

### External library example

`external_allow` / `external_deny` values are regular expressions.

```toml
[[layers]]
name           = "domain"
external_mode  = "opt-in"
external_allow = []                          # no external libs

[[layers]]
name           = "usecase"
external_mode  = "opt-in"
external_allow = ["serde", "uuid", "chrono"]

[[layers]]
name           = "infrastructure"
external_mode  = "opt-out"                   # anything goes
external_deny  = []
```

## `[[layers.allow_call_patterns]]`

Restricts which methods may be called on a given layer's types from the DI entrypoint.

<Aside type="caution">
`allow_call_patterns` is only valid on the `main` layer (or equivalent DI entrypoint). Placing it on other layers causes a configuration error.
</Aside>

| Key | Description |
|---|---|
| `callee_layer` | The layer whose methods are being restricted |
| `allow_methods` | List of permitted method names |

```toml
[[layers]]
name            = "main"
paths           = ["src/main.rs"]
dependency_mode = "opt-in"
allow           = ["domain", "infrastructure", "usecase"]

  [[layers.allow_call_patterns]]
  callee_layer  = "infrastructure"
  allow_methods = ["new", "build", "create", "init", "setup"]
```

This detects violations like:

```rust
// OK: instance creation (matches allow_methods)
let repo = UserRepositoryImpl::new();

// VIOLATION: direct business logic call
repo.find_user(1);  // ❌ not in allow_methods
```
