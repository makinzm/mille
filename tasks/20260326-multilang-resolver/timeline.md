# Timeline: PR#80 複数言語 fixture テスト + Resolver リファクタ

## 2026-03-26

### 調査フェーズ
- DispatchingResolver の構造を確認 → 全7言語の Resolver を常に生成する Fat Dispatcher
- 既存 fixture は全て単一言語 → 複数言語混在のテストが不在
- ユーザーと方針合意: Phase 1 で multilang テスト → Phase 2 で Resolver リファクタ

### テスト作成
- Fixture A (multilang_mixed_sample): TS + PY + Go 混在レイヤー — 11テスト全通過
- Fixture B (multilang_split_sample): ts/ py/ go/ 言語分離 — 11テスト全通過
- 既存テスト全通過確認済み

### CI dogfooding
- ci.yml の dogfood-rust ジョブに multilang_mixed_sample / multilang_split_sample 追加

### リファクタ
- DispatchingResolver を Registry パターンに移行
  - 7言語フィールド → HashMap<&str, Box<dyn Resolver>>
  - from_resolve_config() に languages パラメータ追加
  - runner.rs 3箇所で app_config.project.languages を渡すよう変更
- 初回テスト: .java (5文字) が 4文字上限の拡張子マッチに引っかかり Java テスト全滅
  - resolver_for() を rfind('.') ベースに修正 → 全テスト通過
