---
title: mille init
description: Auto-generate mille.toml by analyzing your project's imports
---

import { Aside } from '@astrojs/starlight/components';

## Overview

```sh
mille init
```

Analyzes import statements in your source files to infer layer structure and dependencies, then generates `mille.toml`.

## Options

| Flag | Default | Description |
|---|---|---|
| `--output <path>` | `mille.toml` | Write config to a custom path |
| `--force` | false | Overwrite an existing file without prompting |
| `--depth <N>` | auto | Layer detection depth from project root |

## Auto-depth Detection

`mille init` tries depths 1–6, skips common source-layout roots (`src`, `lib`, `app`, etc.), and selects the first depth that yields 2–8 candidate layers.

For a project with `src/domain/entity`, `src/domain/repository`, `src/usecase/` — depth 2 is chosen, rolling `entity` and `repository` up into `domain`.

Use `--depth N` to override.

## Example Output

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

## Monorepo Naming

When multiple sub-projects contain the same directory name (e.g. `crawler/src/domain` and `server/src/domain`), `mille init` adds a distinguishing prefix (`crawler_domain`, `server_domain`). Merging is left to you.

## Language-specific Auto-detection

| Language | Auto-detected |
|---|---|
| Go | Reads `go.mod` and generates `[resolve.go] module_name` |
| Python | Detects `src/` layout and adds `src` to `package_names` |
| Java/Kotlin | Reads `pom.xml` / `build.gradle` + `settings.gradle` for `module_name` |

<Aside type="note">
`mille init` always exits with code 0. Run `mille check` afterward to see results.
</Aside>
