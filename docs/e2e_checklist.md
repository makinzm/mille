# E2E テスト網羅性チェックリスト

> **参照タイミング**: 新しい言語サポートを追加したとき・フィクスチャを変更したとき・設定項目の実装を変更したとき

## 原則

- 1 テスト = 1 設定項目の違反（他は全て正常にする）
- 正常系だけでは不十分 — 壊したとき失敗しなければカバレッジとして無価値
- 対象言語ごとに以下の全項目を確認する

---

## チェックリスト

- [ ] dep opt-in: `allow` を壊す → 依存違反
- [ ] dep opt-out: `deny` を設定 → 依存違反
- [ ] external opt-in: `external_allow = []` → 外部違反
- [ ] external opt-out: `external_deny` を設定 → 外部違反
- [ ] allow_call_patterns: 禁止メソッド呼び出し → `CallPatternViolation`

---

## 言語固有の注記

### YAML (naming-only)

YAML は import の概念がないため、以下の項目は N/A:
- dep opt-in / dep opt-out: N/A
- external opt-in / external opt-out: N/A
- allow_call_patterns: N/A

YAML で有効なチェック:
- [x] name_deny: キー (Symbol) と値 (StringLiteral) の命名チェック
- [x] name_targets: `["symbol"]` / `["string_literal"]` で対象の絞り込み
