# Timeline

## 2026-03-24

### 調査フェーズ

- ユーザーから報告: `name_deny = ["gcp"]` が `cfg.gcp.staging_bucket`（属性アクセス）を検出しない
- 同様に docstring 内の `cfg.gcp.project` も通過している
- Python パーサー (`python.rs`) を調査: `collect_python_names` は `function_definition`, `class_definition`, `assignment`, `comment`, `string` のみ処理
- `attribute` ノード（ドットアクセス）は処理対象外 → `gcp` が抽出されない原因
- docstring は tree-sitter 上では `string` ノードなので理論的には `StringLiteral` として抽出されるはず → テストで検証したところ正常に動作

### RED フェーズ

- `NameKind::Identifier` が存在しないためコンパイルエラー（期待通り）
- テスト追加:
  - `test_py_parse_names_attribute_identifier`: 属性アクセスから Identifier 抽出
  - `test_py_parse_names_docstring_string_literal`: docstring が StringLiteral として検出される確認
  - `test_detect_naming_identifier_violation`: Identifier が name_deny にマッチ
  - `test_detect_naming_target_filter_excludes_identifier`: name_targets フィルタ

### GREEN フェーズ

- `NameKind::Identifier` / `NameTarget::Identifier` 追加
- `ParsedNames` に `identifiers` フィールド追加（コンパイルタイムガード: 全パーサーで追加必須）
- 全8言語パーサーに属性アクセス抽出を追加:
  - Python: `attribute` ノード
  - Rust: `field_expression` ノード
  - TypeScript: `member_expression` ノード
  - Go: `selector_expression` ノード
  - Java: `field_access` ノード
  - Kotlin: `navigation_expression` ノード
  - PHP: `member_access_expression` / `scoped_property_access_expression` ノード
  - C: `field_expression` ノード
- dogfooding テストで誤検知発覚: `import.category` の `category` が `go` にマッチ
  - `mille.toml` の usecase レイヤー `name_allow` に `"category"` 追加で解決
- 全375ユニットテスト + 全E2Eテスト パス
