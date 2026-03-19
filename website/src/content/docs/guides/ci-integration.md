---
title: CI インテグレーション
description: GitHub Actions に mille を組み込む方法
---

import { Aside } from '@astrojs/starlight/components';

## GitHub Actions セットアップ

```yaml
# .github/workflows/architecture-check.yml
name: Architecture Check

on: [push, pull_request]

jobs:
  mille:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install mille (cargo)
        run: cargo install mille

      - name: Check architecture
        run: mille check --format github-actions
```

## `--format github-actions` の出力

違反があると PR のコードレビュー画面に直接アノテーションが表示されます:

```
::error file=src/usecase/order.rs,line=3::External violation: 'sqlx' is not allowed in 'usecase' (import: sqlx)
::error file=src/main.rs,line=15::Call pattern violation: 'find_user' is not in allow_methods
```

## 出力フォーマット一覧

| フォーマット | 用途 |
|---|---|
| `terminal`（デフォルト） | ローカル開発。読みやすいターミナル出力 |
| `github-actions` | CI。PR レビュー画面にアノテーション表示 |
| `json` | 外部ツール連携。機械可読な JSON 出力 |

## 段階的な CI 導入

<Aside type="tip">
既存プロジェクトへの段階的な導入には `--fail-on` と `[severity]` の組み合わせが有効です。
</Aside>

**ステップ 1**: まず warning として警告のみ出す

```toml
# mille.toml
[severity]
dependency_violation = "warning"
```

```yaml
- run: mille check --format github-actions
  # exit 0 — CI は落とさず警告のみ表示
```

**ステップ 2**: 修正が進んだら error に変更して強制

```toml
[severity]
dependency_violation = "error"
```

## npm でのインストール例

```yaml
- name: Install mille (npm)
  run: npm install -g @makinzm/mille

- name: Check architecture
  run: mille check --format github-actions
```

## 終了コード

| コード | 意味 |
|---|---|
| `0` | 違反なし |
| `1` | 違反あり |
| `3` | 設定ファイルエラー |

GitHub Actions は exit code 1 を検知して CI を失敗させます。
