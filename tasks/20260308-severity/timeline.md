# Timeline

## 2026-03-08

### 計画
- PR 14: `[severity]` 設定の実装（`unknown_import` 含む）
- ブランチ: `feat/pr14-severity`

### RED フェーズ（`b471080`）
- `ViolationKind::UnknownImport` 追加
- フォーマッタに `UnknownImport` アーム追加
- `SeverityConfig::default()` 実装
- `ViolationDetector::with_severity()` と `detect_unknown()` をスタブで追加
- 新 unit テスト（severity 設定の動作検証）
- `FailOn` enum + `--fail-on` CLI arg 追加
- `tests/e2e_severity.rs` 新規作成
- `--no-verify` でコミット

### GREEN フェーズ（`bbcb3ad`）
- `detect_unknown()` 実装
- `parse_severity()` ヘルパー追加
- `detect()` / `detect_external()` / `detect_call_patterns()` を severity config から取得するよう変更
- `check_architecture.rs` で `with_severity()` + `detect_unknown()` 呼び出し
- `runner.rs` で `--fail-on` の exit code ロジック実装
- `e2e_check.rs` の既存テスト（`new` → `with_severity` の変更に追従）更新
- 全テスト合格・lefthook 通過

### REFACTOR フェーズ
- README に `[severity]` セクションと `--fail-on` 追記
- `docs/TODO.md` 更新（PR 14 完了チェック）
