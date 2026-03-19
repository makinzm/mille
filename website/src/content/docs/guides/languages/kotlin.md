---
title: Kotlin
description: Kotlin プロジェクトでの mille 設定例
---

## 設定例

```toml
[project]
name      = "my-kotlin-app"
root      = "."
languages = ["kotlin"]

[resolve.java]
module_name = "com.example.myapp"

[[layers]]
name            = "domain"
paths           = ["src/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-out"

[[layers]]
name            = "usecase"
paths           = ["src/usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-out"

[[layers]]
name            = "infrastructure"
paths           = ["src/infrastructure/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-in"
external_allow  = ["kotlinx.coroutines"]
```

## Java との違い

Kotlin は `languages = ["kotlin"]` を指定しますが、インポート解決の設定は `[resolve.java]` セクションを共用します。

## `mille init` の動作

`build.gradle.kts` または `build.gradle` + `settings.gradle` を読み取り `module_name` を自動設定します。

## インポートの分類

Java と同じロジックです:

| インポート | 分類 |
|---|---|
| `import com.example.myapp.domain.User` | 内部（`module_name` で始まる） |
| `import kotlinx.coroutines.launch` | 外部 |
| `import java.util.UUID` | 外部（標準ライブラリ） |

## Gradle レイアウトへの対応

Maven の `src/main/java/` のような深いレイアウトにも `**/layer/**` グロブで対応します:

```toml
[[layers]]
name  = "domain"
paths = ["**/domain/**"]   # src/main/kotlin/com/example/domain/ にも一致
```
