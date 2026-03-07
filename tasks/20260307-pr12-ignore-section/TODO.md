# PR 12: `[ignore]` セクションの実装

## 概要

`mille.toml` の `[ignore]` セクションをパースするだけでなく、実際にチェックから除外する。

```toml
[ignore]
paths         = ["**/mock/**", "**/generated/**"]   # 完全に除外
test_patterns = ["**/*_test.go", "**/*.spec.ts"]    # テストファイル（依存ルールを適用しない）
```

## タスクリスト

- [ ] `check_architecture.rs` で `ignore.paths` に一致するファイルをコレクションから除外
- [ ] `check_architecture.rs` で `test_patterns` に一致するファイルの import を違反チェック対象外にする
- [ ] ユニットテストの追加（`check_architecture.rs` 内）
- [ ] E2E テストの追加（`tests/e2e_ignore.rs`）
- [ ] `docs/e2e_checklist.md` の確認・更新

## 受け入れ条件

- `ignore.paths` に一致するファイルは収集されず、layer stats にも含まれない
- `test_patterns` に一致するファイルは layer stats に含まれるが、依存違反は報告されない
- `ignore` セクションが未設定のとき（`None`）は従来通り全ファイルをチェック
