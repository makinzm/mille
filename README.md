# mille

> Architecture Checker — static analysis CLI for layered architecture rules

`mille` is a CLI tool that enforces **dependency rules for layered architectures** (Clean Architecture, Onion Architecture, Hexagonal Architecture, etc.).

It is implemented in Rust, supports multiple languages from a single TOML config, and is designed to run in CI/CD pipelines.

## Features

| Feature | Status |
|---|---|
| Internal layer dependency check (`dependency_mode`) | ✅ |
| External library dependency check (`external_mode`) | ✅ |
| DI entrypoint method call check (`allow_call_patterns`) | ✅ |
| Rust support | ✅ |
| Go support | ✅ |
| TypeScript / JavaScript support | planned |
| Python support | planned |

## How to Install

### cargo (Rust users)

```sh
cargo install mille
```

### go install

```sh
go install github.com/makinzm/mille/packages/go@latest
```

On first run, the Go wrapper downloads the pre-built binary from GitHub Releases and caches it at `~/.mille/bin/<version>/mille`. Subsequent runs use the cached binary directly — performance is equivalent to `cargo install`.

### Direct binary download

Pre-built binaries for each platform are available on [GitHub Releases](https://github.com/makinzm/mille/releases):

| Platform | Archive |
|---|---|
| Linux x86_64 | `mille-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux arm64 | `mille-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `mille-<version>-x86_64-apple-darwin.tar.gz` |
| macOS arm64 | `mille-<version>-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `mille-<version>-x86_64-pc-windows-msvc.zip` |

```sh
# Example: Linux x86_64
curl -L https://github.com/makinzm/mille/releases/latest/download/mille-<version>-x86_64-unknown-linux-gnu.tar.gz | tar xz
./mille check
```

## Quick Start

### 1. Create `mille.toml`

Place `mille.toml` in your project root:

**Rust project example:**

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

**Go project example:**

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

### 2. Run `mille check`

```sh
mille check
```

Exit codes:

| Code | Meaning |
|---|---|
| `0` | No violations |
| `1` | One or more errors detected |
| `3` | Configuration file error |

## Configuration Reference

### `[project]`

| Key | Description |
|---|---|
| `name` | Project name |
| `root` | Root directory for analysis |
| `languages` | List of languages to check (e.g. `["rust", "go"]`) |

### `[[layers]]`

| Key | Description |
|---|---|
| `name` | Layer name |
| `paths` | Glob patterns for files belonging to this layer |
| `dependency_mode` | `"opt-in"` (deny all except `allow`) or `"opt-out"` (allow all except `deny`) |
| `allow` | Layers allowed as dependencies (when `dependency_mode = "opt-in"`) |
| `deny` | Layers forbidden as dependencies (when `dependency_mode = "opt-out"`) |
| `external_mode` | `"opt-in"` or `"opt-out"` for external library usage |
| `external_allow` | Regex patterns of allowed external packages (when `external_mode = "opt-in"`) |
| `external_deny` | Regex patterns of forbidden external packages (when `external_mode = "opt-out"`) |

### `[[layers.allow_call_patterns]]`

Restricts which methods may be called on a given layer's types. Only valid on the `main` layer.

| Key | Description |
|---|---|
| `callee_layer` | The layer whose methods are being restricted |
| `allow_methods` | List of method names that are permitted |

### `[resolve.go]`

| Key | Description |
|---|---|
| `module_name` | Go module name (matches the module path in `go.mod`) |

## How it Works

mille uses [tree-sitter](https://tree-sitter.github.io/) for AST-based import extraction — no regex heuristics. The core engine is language-agnostic; language-specific logic is isolated to the `parser` and `resolver` layers.

```
mille.toml
    │
    ▼
Layer definitions
    │
Source files (*.rs, *.go, ...)
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
