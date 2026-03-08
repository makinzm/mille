# fix: mille init が external_mode を生成しない

## 問題

`LayerConfig.external_mode` は必須フィールド（`#[serde(default)]` なし）だが、
`mille init` が生成する TOML に `external_mode` が含まれていない。
ユーザーがそのまま `mille check` を実行するとパースエラーになる。

## 方針：LayerSuggestion を廃止して LayerConfig を直接使う

### 型から保証する仕組み

- `LayerSuggestion`（中間型）を廃止
- `infer_layers()` の戻り型を `Vec<LayerConfig>` に変更
- `generate_toml()` の引数を `&[LayerConfig]` に変更
- `LayerConfig` に新しい必須フィールドが追加されたとき `infer_layers()` がコンパイルエラーになる

## チェックリスト

- [x] TODO.md 作成
- [ ] RED: テスト追加（`test_generate_toml_includes_external_mode`）＋型変更 (--no-verify)
- [ ] GREEN: `generate_toml` で `external_mode` を出力する実装
- [ ] E2E テストに `external_mode` アサート追加
- [ ] lefthook 通過確認
- [ ] docs/TODO.md 更新
- [ ] README.md 確認（変更なし）
