---
title: CI Integration
description: How to integrate mille into GitHub Actions
---

import { Aside } from '@astrojs/starlight/components';

## GitHub Actions Setup

```yaml
# .github/workflows/architecture-check.yml
name: Architecture Check

on: [push, pull_request]

jobs:
  mille:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install mille (cargo)
        run: cargo install mille

      - name: Check architecture
        run: mille check --format github-actions
```

## `--format github-actions` Output

Violations appear as annotations directly in PR code reviews:

```
::error file=src/usecase/order.rs,line=3::External violation: 'sqlx' is not allowed in 'usecase' (import: sqlx)
::error file=src/main.rs,line=15::Call pattern violation: 'find_user' is not in allow_methods
```

## Output Formats

| Format | Use Case |
|---|---|
| `terminal` (default) | Local development — readable output |
| `github-actions` | CI — PR review annotations |
| `json` | Tool integration — machine-readable |

## Gradual CI Adoption

<Aside type="tip">
For brownfield projects, use `--fail-on` and `[severity]` together for gradual enforcement.
</Aside>

**Step 1**: Start with warnings only

```toml
# mille.toml
[severity]
dependency_violation = "warning"
```

```yaml
- run: mille check --format github-actions
  # exits 0 — warnings shown but CI doesn't fail
```

**Step 2**: Enforce once violations are fixed

```toml
[severity]
dependency_violation = "error"
```

## npm Install Example

```yaml
- name: Install mille (npm)
  run: npm install -g @makinzm/mille

- name: Check architecture
  run: mille check --format github-actions
```

## Exit Codes

| Code | Meaning |
|---|---|
| `0` | No violations |
| `1` | Violations found |
| `3` | Configuration error |
