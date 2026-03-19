---
title: 設定の概要
description: mille.toml の構造と基本的な使い方
---

`mille.toml` をプロジェクトルートに配置することで mille の動作を制御します。

## ファイル構造

```toml
[project]
name      = "my-app"
root      = "."
languages = ["rust"]

[[layers]]
name            = "domain"
paths           = ["src/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-in"
external_allow  = []

# ... 追加レイヤー

[severity]
dependency_violation = "error"

[ignore]
paths = ["**/generated/**"]
```

## セクション一覧

| セクション | 説明 |
|---|---|
| [`[project]`](#project) | プロジェクト名・ルート・対象言語 |
| [`[[layers]]`](/mille/configuration/layers/) | レイヤー定義と依存ルール |
| [`[resolve.*]`](/mille/configuration/resolve/) | 言語別のインポート解決設定 |
| [`[severity]`](/mille/configuration/severity/) | 違反の重大度設定 |
| `[ignore]` | 解析から除外するパス |

## `[project]`

| キー | 説明 |
|---|---|
| `name` | プロジェクト名 |
| `root` | 解析のルートディレクトリ（`mille.toml` からの相対パス） |
| `languages` | 対象言語: `"rust"` / `"go"` / `"typescript"` / `"javascript"` / `"python"` / `"java"` / `"kotlin"` |

## `[ignore]`

解析対象から除外するファイルを指定します。

| キー | 説明 |
|---|---|
| `paths` | 完全に除外するグロブパターン（生成コード・vendor など） |
| `test_patterns` | レイヤー統計には含めるが違反チェックしないパターン（テストファイル） |

```toml
[ignore]
paths         = ["**/mock/**", "**/generated/**", "**/testdata/**"]
test_patterns = ["**/*_test.go", "**/*.spec.ts", "**/*.test.ts"]
```

**`paths` vs `test_patterns` の使い分け:**

- `paths`: 解析対象外にしたいファイル（生成コード、vendor ディレクトリ、モック）
- `test_patterns`: レイヤーをまたいで意図的にインポートするテストファイル（統合テストなど）
