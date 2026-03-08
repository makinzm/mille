# Timeline

## 2026-03-08

### 調査・設計
- spec.md, runner.rs, args.rs, violation_detector.rs を読んで設計確認
- Go resolver が stdlib も External に分類する仕様を確認（database/sql, fmt, os が External）
- E2E fixture は既存の go_sample を使用
- `--output` オプションを全フォーマットで有効にする（analyze と同じ挙動）

### RED phase
- `compute_external_report` 純粋関数を中心にユニットテスト設計
- `report_external` が `std::fs::read_to_string` を呼ぶため、FixedFileRepo+FixedParser アプローチでは
  ファイルが実在しないとエラーになることが発覚 → 純粋計算関数 `compute_external_report` を分離する設計に変更

### GREEN phase
- `src/usecase/report_external.rs`: `compute_external_report` 実装、ユニットテスト 10 件通過
- `src/presentation/cli/args.rs`: `ReportCommand::External` + `ReportExternalFormat` 追加
- `src/runner.rs`: `Command::Report` match arm + formatter 2 関数追加
- E2E テスト 4 件通過（go_sample fixture）

### REFACTOR phase
- README に `mille report external` セクション追加
- docs/TODO.md 更新（PR 15 完了マーク）
