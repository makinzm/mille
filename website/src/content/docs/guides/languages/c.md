---
title: C
description: C プロジェクトでの mille 設定例
---

## 設定例

```toml
[project]
name      = "my-c-app"
root      = "."
languages = ["c"]

[[layers]]
name            = "domain"
paths           = ["src/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "usecase"
paths           = ["src/usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "infrastructure"
paths           = ["src/infrastructure/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []
```

## インクルードの分類

| インクルード | 分類 |
|---|---|
| `#include "domain/user.h"` | 内部（ローカルヘッダ） |
| `#include "create_user.h"` | 内部（同ディレクトリ） |
| `#include <stdio.h>` | 標準ライブラリ |
| `#include <stdlib.h>` | 標準ライブラリ |
| `#include <curl/curl.h>` | 外部（サードパーティ） |

## 分類ルール

- `#include "..."` （引用符）→ **Internal**（プロジェクト内ヘッダ）
- `#include <...>` （山括弧）→ C 標準 / POSIX ヘッダなら **Stdlib**、それ以外は **External**

相対パス（`../domain/user.h`）は正規化されてレイヤーマッチングに使用されます。

## 対応ファイル拡張子

`.c` と `.h` の両方が解析対象です。

## ネーミング規則

Symbol（関数定義、struct/enum/union/typedef）、Variable（グローバル変数）、Comment のすべてが `name_deny` の検出対象です。
