# mille

> Like a mille crêpe — your architecture, one clean layer at a time.

`mille` is a static analysis CLI that enforces **dependency rules for layered architectures** — Clean Architecture, Onion Architecture, Hexagonal Architecture, and more.

One TOML config. Rust-powered. CI-ready. Supports multiple languages from a single config file.

## What it checks

**Languages:** Rust, Go, TypeScript, JavaScript, Python, Java, Kotlin, PHP, C

| Check | Description |
|---|---|
| `dependency_mode` | Layer dependency rules — control which layers can import from which |
| `external_mode` | External library rules — restrict third-party package usage per layer |
| `allow_call_patterns` | DI method call rules — limit which methods may be called on injected types |
| `name_deny` | Naming convention rules — forbid infrastructure keywords in domain/usecase |

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
mille init ./path/to/project    # specify target directory
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

**Naming in monorepos**: When multiple sub-projects contain a directory with the same name (e.g. `crawler/src/domain` and `server/src/domain`), `mille init` gives each its own layer with a distinguishing prefix (`crawler_domain`, `server_domain`). Merging is left to you.

**Excluded paths**: `mille check` automatically skips `.venv`, `venv`, `node_modules`, `target`, `dist`, `build`, and similar build/dependency directories, so generated `paths` patterns like `apps/**` are safe to use.

**Python submodule imports**: `external_allow = ["matplotlib"]` correctly allows both `import matplotlib` and `import matplotlib.pyplot`.

**Python `src/` layout (namespace packages)**: When your project uses a `src/` layout and imports like `from src.domain.entity import Foo`, `mille init` detects that `src` is used as a top-level import prefix and automatically adds it to `package_names`. This means `from src.domain...` is classified as Internal and `src` does not appear in `external_allow`. Cross-layer imports like `from src.domain.entity import Foo` (written in `src/infrastructure/`) are correctly resolved to the `src/domain` layer and appear as an `allow` dependency in the generated `mille.toml`. Files at the project root of a sub-tree (e.g. `src/main.py`) are included in the `src` layer rather than being silently skipped.

**Go projects**: `mille init` reads `go.mod` and generates `[resolve.go] module_name` automatically — internal module imports are classified correctly during `mille check`. External packages appear in `external_allow` with their full import paths (e.g. `"github.com/cilium/ebpf"`, `"fmt"`, `"net/http"`).

**TypeScript/JavaScript subpath imports**: `external_allow = ["vitest"]` correctly allows both `import "vitest"` and `import "vitest/config"`. Scoped packages (`@scope/name/sub`) are matched by `"@scope/name"`.

**Java/Kotlin projects**: `mille init` uses `package` declarations — not directory depth — to detect layers. This works correctly for Maven's `src/main/java/com/example/myapp/domain/` as well as flat `src/domain/` layouts. `pom.xml` (Maven) and `build.gradle` + `settings.gradle` (Gradle) are read automatically to generate `[resolve.java] module_name`. Layer paths use `**/layer/**` globs so `mille check` matches regardless of the source root depth.

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

**Java / Kotlin:**

```toml
[project]
name      = "my-java-app"
root      = "."
languages = ["java"]   # or ["kotlin"] for Kotlin projects

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

> `module_name` is the base package of your project (e.g. `com.example.myapp`). Imports starting with this prefix are classified as **Internal** and matched against layer globs. All other imports (including `java.util.*` stdlib) are classified as **External** and subject to `external_allow` / `external_deny` rules.

### 2. Visualize with `mille analyze`

Before enforcing rules, you can inspect the actual dependency graph:

```sh
mille analyze                          # human-readable terminal output (default)
mille analyze --format json            # machine-readable JSON graph
mille analyze --format dot             # Graphviz DOT (pipe to: dot -Tsvg -o graph.svg)
mille analyze --format svg             # self-contained SVG image (open in a browser)
mille analyze ./path/to/project        # specify target directory
```

Example SVG output (dark theme, green edges):

```sh
mille analyze --format svg > graph.svg && open graph.svg
```

`mille analyze` always exits `0` — it only visualizes, never enforces rules.

### 3. Inspect external dependencies with `mille report external`

```sh
mille report external                              # human-readable table (default)
mille report external --format json                # machine-readable JSON
mille report external --output report.json --format json   # write to file
mille report external ./path/to/project            # specify target directory
```

Shows which external packages each layer actually imports — useful for auditing `external_allow` lists or documenting your dependency footprint.

Example output:

```
External Dependencies by Layer

  domain          (none)
  usecase         (none)
  infrastructure  database/sql
  cmd             fmt, os
```

`mille report external` always exits `0` — it only reports, never enforces rules.

### 4. Run `mille check`

```sh
mille check
mille check ./path/to/project        # specify target directory
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
| `languages` | Languages to check: `"rust"`, `"go"`, `"typescript"`, `"javascript"`, `"python"`, `"java"`, `"kotlin"`, `"php"`, `"c"` |

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
| `name_deny` | Forbidden keywords for naming convention check (case-insensitive partial match) |
| `name_allow` | Substrings to strip before `name_deny` check (e.g. `"category"` prevents `"go"` match inside it) |
| `name_targets` | Targets to check: `"file"`, `"symbol"`, `"variable"`, `"comment"`, `"string_literal"`, `"identifier"` (default: all) |
| `name_deny_ignore` | Glob patterns for files to exclude from naming checks (e.g. `"**/test_*.rs"`) |

