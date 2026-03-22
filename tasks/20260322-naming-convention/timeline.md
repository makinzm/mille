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

### コンパイルエラーログ
(RED フェーズ実施後に記録)
