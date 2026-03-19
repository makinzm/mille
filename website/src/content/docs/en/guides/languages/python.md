---
title: Python
description: mille configuration for Python projects
---

## Configuration Example

```toml
[project]
name      = "my-python-app"
root      = "."
languages = ["python"]

[resolve.python]
src_root      = "."
package_names = ["domain", "usecase", "infrastructure"]

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
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "infrastructure"
paths           = ["infrastructure/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []
```

## `src/` Layout (namespace packages)

When using a `src/` layout with imports like `from src.domain.entity import Foo`, `mille init` automatically adds `src` to `package_names`:

```toml
[resolve.python]
src_root      = "."
package_names = ["src"]   # ← auto-set by mille init
```

## Import Classification

| Import | Classification |
|---|---|
| `from .sibling import X` (relative) | Internal |
| `import domain.entity` (matches `package_names`) | Internal |
| `from src.domain.entity import Foo` | Internal (when `src` is in `package_names`) |
| `import os`, `import sqlalchemy` | External |

## Submodule Imports

`external_allow = ["matplotlib"]` correctly allows both `import matplotlib` and `import matplotlib.pyplot`.
