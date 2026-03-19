---
title: Configuration Overview
description: Structure and basic usage of mille.toml
---

Place `mille.toml` in your project root to configure mille.

## File Structure

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

# ... additional layers

[severity]
dependency_violation = "error"

[ignore]
paths = ["**/generated/**"]
```

## Sections

| Section | Description |
|---|---|
| [`[project]`](#project) | Project name, root, and target languages |
| [`[[layers]]`](/mille/en/configuration/layers/) | Layer definitions and dependency rules |
| [`[resolve.*]`](/mille/en/configuration/resolve/) | Language-specific import resolution |
| [`[severity]`](/mille/en/configuration/severity/) | Violation severity settings |
| `[ignore]` | Paths to exclude from analysis |

## `[project]`

| Key | Description |
|---|---|
| `name` | Project name |
| `root` | Root directory for analysis (relative to `mille.toml`) |
| `languages` | Languages to check: `"rust"` / `"go"` / `"typescript"` / `"javascript"` / `"python"` / `"java"` / `"kotlin"` |

## `[ignore]`

| Key | Description |
|---|---|
| `paths` | Glob patterns to fully exclude (generated code, vendor dirs, mocks) |
| `test_patterns` | Patterns for files counted in layer stats but not violation-checked (test files) |

```toml
[ignore]
paths         = ["**/mock/**", "**/generated/**", "**/testdata/**"]
test_patterns = ["**/*_test.go", "**/*.spec.ts", "**/*.test.ts"]
```

**`paths` vs `test_patterns`:**

- `paths`: Files to exclude entirely from analysis
- `test_patterns`: Test files that intentionally import across layers (e.g., integration tests)
