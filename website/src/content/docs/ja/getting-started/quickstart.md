---
title: クイックスタート
description: mille init から mille check まで、最短で始めるガイド
---

import { Steps } from '@astrojs/starlight/components';

<Steps>

1. **`mille init` で設定ファイルを生成する**

   ```sh
   mille init
   ```

   プロジェクト内のimport文を解析してレイヤー構造と依存関係を推論し、`mille.toml` を生成します:

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

   生成後に内容を確認し、必要に応じて調整してください。

2. **`mille analyze` で依存グラフを可視化する**（オプション）

   ```sh
   mille analyze
   ```

   ルールを適用する前に実際の依存関係を確認できます。SVG 出力でブラウザ表示も可能:

   ```sh
   mille analyze --format svg > graph.svg && open graph.svg
   ```

3. **`mille check` で検査する**

   ```sh
   mille check
   ```

   違反がなければ exit code 0 で終了します。

4. **CI に組み込む**

   GitHub Actions の場合:

   ```yaml
   - run: mille check --format github-actions
   ```

   違反があると PR にアノテーションで表示されます。

</Steps>

## 次のステップ

- [設定リファレンス](/mille/ja/configuration/overview/) — `mille.toml` の全オプション
- [CI インテグレーション](/mille/ja/guides/ci-integration/) — GitHub Actions への組み込み方法
- [言語別ガイド](/mille/ja/guides/languages/rust/) — 言語ごとの設定例
