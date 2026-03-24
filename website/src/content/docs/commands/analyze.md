---
title: mille analyze
description: 依存グラフを可視化する（ルール適用なし）
---

## 概要

```sh
mille analyze
mille analyze ./path/to/project    # 対象ディレクトリを指定
```

実際の依存関係をグラフとして可視化します。ルールを適用しないため、`mille check` の前にアーキテクチャの現状を把握するのに最適です。

位置引数でプロジェクトディレクトリを指定できます。省略時はカレントディレクトリ（`.`）が対象です。

`mille analyze` は常に exit code 0 で終了します。

## 出力フォーマット

```sh
mille analyze                  # ターミナル出力（デフォルト）
mille analyze --format json    # JSON グラフ
mille analyze --format dot     # Graphviz DOT
mille analyze --format svg     # 自己完結型 SVG
```

### SVG 出力

```sh
mille analyze --format svg > graph.svg && open graph.svg
```

ブラウザで開ける SVG ファイルを生成します（ダークテーマ・グリーンエッジ）。

### DOT 出力（Graphviz）

```sh
mille analyze --format dot | dot -Tsvg -o graph.svg
```

### JSON 出力例

```json
{
  "layers": ["domain", "usecase", "infrastructure"],
  "edges": [
    { "from": "usecase", "to": "domain" },
    { "from": "infrastructure", "to": "domain" }
  ]
}
```
