---
title: mille init
description: プロジェクトの import を解析して mille.toml を自動生成する
---

import { Aside } from '@astrojs/starlight/components';

## 概要

```sh
mille init
```

プロジェクト内の import 文を解析してレイヤー構造と依存関係を推論し、`mille.toml` を自動生成します。

## オプション

| フラグ | デフォルト | 説明 |
|---|---|---|
| `--output <path>` | `mille.toml` | 出力先ファイルパス |
| `--force` | false | 既存ファイルを確認なしで上書き |
| `--depth <N>` | 自動 | レイヤー検出の深さ |

## 自動深さ検出

`mille init` は深さ 1〜6 を試し、`src` / `lib` / `app` などのソースレイアウトルートをスキップしながら、2〜8 個のレイヤー候補が得られる最初の深さを自動選択します。

例: `src/domain/entity`, `src/domain/repository`, `src/usecase/` というプロジェクトでは深さ 2 が選ばれ、`entity` と `repository` が `domain` にまとめられます。

`--depth N` で自動検出を上書きできます。

## 出力例

```
Detected languages: rust
Scanning imports...
Using layer depth: 2

Inferred layer structure:
  domain               ← (no internal dependencies)
  usecase              → domain
    external: anyhow
  infrastructure       → domain
    external: serde, tokio

Generated 'mille.toml'
```

## モノレポでの命名

複数のサブプロジェクトに同名ディレクトリ（例: `crawler/src/domain` と `server/src/domain`）がある場合、`mille init` は区別するプレフィックスをつけます（`crawler_domain`, `server_domain`）。マージは手動で行います。

## 言語別の自動検出

| 言語 | 自動検出する内容 |
|---|---|
| Go | `go.mod` を読み取り `[resolve.go] module_name` を自動生成 |
| Python | `src/` レイアウトを検出し `package_names` に `src` を自動追加 |
| Java/Kotlin | `pom.xml` / `build.gradle` + `settings.gradle` を読み取り `module_name` を設定 |

<Aside type="note">
`mille init` は常に exit code 0 で終了します。生成後に `mille check` を実行して結果を確認してください。
</Aside>
