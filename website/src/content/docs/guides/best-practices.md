---
title: ベストプラクティス
description: mille と他ツールの使い分け・併用パターン
---

## mille の YAML チェックが有効な場面

mille の YAML サポートは `name_deny` による **命名規則チェック** に特化しています。以下のようなユースケースで威力を発揮します：

- **レイヤー横断の命名一貫性**: `base/` レイヤーにクラウドプロバイダ固有キーワード（`aws`, `gcp`）が混入していないか
- **環境分離の検証**: `model-config/` に `production` や `staging` が紛れ込んでいないか
- **ソースコードと設定ファイルの統一ルール**: Rust/Python のソースコードと YAML 設定を同じ `mille.toml` でまとめてチェック

## ツール比較

| 観点 | mille | kube-linter | conftest | Kyverno CLI |
|---|---|---|---|---|
| **目的** | レイヤー境界の命名規則 | K8s ベストプラクティス | 汎用ポリシー (Rego) | K8s ポリシー (CEL/Rego) |
| **対象** | ソースコード + YAML | K8s マニフェスト | 任意の構造化データ | K8s マニフェスト |
| **ルール記述** | TOML (宣言的) | 組み込みチェック | Rego | YAML (ClusterPolicy) |
| **学習コスト** | 低 | 低 | 中〜高 | 中 |
| **CI 統合** | `mille check` | `kube-linter lint` | `conftest test` | `kyverno apply` |
| **ソースコード対応** | Rust/Go/TS/Python 等 9言語 + YAML | なし | なし | なし |

## 併用パターン

### 推奨構成

```
mille          → 命名境界（ソースコード + YAML を横断）
kube-linter    → K8s ベストプラクティス（リソース制限、セキュリティ）
conftest       → 汎用ポリシー（組織固有のカスタムルール）
Kyverno CLI    → Admission dry-run（本番適用前の最終検証）
```

### CI パイプライン例

```yaml
jobs:
  lint:
    steps:
      # 1. アーキテクチャ + 命名チェック
      - run: mille check

      # 2. K8s ベストプラクティス
      - run: kube-linter lint manifests/

      # 3. カスタムポリシー
      - run: conftest test manifests/ -p policy/
```

### 使い分けの判断基準

- **「このレイヤーに特定のキーワードが現れてはいけない」** → mille
- **「K8s リソースにリソース制限が設定されているか」** → kube-linter
- **「組織固有のラベル規約に従っているか」** → conftest
- **「本番クラスタのポリシーを事前検証したい」** → Kyverno CLI
