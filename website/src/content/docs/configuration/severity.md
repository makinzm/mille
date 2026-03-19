---
title: 重大度設定
description: 違反の severity を error / warning / info で制御する
---

import { Aside } from '@astrojs/starlight/components';

## `[severity]`

違反の重大度を設定します。`"error"` / `"warning"` / `"info"` の3段階で制御できます。

| キー | デフォルト | 説明 |
|---|---|---|
| `dependency_violation` | `"error"` | レイヤー依存ルール違反 |
| `external_violation` | `"error"` | 外部ライブラリルール違反 |
| `call_pattern_violation` | `"error"` | DI エントリポイントのメソッド呼び出しルール違反 |
| `unknown_import` | `"warning"` | 分類できなかったインポート |

```toml
[severity]
dependency_violation   = "warning"   # 段階的導入時は warning に落とす
external_violation     = "error"
call_pattern_violation = "error"
unknown_import         = "warning"
```

## `--fail-on` フラグとの組み合わせ

```sh
mille check                      # error のみで exit 1（デフォルト）
mille check --fail-on warning    # warning でも exit 1
mille check --fail-on error      # デフォルトと同じ
```

<Aside type="tip">
**段階的導入の推奨手順**

1. まず `dependency_violation = "warning"` に設定して `mille check` を CI に組み込む
2. 違反を確認しながら少しずつ修正する
3. 修正完了後に `"error"` に戻す
</Aside>

## 終了コード

| コード | 意味 |
|---|---|
| `0` | 違反なし（または `--fail-on warning` なしの warning のみ） |
| `1` | 設定した閾値以上の違反が1件以上ある |
| `3` | 設定ファイルエラー |
