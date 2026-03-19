---
title: mille analyze
description: Visualize the dependency graph (no rules enforced)
---

## Overview

```sh
mille analyze
```

Visualizes actual dependencies as a graph without enforcing rules. Ideal for understanding the current state of your architecture before running `mille check`.

`mille analyze` always exits with code 0.

## Output Formats

```sh
mille analyze                  # terminal output (default)
mille analyze --format json    # JSON graph
mille analyze --format dot     # Graphviz DOT
mille analyze --format svg     # self-contained SVG
```

### SVG Output

```sh
mille analyze --format svg > graph.svg && open graph.svg
```

Generates an SVG file you can open in a browser (dark theme, green edges).

### DOT Output (Graphviz)

```sh
mille analyze --format dot | dot -Tsvg -o graph.svg
```

### JSON Output Example

```json
{
  "layers": ["domain", "usecase", "infrastructure"],
  "edges": [
    { "from": "usecase", "to": "domain" },
    { "from": "infrastructure", "to": "domain" }
  ]
}
```
