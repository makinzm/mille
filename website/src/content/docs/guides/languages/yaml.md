---
title: YAML
description: YAML ファイルでの mille 設定例
---

YAML は **naming-only 言語** です。import の概念がないため、`dependency_mode` / `external_mode` / `allow_call_patterns` は使用しません。`name_deny` によるネーミング規則チェックのみをサポートします。

## 設定例

```toml
[project]
name      = "my-k8s-project"
root      = "."
languages = ["yaml"]

[[layers]]
name            = "config"
paths           = ["config/**"]
dependency_mode = "opt-out"
external_mode   = "opt-out"
name_deny       = ["aws", "gcp"]

[[layers]]
name            = "manifests"
paths           = ["manifests/**"]
dependency_mode = "opt-out"
external_mode   = "opt-out"
```

## 名前の分類

| YAML 要素 | NameKind | 例 |
|---|---|---|
| マッピングキー | `Symbol` | `aws_region:` の `aws_region` |
| スカラー値（プレーン） | `StringLiteral` | `region: us-east-1` の `us-east-1` |
| スカラー値（引用符付き） | `StringLiteral` | `image: "my-app:latest"` の `my-app:latest` |
| コメント | `Comment` | `# Deploy to AWS` |

## ユースケース

### Kubernetes マニフェストの命名規則

特定のレイヤーにクラウドプロバイダ固有のキーワードが現れないようにする：

```toml
[[layers]]
name      = "base"
paths     = ["base/**"]
name_deny = ["aws", "gcp", "azure"]
```

### MLOps 設定ファイルのチェック

環境固有のキーワードを分離する：

```toml
[[layers]]
name      = "model-config"
paths     = ["configs/models/**"]
name_deny = ["staging", "production"]
name_targets = ["string_literal"]   # 値のみチェック
```

## 対応ファイル拡張子

`.yaml` と `.yml` の両方が解析対象です。

## 制限事項

- YAML は import の概念がないため、`dependency_mode` / `external_mode` は常に `opt-out` に設定してください
- `allow_call_patterns` は使用できません
- `name_deny` のみが有効なチェックです
