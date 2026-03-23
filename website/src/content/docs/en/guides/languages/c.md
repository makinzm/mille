---
title: C
description: mille configuration for C projects
---

## Configuration example

```toml
[project]
name      = "my-c-app"
root      = "."
languages = ["c"]

[[layers]]
name            = "domain"
paths           = ["src/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "usecase"
paths           = ["src/usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "infrastructure"
paths           = ["src/infrastructure/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []
```

## Include classification

| Include | Category |
|---|---|
| `#include "domain/user.h"` | Internal (local header) |
| `#include "create_user.h"` | Internal (same directory) |
| `#include <stdio.h>` | Stdlib |
| `#include <stdlib.h>` | Stdlib |
| `#include <curl/curl.h>` | External (third-party) |

## Classification rules

- `#include "..."` (double quotes) → **Internal** (project headers)
- `#include <...>` (angle brackets) → **Stdlib** if it's a C standard / POSIX header, otherwise **External**

Relative paths (e.g. `../domain/user.h`) are normalized before layer matching.

## Supported file extensions

Both `.c` and `.h` files are analyzed.

## Naming conventions

Symbol (function definitions, struct/enum/union/typedef), Variable (global variables), and Comment are all checked against `name_deny` rules.
