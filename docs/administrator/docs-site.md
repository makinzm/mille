# ドキュメントサイト運用ガイド

## 概要

`website/` ディレクトリに [Starlight (Astro)](https://starlight.astro.build/) で構築されたドキュメントサイトがあります。

- **検索**: Pagefind（ビルド時に自動インデックス生成）
- **i18n**: 日本語（デフォルト）/ 英語
- **デプロイ先**: GitHub Pages (`https://makinzm.github.io/mille/`)

## ローカルで動作確認する

### 前提

[proto](https://moonrepo.dev/proto) がインストールされていること。

```sh
# proto のインストール（未インストールの場合）
curl -fsSL https://moonrepo.dev/install/proto.sh | bash
```

### 初回セットアップ

```sh
cd website

# proto で Bun をインストール（.prototools で 1.3.11 が固定）
proto install bun

# 依存関係のインストール（proto 経由で bun を実行）
proto run bun -- install
```

### 開発サーバー起動（ホットリロード付き）

```sh
cd website
proto run bun -- run dev
```

→ http://localhost:4321/mille/ でサイトを確認できます。

### 本番ビルドの確認

```sh
cd website
proto run bun -- run build    # dist/ に静的ファイルを生成（Pagefind インデックスも生成）
proto run bun -- run preview  # dist/ をローカルサーブ
```

→ http://localhost:4321/mille/ でビルド後のサイトを確認できます。

> **Note**: 検索（Pagefind）は `dev` では動作しません。`build && preview` で確認してください。

## コンテンツを編集する

### ページの場所

```
website/src/content/docs/
├── ja/          # 日本語（デフォルト言語）
│   ├── index.mdx
│   ├── getting-started/
│   ├── configuration/
│   ├── commands/
│   └── guides/
└── en/          # 英語
    └── (同構成)
```

### 新しいページを追加する

1. `website/src/content/docs/ja/<section>/<page>.md` を作成する
2. フロントマターを記述する:

```markdown
---
title: ページタイトル
description: 1行の説明
---
```

3. 対応する英語版 `website/src/content/docs/en/<section>/<page>.md` を作成する
4. `website/astro.config.mjs` の `sidebar` に追加する

### 利用できる Starlight コンポーネント

```mdx
import { Aside, Steps, Card, CardGrid, Tabs, TabItem } from '@astrojs/starlight/components';

<Aside type="tip">ヒント</Aside>
<Aside type="caution">注意</Aside>
<Aside type="note">メモ</Aside>

<Steps>
1. ステップ1
2. ステップ2
</Steps>
```

## CI/CD

`.github/workflows/docs.yml` が `website/**` への push 時に自動デプロイします。

### デプロイ先の GitHub Pages 設定（初回のみ）

1. GitHub リポジトリの **Settings > Pages** を開く
2. Source を **GitHub Actions** に設定する

### ワークフローの手動トリガー

GitHub Actions の **Actions** タブ → **Deploy docs to GitHub Pages** → **Run workflow** で手動実行できます。

## バージョン管理

ドキュメントのバージョンは管理していません（常に最新の `main` ブランチの内容を反映）。
