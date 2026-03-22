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
