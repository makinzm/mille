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

### 1. Generate `mille.toml` with `mille init`

```sh
mille init
```

`mille init` analyzes actual import statements in your source files to infer layer structure and dependencies — no predetermined naming conventions needed. It prints the inferred dependency graph before writing the config:

```
Detected languages: rust
Scanning imports...
Using layer depth: 2

Inferred layer structure:
  domain               ← (no internal dependencies)
  usecase              → domain
    external: anyhow
  infrastructure       → domain
    external: serde, tokio

Generated 'mille.toml'
```

| Flag | Default | Description |
|---|---|---|
| `--output <path>` | `mille.toml` | Write config to a custom path |
| `--force` | false | Overwrite an existing file without prompting |
| `--depth <N>` | auto | Layer detection depth from project root |

**`--depth` and auto-detection**: `mille init` automatically finds the right layer depth by trying depths 1–6, skipping common source-layout roots (`src`, `lib`, `app`, etc.), and selecting the first depth that yields 2–8 candidate layers. For a project with `src/domain/entity`, `src/domain/repository`, `src/usecase/` — depth 2 is chosen, rolling `entity` and `repository` up into `domain`. Use `--depth N` to override when auto-detection picks the wrong level.

The generated config includes `allow` (inferred internal dependencies) and `external_allow` (detected external packages) per layer. After generating, review the config and run `mille check` to see results.

### 2. (Or) Create `mille.toml` manually

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

### 2. Visualize with `mille analyze`

Before enforcing rules, you can inspect the actual dependency graph:

```sh
mille analyze                  # human-readable terminal output (default)
mille analyze --format json    # machine-readable JSON graph
mille analyze --format dot     # Graphviz DOT (pipe to: dot -Tsvg -o graph.svg)
mille analyze --format svg     # self-contained SVG image (open in a browser)
```

Example SVG output (dark theme, green edges):

```sh
mille analyze --format svg > graph.svg && open graph.svg
```

`mille analyze` always exits `0` — it only visualizes, never enforces rules.

### 3. Run `mille check`

```sh
mille check
```

Output formats:

```sh
mille check                          # human-readable terminal output (default)
mille check --format github-actions  # GitHub Actions annotations (::error file=...)
mille check --format json            # machine-readable JSON
```

Fail threshold:

```sh
mille check                         # exit 1 on error-severity violations only (default)
mille check --fail-on warning       # exit 1 on any violation (error or warning)
mille check --fail-on error         # explicit default — same as no flag
```

Exit codes:

| Code | Meaning |
|---|---|
| `0` | No violations (or only warnings without `--fail-on warning`) |
| `1` | One or more violations at the configured fail threshold |
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

### `[ignore]`

Exclude files from the architecture check entirely, or suppress violations for test/mock files.

| Key | Description |
|---|---|
| `paths` | Glob patterns — matching files are excluded from collection and not counted in layer stats |
| `test_patterns` | Glob patterns — matching files are still counted in layer stats but their imports are not violation-checked |

```toml
[ignore]
paths         = ["**/mock/**", "**/generated/**", "**/testdata/**"]
test_patterns = ["**/*_test.go", "**/*.spec.ts", "**/*.test.ts"]
```

**When to use `paths` vs `test_patterns`:**

- `paths`: Files that should not be analyzed at all (generated code, vendor directories, mocks)
- `test_patterns`: Test files that intentionally import across layers (e.g., integration tests that import both domain and infrastructure)

### `[severity]`

Control the severity level of each violation type. Violations can be `"error"`, `"warning"`, or `"info"`.

| Key | Default | Description |
|---|---|---|
| `dependency_violation` | `"error"` | Layer dependency rule violated |
| `external_violation` | `"error"` | External library rule violated |
| `call_pattern_violation` | `"error"` | DI entrypoint method call rule violated |
| `unknown_import` | `"warning"` | Import that could not be classified |

```toml
[severity]
dependency_violation   = "warning"   # treat as warning for gradual adoption
external_violation     = "error"
call_pattern_violation = "error"
unknown_import         = "warning"
```

Use `--fail-on warning` to exit 1 even for warnings when integrating into CI gradually.

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
