# Kotlin サポート実装タイムライン

## 2026-03-15

### 調査
- tree-sitter-kotlin 0.3.8 (fwcd/tree-sitter-kotlin) が crates.io に存在することを確認
- 前 PR で `.kt` ファイルの `is_source_file` / `scan_jvm_project` / `classify_java_import_for_init` 対応済み
- パーサー実装のみ追加すれば `mille check` / `mille init` 両方が動く見込み

### 方針
- TDD: RED → GREEN → REFACTOR
- grammar ノード種別は `tree-sitter-kotlin` を `cargo add` して実際にパースして確認
