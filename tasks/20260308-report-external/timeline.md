# Timeline

## 2026-03-08

### 調査・設計
- spec.md, runner.rs, args.rs, violation_detector.rs を読んで設計確認
- Go resolver が stdlib も External に分類する仕様を確認（database/sql, fmt, os が External）
- E2E fixture は既存の go_sample を使用
- `--output` オプションを全フォーマットで有効にする（analyze と同じ挙動）

### RED phase
- ユニットテスト（スタブ実装）と E2E テストを `--no-verify` でコミット予定
