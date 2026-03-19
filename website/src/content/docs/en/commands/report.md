---
title: mille report external
description: List external dependency packages by layer
---

## Overview

```sh
mille report external
```

Lists the external packages actually imported by each layer. Useful for auditing `external_allow` lists or documenting your dependency footprint.

`mille report external` always exits with code 0.

## Output Formats

```sh
mille report external                  # terminal output (default)
mille report external --format json    # JSON output
mille report external --output report.json --format json   # write to file
```

### Terminal Output Example

```
External Dependencies by Layer

  domain          (none)
  usecase         (none)
  infrastructure  database/sql
  cmd             fmt, os
```

### JSON Output Example

```json
{
  "layers": {
    "domain": [],
    "usecase": [],
    "infrastructure": ["database/sql"],
    "cmd": ["fmt", "os"]
  }
}
```

## Use Cases

- Verify no packages are missing from `external_allow`
- Audit for unintended external dependencies
- Document your dependency footprint for compliance or review
