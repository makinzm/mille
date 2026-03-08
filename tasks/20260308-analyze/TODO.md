# PR 13: `mille analyze` コマンド

## 概要

依存グラフを可視化する `mille analyze` サブコマンドを追加する。
ルール適用なし（violations を出さない）で、レイヤー間の依存をグラフ表示する。

## タスク一覧

- [ ] `tasks/20260308-analyze/TODO.md` 作成（本ファイル）
- [ ] RED: テストを書く（`--no-verify` コミット）
- [ ] GREEN: 実装
  - [ ] `src/presentation/cli/args.rs` — `Analyze` サブコマンド + `AnalyzeFormat` 追加
  - [ ] `src/usecase/analyze.rs` — `analyze()` 関数 + `AnalyzeResult` 型
  - [ ] `src/usecase/mod.rs` — `analyze` モジュール公開
  - [ ] `src/presentation/formatter/svg.rs` — SVG グラフ生成
  - [ ] `src/presentation/formatter/mod.rs` — `svg` モジュール公開
  - [ ] `src/runner.rs` — `Command::Analyze` ディスパッチ追加
- [ ] REFACTOR: ドキュメント更新
  - [ ] `docs/TODO.md` — PR 13 チェックボックス更新
  - [ ] `README.md` — `mille analyze` の使用例追記

## 出力フォーマット

| フォーマット | 内容 |
|---|---|
| `terminal` | テキスト依存マトリクス（デフォルト） |
| `json` | JSON グラフ |
| `dot` | Graphviz DOT |
| `svg` | 自己完結 SVG（画像表示） |

## デザイン仕様（SVG）

- 背景: `#0F172A`、ノード: `#1E293B`、border: `#22C55E`
- テキスト: `#F8FAFC`、エッジ: `#22C55E`
- フォント: monospace
- レイアウト: トポロジカルソートで上→下

## E2E テストケース

1. `test_analyze_json_valid_shape`
2. `test_analyze_json_has_layer_names`
3. `test_analyze_json_has_edge`
4. `test_analyze_dot_starts_with_digraph`
5. `test_analyze_dot_has_node_and_edge`
6. `test_analyze_svg_is_valid_xml`
7. `test_analyze_svg_has_layer_text`
8. `test_analyze_svg_has_edge_line`
9. `test_analyze_terminal_shows_layers`
10. `test_analyze_exits_zero_always`
