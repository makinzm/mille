---
title: Python
description: Python プロジェクトでの mille 設定例
---

## 設定例

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

## `src/` レイアウト（namespace packages）

`src/` レイアウトを使い `from src.domain.entity import Foo` のようにインポートする場合、`mille init` は自動的に `package_names` に `src` を追加します。

```toml
[resolve.python]
src_root      = "."
package_names = ["src"]   # ← mille init が自動設定
```

## インポートの分類

| インポート | 分類 |
|---|---|
| `from .sibling import X`（相対インポート） | 内部 |
| `import domain.entity`（`package_names` に一致） | 内部 |
| `from src.domain.entity import Foo` | 内部（`src` が `package_names` にある場合） |
| `import os`, `import sqlalchemy` | 外部 |

## サブモジュールインポート

`external_allow = ["matplotlib"]` と設定すると `import matplotlib` も `import matplotlib.pyplot` も許可されます。
