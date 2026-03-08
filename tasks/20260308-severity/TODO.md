# PR 14: `[severity]` 設定の実装

## 目的

`mille.toml` の `[severity]` セクションを実際に動作させる。
現状はフィールドがパースされるだけで、違反は常に `Error` で出力されている。

## タスク

- [ ] `ViolationKind::UnknownImport` を追加（未解決 import を報告できるようにする）
- [ ] `ViolationDetector` が `SeverityConfig` を受け取るよう変更
- [ ] `detect()` が `dependency_violation` の severity を使う
- [ ] `detect_external()` が `external_violation` の severity を使う
- [ ] `detect_call_patterns()` が `call_pattern_violation` の severity を使う
- [ ] `detect_unknown()` を追加 — `ImportCategory::Unknown` を `unknown_import` severity で報告
- [ ] `check_architecture.rs` で `detect_unknown()` を呼ぶ
- [ ] `--fail-on` CLI オプション追加（`error` / `warning`）
- [ ] exit code: `--fail-on warning` のとき warning があれば exit 1
- [ ] unit テスト（`violation_detector.rs`）
- [ ] E2E テスト（`tests/e2e_severity.rs`）
- [ ] `docs/TODO.md` 更新
- [ ] `README.md` 更新

## 仕様参照

spec.md `[severity]` セクション:
- `dependency_violation` デフォルト `"error"`
- `external_violation` デフォルト `"error"`
- `call_pattern_violation` デフォルト `"error"`
- `unknown_import` デフォルト `"warning"`

値は `"error"` / `"warning"` / `"info"` から選択。

exit code:
- 0: 違反なし（または warning のみかつ `--fail-on warning` 未指定）
- 1: error 違反あり（または `--fail-on warning` 指定時に warning あり）
- 3: 設定エラー
