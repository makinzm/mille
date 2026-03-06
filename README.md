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
| Python support | ✅ |
| TypeScript / JavaScript support | planned |

## How to Install

### cargo (Rust users)

```sh
cargo install mille
```

### pip / uv (Python users)

```sh
# uv (recommended)
uv add --dev mille
uv run mille check

# pip
pip install mille
mille check
```

The Python package is a native extension built with [maturin](https://github.com/PyO3/maturin) (PyO3). It provides both a CLI (`mille check`) and a Python API (`import mille; mille.check(...)`).

### go install

```sh
go install github.com/makinzm/mille/packages/go@latest
```

The Go wrapper embeds `mille.wasm` (the compiled Rust core) and runs it via [wazero](https://wazero.io/) — a zero-dependency WebAssembly runtime. No network access or caching required; the binary is fully self-contained.

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

**Python project example:**

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

### `[resolve.python]`

| Key | Description |
|---|---|
| `src_root` | Root directory of the Python source tree (relative to `mille.toml`) |
| `package_names` | List of your own package names (used to classify absolute imports as internal). e.g. `["domain", "usecase", "infrastructure"]` |

**How Python imports are classified:**

| Import | Classification |
|---|---|
| `from .sibling import X` (relative) | Internal |
| `import domain.entity` (matches a `package_names` entry) | Internal |
| `import os`, `import sqlalchemy` (others) | External |

## Python API

In addition to the CLI, the Python package exposes a programmatic API:

```python
import mille

# Run architecture check and get a result object
result = mille.check("path/to/mille.toml")  # defaults to "mille.toml"

print(f"violations: {len(result.violations)}")
for v in result.violations:
    print(f"  {v.file}:{v.line}  {v.from_layer} -> {v.to_layer}  ({v.import_path})")

for stat in result.layer_stats:
    print(f"  {stat.name}: {stat.file_count} file(s), {stat.violation_count} violation(s)")
```

**Types exposed:**

| Class | Attributes |
|---|---|
| `CheckResult` | `violations: list[Violation]`, `layer_stats: list[LayerStat]` |
| `Violation` | `file`, `line`, `from_layer`, `to_layer`, `import_path`, `kind` |
| `LayerStat` | `name`, `file_count`, `violation_count` |

## How it Works

mille uses [tree-sitter](https://tree-sitter.github.io/) for AST-based import extraction — no regex heuristics. The core engine is language-agnostic; language-specific logic is isolated to the `parser` and `resolver` layers.

```
mille.toml
    │
    ▼
Layer definitions
    │
Source files (*.rs, *.go, *.py, ...)
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
