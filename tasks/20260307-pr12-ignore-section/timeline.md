# Timeline: PR 12 - [ignore] セクションの実装

## 2026-03-07

### 調査
- `check_architecture.rs`: ファイル収集 → パース → 依存検出のフローを確認
- `FsSourceFileRepository`: パターンで収集するが ignore フィルタリングなし
- `IgnoreConfig`: 既にパースされているが `check()` では使われていない

### 実装方針
- `check_architecture.rs` の `check()` 内で:
  1. `ignore.paths` に一致するファイルを layer ファイルリストから除外（収集後フィルタ）
  2. `test_patterns` に一致するファイルから生成された resolved import を violations 検出から除外
- glob マッチには `glob::Pattern` を使用（既存 dependency）

### RED → GREEN
- E2E テスト初回実行: 4件失敗（fixture 設計ミス）
  - 他レイヤーも external_allow=[] により違反を出していた
  - domain ファイルが serde を import → ExternalViolation
  - main.rs が serde_json を import → ExternalViolation
- fixture を opt-out ベースに変更し infrastructure のみ violations が出る設計に修正
- 再実行: 7件全通過 ✅

### 変更ファイル
- `src/usecase/check_architecture.rs`: `matches_any_glob` ヘルパーと ignore 適用ロジック
- `tests/e2e_ignore.rs`: E2E テスト7件
