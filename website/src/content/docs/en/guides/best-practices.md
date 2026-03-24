---
title: Best Practices
description: How to use mille alongside other tools
---

## When mille's YAML checks are useful

mille's YAML support focuses on **naming convention checks** via `name_deny`. It shines in these use cases:

- **Cross-layer naming consistency**: Ensure cloud-provider-specific keywords (`aws`, `gcp`) don't leak into `base/` layers
- **Environment separation**: Verify that `production` or `staging` don't appear in `model-config/`
- **Unified rules for code and config**: Check Rust/Python source code and YAML config files with the same `mille.toml`

## Tool comparison

| Aspect | mille | kube-linter | conftest | Kyverno CLI |
|---|---|---|---|---|
| **Purpose** | Layer boundary naming rules | K8s best practices | General-purpose policy (Rego) | K8s policy (CEL/Rego) |
| **Scope** | Source code + YAML | K8s manifests | Any structured data | K8s manifests |
| **Rule format** | TOML (declarative) | Built-in checks | Rego | YAML (ClusterPolicy) |
| **Learning curve** | Low | Low | Medium–High | Medium |
| **CI integration** | `mille check` | `kube-linter lint` | `conftest test` | `kyverno apply` |
| **Source code support** | 9 languages + YAML | None | None | None |

## Combining tools

### Recommended setup

```
mille          → Naming boundaries (across source code + YAML)
kube-linter    → K8s best practices (resource limits, security)
conftest       → General-purpose policy (custom organizational rules)
Kyverno CLI    → Admission dry-run (final validation before production)
```

### CI pipeline example

```yaml
jobs:
  lint:
    steps:
      # 1. Architecture + naming checks
      - run: mille check

      # 2. K8s best practices
      - run: kube-linter lint manifests/

      # 3. Custom policies
      - run: conftest test manifests/ -p policy/
```

### Decision guide

- **"This layer must not contain a specific keyword"** → mille
- **"Do K8s resources have resource limits?"** → kube-linter
- **"Do resources follow our org's labeling conventions?"** → conftest
- **"Validate against production cluster policies before deploy"** → Kyverno CLI
