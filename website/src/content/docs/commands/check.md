---
title: mille check
description: アーキテクチャの依存ルールを検査する
---

## 概要

```sh
mille check
```

`mille.toml` に定義されたルールに基づき、レイヤー依存・外部ライブラリ依存・メソッド呼び出しパターンを検査します。

## 出力フォーマット

```sh
mille check                          # ターミナル出力（デフォルト）
mille check --format github-actions  # GitHub Actions アノテーション
mille check --format json            # JSON 出力
```

### ターミナル出力例

```
VIOLATION [error] dependency_violation
  src/usecase/user.rs:12
  usecase → infrastructure (denied)
  import: crate::infrastructure::db::UserRepository

1 violation(s) found
```

### GitHub Actions アノテーション

```
::error file=src/usecase/user.rs,line=12::dependency_violation: usecase → infrastructure (denied)
```

### JSON 出力

```json
{
  "violations": [
    {
      "type": "dependency_violation",
      "severity": "error",
      "file": "src/usecase/user.rs",
      "line": 12,
      "message": "usecase → infrastructure (denied)"
    }
  ]
}
```

## 失敗閾値

```sh
mille check                         # error 時のみ exit 1（デフォルト）
mille check --fail-on warning       # warning でも exit 1
mille check --fail-on error         # デフォルトと同じ
```

## 終了コード

| コード | 意味 |
|---|---|
| `0` | 違反なし |
| `1` | 違反あり |
| `3` | 設定ファイルエラー |

## 自動除外パス

`mille check` は以下のディレクトリを自動的にスキップします:
`.venv`, `venv`, `node_modules`, `target`, `dist`, `build` など
