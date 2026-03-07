# GitHub Actions での mille 活用ガイド

## セットアップ

```yaml
# .github/workflows/mille.yml
name: Architecture Check

on: [push, pull_request]

jobs:
  mille:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install mille
        run: |
          curl -sSfL https://github.com/YOUR_ORG/mille/releases/latest/download/mille-x86_64-unknown-linux-gnu.tar.gz \
            | tar xz -C /usr/local/bin

      - name: Check architecture
        run: mille check --format github-actions
```

## `--format github-actions` の出力形式

違反があると PR のコードレビュー画面に直接アノテーションが表示されます。

```
::error file=src/usecase/order.rs,line=3::External violation: 'sqlx' is not allowed in 'usecase' (import: sqlx)
::error file=src/main.rs,line=15::Call pattern violation: 'find_user' is not in allow_methods (call: repo.find_user)
```

### フォーマット仕様

| フォーマット | 用途 |
|---|---|
| `terminal`（デフォルト） | ローカル開発。❌/✅ マーカー付きの読みやすい出力 |
| `github-actions` | CI。PR レビュー画面にアノテーションを表示 |
| `json` | 外部ツールとの連携。機械可読な JSON 出力 |

## ローカルでの検証

```sh
# デフォルト（terminal）
mille check

# GitHub Actions と同じ出力を手元で確認
mille check --format github-actions

# JSON 出力（jq で整形）
mille check --format json | jq .
```

## exit code

| コード | 意味 |
|---|---|
| `0` | 違反なし |
| `1` | error 違反あり |
| `3` | 設定ファイルエラー |

GitHub Actions は exit code 1 を検知して CI を失敗させるため、`--format github-actions` 指定時もアノテーション出力と同時に exit 1 されます。
