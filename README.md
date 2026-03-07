# mille

> Like a mille crêpe — your architecture, one clean layer at a time.

```
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  presentation
  · · · · · · · · · · · · · · · · · ·  (deps only flow inward)
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  infrastructure
  · · · · · · · · · · · · · · · · · ·
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  usecase
  · · · · · · · · · · · · · · · · · ·
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  domain
```

`mille` is a static analysis CLI that enforces **dependency rules for layered architectures** — Clean Architecture, Onion Architecture, Hexagonal Architecture, and more.

One TOML config. Rust-powered. CI-ready. Supports multiple languages from a single config file.

## What it checks

| Check | Rust | Go | TypeScript | JavaScript | Python |
|---|:---:|:---:|:---:|:---:|:---:|
| Layer dependency rules (`dependency_mode`) | ✅ | ✅ | ✅ | ✅ | ✅ |
| External library rules (`external_mode`) | ✅ | ✅ | ✅ | ✅ | ✅ |
| DI method call rules (`allow_call_patterns`) | ✅ | ✅ | ✅ | ✅ | ✅ |

## Install

### cargo

```sh
cargo install mille
```

### npm

```sh
npm install -g @makinzm/mille
mille check
```

Or without installing globally:

```sh
npx @makinzm/mille check
```

Requires Node.js ≥ 18. Bundles `mille.wasm` — no native compilation needed.

### go install

```sh
go install github.com/makinzm/mille/packages/go/mille@latest
```

Embeds `mille.wasm` via [wazero](https://wazero.io/) — fully self-contained binary.

### pip / uv

```sh
# uv (recommended)
uv add --dev mille
uv run mille check

# pip
pip install mille
mille check
```

### Binary download

Pre-built binaries are on [GitHub Releases](https://github.com/makinzm/mille/releases):

| Platform | Archive |
|---|---|
| Linux x86_64 | `mille-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux arm64 | `mille-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `mille-<version>-x86_64-apple-darwin.tar.gz` |
| macOS arm64 | `mille-<version>-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `mille-<version>-x86_64-pc-windows-msvc.zip` |

## Quick Start

### 1. Create `mille.toml`

Place `mille.toml` in your project root:

**Rust:**

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

**TypeScript / JavaScript:**

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

> Use `languages = ["javascript"]` for plain `.js` / `.jsx` projects (no `[resolve.typescript]` needed).

**Go:**

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

**Python:**

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

### 2. Run `mille check`

```sh
mille check
```

Exit codes:

| Code | Meaning |
|---|---|
| `0` | No violations |
| `1` | One or more violations detected |
| `3` | Configuration file error |

## Configuration Reference

### `[project]`

| Key | Description |
|---|---|
| `name` | Project name |
| `root` | Root directory for analysis |
| `languages` | Languages to check: `"rust"`, `"go"`, `"typescript"`, `"javascript"`, `"python"` |

### `[[layers]]`

| Key | Description |
|---|---|
| `name` | Layer name |
| `paths` | Glob patterns for files in this layer |
| `dependency_mode` | `"opt-in"` (deny all except `allow`) or `"opt-out"` (allow all except `deny`) |
| `allow` | Allowed layers (when `dependency_mode = "opt-in"`) |
| `deny` | Forbidden layers (when `dependency_mode = "opt-out"`) |
| `external_mode` | `"opt-in"` or `"opt-out"` for external library usage |
| `external_allow` | Allowed external packages (when `external_mode = "opt-in"`) |
| `external_deny` | Forbidden external packages (when `external_mode = "opt-out"`) |

### `[[layers.allow_call_patterns]]`

Restricts which methods may be called on a given layer's types. Only valid on the `main` layer (or equivalent DI entrypoint).

| Key | Description |
|---|---|
| `callee_layer` | The layer whose methods are being restricted |
| `allow_methods` | List of method names that are permitted |

### `[resolve.typescript]`

| Key | Description |
|---|---|
| `tsconfig` | Path to `tsconfig.json`. mille reads `compilerOptions.paths` and resolves path aliases (e.g. `@/*`) as internal imports. |

**How TypeScript / JavaScript imports are classified:**

| Import | Classification |
|---|---|
| `import X from "./module"` | Internal |
| `import X from "../module"` | Internal |
| `import X from "@/module"` (path alias in `tsconfig.json`) | Internal |
| `import X from "react"` | External |
| `import fs from "node:fs"` | External |

### `[resolve.go]`

| Key | Description |
|---|---|
| `module_name` | Go module name (matches `go.mod`) |

### `[resolve.python]`

| Key | Description |
|---|---|
| `src_root` | Root directory of the Python source tree (relative to `mille.toml`) |
| `package_names` | Your package names — imports starting with these are classified as internal. e.g. `["domain", "usecase"]` |

**How Python imports are classified:**

| Import | Classification |
|---|---|
| `from .sibling import X` (relative) | Internal |
| `import domain.entity` (matches `package_names`) | Internal |
| `import os`, `import sqlalchemy` | External |

## How it Works

mille uses [tree-sitter](https://tree-sitter.github.io/) for AST-based import extraction — no regex heuristics.

```
mille.toml
    │
    ▼
Layer definitions
    │
Source files (*.rs, *.go, *.py, *.ts, *.js, ...)
    │ tree-sitter parse
    ▼
RawImport list
    │ Resolver (stdlib / internal / external)
    ▼
ResolvedImport list
    │ ViolationDetector
    ▼
Violations → terminal output
```

## Dogfooding

mille checks its own source code on every CI run:

```sh
mille check   # uses ./mille.toml
```

See [mille.toml](./mille.toml) for the architecture rules applied to mille itself.

## Documentation

- [spec.md](./spec.md) — Full specification (in Japanese)
- [docs/TODO.md](./docs/TODO.md) — Development roadmap
