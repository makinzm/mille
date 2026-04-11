---
title: Elixir
description: mille configuration for Elixir projects
---

## Configuration Example

```toml
[project]
name      = "my-elixir-app"
root      = "."
languages = ["elixir"]

[resolve.elixir]
app_name = "MyApp"

[[layers]]
name            = "domain"
paths           = ["lib/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "usecase"
paths           = ["lib/usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "infrastructure"
paths           = ["lib/infrastructure/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []
```

## `app_name` Setting

Set `[resolve.elixir] app_name` to the PascalCase module name corresponding to `:app` in `mix.exs` (e.g. `MyApp`).

```toml
[resolve.elixir]
app_name = "MyApp"   # when :app is :my_app in mix.exs
```

## Import Classification

All four Elixir directives are analyzed as dependencies.

| Directive | Example | Classification |
|---|---|---|
| `alias` | `alias MyApp.Domain.User` | Internal (matches `app_name`) |
| `alias` with `as:` | `alias MyApp.Domain.User, as: U` | Internal |
| `import` | `import Enum` | External |
| `require` | `require Logger` | External |
| `use` | `use Ecto.Schema` | External |

## Module-to-Path Resolution

Internal module paths are resolved using the following rule:

`MyApp.Domain.User` → strip `app_name` → `Domain.User` → lowercase → `lib/domain/user.ex`
