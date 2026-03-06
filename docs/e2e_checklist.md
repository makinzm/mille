# E2E テスト網羅性チェックリスト

> **参照タイミング**: 新しい言語サポートを追加したとき・フィクスチャを変更したとき・設定項目の実装を変更したとき

## 原則

- 1 テスト = 1 設定項目の違反（他は全て正常にする）
- 正常系だけでは不十分 — 壊したとき失敗しなければカバレッジとして無価値
- 新しい言語を追加するときは全 5 項目を揃える

---

## Rust

- [ ] dep opt-in: `allow` を壊す → 依存違反
- [ ] dep opt-out: `deny` を設定 → 依存違反
- [ ] external opt-in: `external_allow = []` → 外部違反
- [ ] external opt-out: `external_deny` を設定 → 外部違反
- [ ] allow_call_patterns: 禁止メソッド呼び出し → `CallPatternViolation`

## Go

- [ ] dep opt-in: `allow` を壊す → 依存違反
- [ ] dep opt-out: `deny` を設定 → 依存違反
- [ ] external opt-in: `external_allow = []` → 外部違反
- [ ] external opt-out: `external_deny` を設定 → 外部違反
- [ ] allow_call_patterns: 禁止メソッド呼び出し → `CallPatternViolation`

## Python

- [ ] dep opt-in: `allow` を壊す → 依存違反
- [ ] dep opt-out: `deny` を設定 → 依存違反
- [ ] external opt-in: `external_allow = []` → 外部違反
- [ ] external opt-out: `external_deny` を設定 → 外部違反
- [ ] allow_call_patterns: 禁止メソッド呼び出し → `CallPatternViolation`

## TypeScript

- [ ] dep opt-in: `allow` を壊す → 依存違反
- [ ] dep opt-out: `deny` を設定 → 依存違反
- [ ] external opt-in: `external_allow = []` → 外部違反
- [ ] external opt-out: `external_deny` を設定 → 外部違反
- [ ] allow_call_patterns: 禁止メソッド呼び出し → `CallPatternViolation`

## JavaScript

- [ ] dep opt-in: `allow` を壊す → 依存違反
- [ ] dep opt-out: `deny` を設定 → 依存違反
- [ ] external opt-in: `external_allow = []` → 外部違反
- [ ] external opt-out: `external_deny` を設定 → 外部違反
- [ ] allow_call_patterns: 禁止メソッド呼び出し → `CallPatternViolation`
