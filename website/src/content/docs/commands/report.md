---
title: mille report external
description: レイヤー別の外部依存パッケージを一覧表示する
---

## 概要

```sh
mille report external
```

各レイヤーが実際にインポートしている外部パッケージを一覧表示します。`external_allow` リストの監査やプロジェクトの依存フットプリント把握に役立ちます。

`mille report external` は常に exit code 0 で終了します。

## 出力フォーマット

```sh
mille report external                  # ターミナル出力（デフォルト）
mille report external --format json    # JSON 出力
mille report external --output report.json --format json   # ファイルへ書き出し
```

### ターミナル出力例

```
External Dependencies by Layer

  domain          (none)
  usecase         (none)
  infrastructure  database/sql
  cmd             fmt, os
```

### JSON 出力例

```json
{
  "layers": {
    "domain": [],
    "usecase": [],
    "infrastructure": ["database/sql"],
    "cmd": ["fmt", "os"]
  }
}
```

## 活用シーン

- `external_allow` に不足しているパッケージがないか確認する
- 意図しない外部依存が混入していないか監査する
- ドキュメント用に依存フットプリントを記録する
