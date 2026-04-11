---
title: Elixir
description: Elixir プロジェクトでの mille 設定例
---

## 設定例

```toml
[project]
name      = "my-elixir-app"
root      = "."
languages = ["elixir"]

[resolve.elixir]
app_name = "MyApp"

[[layers]]
name            = "domain"
paths           = ["lib/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "usecase"
paths           = ["lib/usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "infrastructure"
paths           = ["lib/infrastructure/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []
```

## `app_name` の設定

`[resolve.elixir] app_name` には `mix.exs` の `:app` に対応する PascalCase のモジュール名（例: `MyApp`）を指定します。

```toml
[resolve.elixir]
app_name = "MyApp"   # mix.exs の :app が :my_app の場合
```

## インポートの分類

Elixir の 4 種ディレクティブはすべて依存関係として解析されます。

| ディレクティブ | 例 | 分類 |
|---|---|---|
| `alias` | `alias MyApp.Domain.User` | 内部（`app_name` に一致） |
| `alias` with `as:` | `alias MyApp.Domain.User, as: U` | 内部 |
| `import` | `import Enum` | 外部 |
| `require` | `require Logger` | 外部 |
| `use` | `use Ecto.Schema` | 外部 |

## モジュールとファイルパスの対応

内部モジュールのパスは以下のルールで解決されます:

`MyApp.Domain.User` → `app_name` を除去 → `Domain.User` → lowercase → `lib/domain/user.ex`
