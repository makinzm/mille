# PR65 タイムライン: naming-convention-check

## 2026-03-22

### ブランチ作成
- `feat/pr65-naming-convention-check` を main から切り出し

### タスクファイル作成
- `tasks/20260322-naming-convention/TODO.md` 作成
- `tasks/20260322-naming-convention/timeline.md` 作成

---

## RED フェーズ 1: 型定義・config パーステスト

### 実装予定テスト
- `test_layer_config_with_name_deny_parses`
- `test_layer_config_with_name_targets_parses`
- `test_layer_config_name_targets_default_is_all`
- `test_severity_config_with_naming_violation_parses`
- `test_severity_config_naming_violation_default_is_error`

### コンパイルエラーログ (RED フェーズ確認済み)

```
error[E0046]: not all trait items implemented, missing: `parse_names`
error[E0063]: missing fields `name_deny` and `name_targets` in initializer of `LayerConfig`
  (複数箇所: check_architecture.rs, init.rs, usecase test doubles 等)
error[E0004]: non-exhaustive patterns: `ViolationKind::NamingViolation` not covered
  (json.rs, github_actions.rs, terminal.rs の match v.kind)
error[E0063]: missing field `naming_violation` in initializer of `config::SeverityConfig`
  (複数箇所)
error: could not compile `mille` due to 17 errors
```

→ 期待通りの RED 状態。`--no-verify` でコミット後 GREEN フェーズへ。

---

## GREEN フェーズ

### 実装内容
- `parse_rust_names()`: function_item / struct_item / enum_item / trait_item / type_item / const_item / static_item / let_declaration / line_comment / block_comment
- `parse_ts_names()`: function_declaration / class_declaration / interface_declaration / type_alias_declaration / method_definition / variable_declarator / comment
- `parse_python_names()`: function_definition / class_definition / comment
- `parse_go_names()`: function_declaration / method_declaration / type_declaration (via type_spec) / var_declaration / const_declaration / short_var_declaration / comment
- `parse_java_names()`: class_declaration / interface_declaration / enum_declaration / method_declaration / constructor_declaration / field_declaration / local_variable_declaration / line_comment / block_comment
- `parse_kotlin_names()`: class_declaration / interface_declaration / object_declaration / function_declaration / property_declaration / multiline_comment / line_comment
- `detect_naming()`: 大文字小文字区別なし部分一致、name_targets フィルタリング、severity 設定対応
- `check_architecture::check()`: ファイルレベルチェック (basename 抽出) + parse_names() 呼び出し + detect_naming() 呼び出し

### テスト結果 (GREEN)
- ライブラリテスト: 308 passed
- E2E テスト (e2e_naming): 10 passed
- 全テストスイート: 全件パス

---

## REFACTOR フェーズ
- docs/TODO.md に PR65 完了チェックを追加
- README.md に `name_deny` / `name_targets` / `naming_violation` の Configuration Reference を追加
- feature matrix に Naming convention rules 行を追加
