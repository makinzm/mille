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
