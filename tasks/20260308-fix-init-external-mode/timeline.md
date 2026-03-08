# Timeline

## 2026-03-08

### 調査
- `LayerConfig.external_mode` は必須フィールド（`#[serde(default)]` なし）
- `LayerSuggestion` に `external_mode` フィールドがない
- `generate_toml()` は `external_mode` を出力していない
- `runner.rs` のフィールドアクセス（`.name`, `.allow`, `.external_allow`, `.paths`）は `LayerConfig` と一致するため runner 側の変更不要

### 設計決定
- `LayerSuggestion` を廃止して `LayerConfig` を直接使う
- `infer_layers()` → `Vec<LayerConfig>`、`generate_toml()` → `&[LayerConfig]`
- `dependency_mode`・`external_mode` のフォーマット用ヘルパー `fn mode_str()` を追加

### RED フェーズ
