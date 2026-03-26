# Timeline: mille add --depth 実装

## 2026-03-26

### コードベース調査
- 既存 `mille add`: 単一ディレクトリ → 1レイヤー追加
- `scan_project`: depth ベースで source dirs を収集 → ancestor_at_depth でロールアップ → infer_layers
- `auto_detect_layer_depth`: depth 1-6 を試し 2-8 候補が出る最初の depth を使用
- 目標: `mille add` に `--depth` を追加し、`scan_project` と `infer_layers` を再利用
