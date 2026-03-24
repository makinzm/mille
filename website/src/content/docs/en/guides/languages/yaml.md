---
title: YAML
description: mille configuration for YAML files
---

YAML is a **naming-only language**. Since YAML has no concept of imports, `dependency_mode` / `external_mode` / `allow_call_patterns` are not used. Only `name_deny` naming convention checks are supported.

## Configuration example

```toml
[project]
name      = "my-k8s-project"
root      = "."
languages = ["yaml"]

[[layers]]
name            = "config"
paths           = ["config/**"]
dependency_mode = "opt-out"
external_mode   = "opt-out"
name_deny       = ["aws", "gcp"]

[[layers]]
name            = "manifests"
paths           = ["manifests/**"]
dependency_mode = "opt-out"
external_mode   = "opt-out"
```

## Name classification

| YAML element | NameKind | Example |
|---|---|---|
| Mapping key | `Symbol` | `aws_region` in `aws_region:` |
| Plain scalar value | `StringLiteral` | `us-east-1` in `region: us-east-1` |
| Quoted scalar value | `StringLiteral` | `my-app:latest` in `image: "my-app:latest"` |
| Comment | `Comment` | `# Deploy to AWS` |

## Use cases

### Kubernetes manifest naming rules

Prevent cloud-provider-specific keywords from appearing in certain layers:

```toml
[[layers]]
name      = "base"
paths     = ["base/**"]
name_deny = ["aws", "gcp", "azure"]
```

### MLOps config file checks

Separate environment-specific keywords:

```toml
[[layers]]
name      = "model-config"
paths     = ["configs/models/**"]
name_deny = ["staging", "production"]
name_targets = ["string_literal"]   # check values only
```

## Supported file extensions

Both `.yaml` and `.yml` files are analyzed.

## Limitations

- YAML has no imports, so `dependency_mode` / `external_mode` should always be set to `opt-out`
- `allow_call_patterns` is not applicable
- Only `name_deny` provides meaningful checks
