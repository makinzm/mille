---
title: Java
description: Java プロジェクトでの mille 設定例（Maven / Gradle）
---

## 設定例

```toml
[project]
name      = "my-java-app"
root      = "."
languages = ["java"]

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
external_allow  = ["java.util.List", "java.util.Map"]
```

## `mille init` の動作

`pom.xml`（Maven）または `build.gradle` + `settings.gradle`（Gradle）を読み取り `module_name` を自動設定します。レイヤーパスは `**/layer/**` グロブを使用するため、Maven の `src/main/java/com/example/myapp/domain/` のような深いレイアウトにも対応します。

### `mille init` 出力例

```
Detected languages: java
Scanning imports...

Inferred layer structure:
  domain               ← (no internal dependencies)
  infrastructure       → domain
    external: java.util.List
  usecase              → domain
  main                 → domain, infrastructure, usecase

Generated 'mille.toml'
```

## インポートの分類

| インポート | 分類 |
|---|---|
| `import com.example.myapp.domain.User` | 内部（`module_name` で始まる） |
| `import static com.example.myapp.util.Helper.method` | 内部 |
| `import java.util.List` | 外部（標準ライブラリ） |
| `import org.springframework.*` | 外部 |

ワイルドカードインポート（`import java.util.*`）は現時点では未対応です。

## Maven / Gradle の自動設定

| ビルドツール | 自動設定 |
|---|---|
| Maven | `pom.xml` の `groupId.artifactId` を `module_name` に使用 |
| Gradle | `build.gradle` の `group` + `settings.gradle` の `rootProject.name` を使用 |