#### Naming Convention Check (`name_deny`)

Forbid infrastructure-specific keywords from appearing in a layer's names.

```toml
[[layers]]
name = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"
external_deny = []

# Usecase layer must not reference specific infrastructure technologies
name_deny    = ["gcp", "aws", "azure", "mysql", "postgres"]
name_allow   = ["category"]   # "category" contains "go" but should not be flagged
name_targets = ["file", "symbol", "variable", "comment", "string_literal", "identifier"]  # default: all targets
name_deny_ignore = ["**/test_*.rs", "tests/**"]  # exclude test files from naming checks
```

**Rules:**
- Case-insensitive (`GCP` = `gcp` = `Gcp`)
- Partial match (`ManageGcp` also matches `gcp`)
- `name_allow` strips listed substrings before matching (e.g. `"category"` prevents false positive on `"go"`)
- `name_deny_ignore` excludes files matching glob patterns from naming checks entirely
- `name_targets` restricts which entity types are checked:
  - `"file"`: file basename (e.g. `aws_client.rs`)
  - `"symbol"`: function, class, struct, enum, trait, interface, type alias names
  - `"variable"`: variable, const, let, static declaration names
  - `"comment"`: inline comment content
  - `"string_literal"`: string literal content
  - `"identifier"`: attribute/field access identifiers (e.g. `gcp` in `cfg.gcp.bucket`)
- Supported languages: Rust, TypeScript, JavaScript, Python, Go, Java, Kotlin, PHP, C
- Severity is controlled by `severity.naming_violation` (default: `"error"`)

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
| `naming_violation` | `"error"` | Naming convention rule violated (`name_deny`) |

```toml
[severity]
dependency_violation   = "warning"   # treat as warning for gradual adoption
external_violation     = "error"
call_pattern_violation = "error"
unknown_import         = "warning"
naming_violation       = "warning"   # treat as warning while rolling out naming rules
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
| `module_name` | Go module name (matches `go.mod`). `mille init` generates this automatically from `go.mod`. |

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

### `[resolve.java]`

| Key | Description |
|---|---|
| `module_name` | Base package of your project (e.g. `com.example.myapp`). Imports starting with this prefix are classified as Internal. Generated automatically by `mille init`. |
| `pom_xml` | Path to `pom.xml` (relative to `mille.toml`). `groupId.artifactId` is used as `module_name` when `module_name` is not set. |
| `build_gradle` | Path to `build.gradle` (relative to `mille.toml`). `group` + `rootProject.name` from `settings.gradle` is used as `module_name` when `module_name` is not set. |

**How Java imports are classified:**

| Import | Classification |
|---|---|
| `import com.example.myapp.domain.User` (starts with `module_name`) | Internal |
| `import static com.example.myapp.util.Helper.method` | Internal |
| `import java.util.List`, `import org.springframework.*` | External |

> Both regular and static imports are supported. Wildcard imports (`import java.util.*`) are not yet extracted by the parser.

### `[resolve.php]`

| Key | Description |
|---|---|
| `namespace` | Base namespace of your project (e.g. `App`). Imports starting with this prefix are classified as Internal. |
| `composer_json` | Path to `composer.json` (relative to `mille.toml`). The first PSR-4 key in `autoload.psr-4` is used as the base namespace when `namespace` is not set. |

**How PHP imports are classified:**

| Import | Classification |
|---|---|
| `use App\Models\User` (starts with `namespace`) | Internal |
| `use App\Services\{Auth, Logger}` (group use, expanded) | Internal |
| `use function App\Helpers\format_date` | Internal |
| `use DateTime`, `use PDO`, `use Exception` | Stdlib |
| `use Illuminate\Http\Request` | External |

> Supported use forms: simple, aliased (`as`), grouped (`{}`), `use function`, `use const`.
> PHP stdlib classes (DateTime, PDO, Exception, etc.) are automatically classified as Stdlib without any configuration.

**Example `mille.toml` for a Laravel project:**

```toml
[project]
name    = "my-laravel-app"
root    = "."
languages = ["php"]

[[layers]]
name  = "domain"
paths = ["app/Domain/**"]

[[layers]]
name  = "application"
paths = ["app/Application/**"]
dependency_mode = "opt-in"
allow = ["domain"]

[[layers]]
name  = "infrastructure"
paths = ["app/Infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain", "application"]

[resolve.php]
composer_json = "composer.json"   # auto-detects "App\\" from autoload.psr-4
```

### C

> `#include "..."` is classified as Internal (project header). `#include <...>` is classified as Stdlib (standard/POSIX headers) or External (third-party libraries).

**Example `mille.toml` for a C project:**

```toml
[project]
name    = "my-c-app"
root    = "."
languages = ["c"]

[[layers]]
name  = "domain"
paths = ["src/domain/**"]

[[layers]]
name  = "usecase"
paths = ["src/usecase/**"]
dependency_mode = "opt-in"
allow = ["domain"]

[[layers]]
name  = "infrastructure"
paths = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow = ["domain"]
```

## How it Works

mille uses [tree-sitter](https://tree-sitter.github.io/) for AST-based import extraction — no regex heuristics.

```
mille.toml
    │
    ▼
Layer definitions
    │
Source files (*.rs, *.go, *.py, *.ts, *.js, *.java, ...)
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
