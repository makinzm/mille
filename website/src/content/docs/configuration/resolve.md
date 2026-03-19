---
title: インポート解決設定
description: 言語別の [resolve.*] 設定リファレンス
---

import { Tabs, TabItem } from '@astrojs/starlight/components';

mille は言語ごとにインポートを「内部（Internal）」か「外部（External）」かに分類します。
`[resolve.*]` セクションでこの解決ロジックを設定します。

## `[resolve.typescript]`

| キー | 説明 |
|---|---|
| `tsconfig` | `tsconfig.json` へのパス。`compilerOptions.paths` のエイリアスを内部インポートとして解決します。 |

```toml
[resolve.typescript]
tsconfig = "./tsconfig.json"
```

**TypeScript / JavaScript のインポート分類:**

| インポート | 分類 |
|---|---|
| `import X from "./module"` | 内部 |
| `import X from "../module"` | 内部 |
| `import X from "@/module"` （tsconfig のパスエイリアス） | 内部 |
| `import X from "react"` | 外部 |
| `import fs from "node:fs"` | 外部 |

## `[resolve.go]`

| キー | 説明 |
|---|---|
| `module_name` | Go モジュール名（`go.mod` と一致）。`mille init` が `go.mod` から自動生成します。 |

```toml
[resolve.go]
module_name = "github.com/myorg/my-go-app"
```

## `[resolve.python]`

| キー | 説明 |
|---|---|
| `src_root` | Python ソースツリーのルートディレクトリ（`mille.toml` からの相対パス） |
| `package_names` | 内部パッケージ名のリスト。これらで始まるインポートが内部として分類されます。 |

```toml
[resolve.python]
src_root      = "."
package_names = ["domain", "usecase", "infrastructure"]
```

**Python のインポート分類:**

| インポート | 分類 |
|---|---|
| `from .sibling import X`（相対インポート） | 内部 |
| `import domain.entity`（`package_names` に一致） | 内部 |
| `import os`, `import sqlalchemy` | 外部 |

## `[resolve.java]`

Java と Kotlin で共通して使用します。

| キー | 説明 |
|---|---|
| `module_name` | プロジェクトのベースパッケージ（例: `com.example.myapp`）。このプレフィックスで始まるインポートが内部として分類されます。 |
| `pom_xml` | `pom.xml` へのパス。`module_name` 未設定時に `groupId.artifactId` を使用。 |
| `build_gradle` | `build.gradle` へのパス。`module_name` 未設定時に `group` + `rootProject.name` を使用。 |

```toml
[resolve.java]
module_name = "com.example.myapp"
```

**Java のインポート分類:**

| インポート | 分類 |
|---|---|
| `import com.example.myapp.domain.User` | 内部 |
| `import static com.example.myapp.util.Helper.method` | 内部 |
| `import java.util.List`, `import org.springframework.*` | 外部 |
