---
title: Go
description: mille configuration for Go projects
---

## Configuration Example

```toml
[project]
name      = "my-go-app"
root      = "."
languages = ["go"]

[resolve.go]
module_name = "github.com/myorg/my-go-app"

[[layers]]
name            = "domain"
paths           = ["domain/**"]
dependency_mode = "opt-in"
allow           = []

[[layers]]
name            = "usecase"
paths           = ["usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]

[[layers]]
name            = "infrastructure"
paths           = ["infrastructure/**"]
dependency_mode = "opt-out"
deny            = []

[[layers]]
name            = "cmd"
paths           = ["cmd/**"]
dependency_mode = "opt-in"
allow           = ["domain", "usecase", "infrastructure"]
```

## `mille init` Behavior

Reads `go.mod` and auto-generates `[resolve.go] module_name`. External packages appear in `external_allow` with full import paths (e.g. `"github.com/cilium/ebpf"`, `"fmt"`, `"net/http"`).

## Import Classification

| Import | Classification |
|---|---|
| `"github.com/myorg/my-go-app/domain"` | Internal (starts with `module_name`) |
| `"fmt"`, `"net/http"` | External (stdlib) |
| `"github.com/gin-gonic/gin"` | External (third-party) |
