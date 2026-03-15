# Timeline: Fix mille init scan logic for src/ namespace layout

## 2026-03-15

### 準備フェーズ
- ブランチ `feat/pr63-fix-init-scan-namespace-imports` 作成
- `tasks/20260315-fix-init-scan-namespace-imports/TODO.md` 作成
- `tasks/20260315-fix-init-scan-namespace-imports/timeline.md` 作成

### 調査結果
- `classify_py_import` (src/runner.rs:884): 現在は先頭セグメントのみ返す
- `resolve_to_known_dir` (src/runner.rs:942): `module_seg` をディレクトリのベース名と比較するだけ
- テストは既存なし (`test_classify_py_import`, `test_resolve_to_known_dir` ともに未定義)

---

### RED フェーズ (2026-03-15)

テスト 5 件を追加。現在の実装では 4 件が失敗することを確認。

```
test runner::tests::test_classify_py_import_returns_full_path ... FAILED
  left: "src"  right: "src.domain.entity"
test runner::tests::test_resolve_to_known_dir_dotted_namespace ... FAILED
test runner::tests::test_resolve_to_known_dir_flat_import_still_works ... FAILED
test runner::tests::test_scan_main_py_creates_layer ... FAILED
test runner::tests::test_ancestor_at_depth_shallower_returns_none ... ok  (regression guard)
test result: FAILED. 272 passed; 4 failed
```

根本原因:
- `classify_py_import`: 先頭セグメント `"src"` のみ返す → フルパス必要
- `resolve_to_known_dir`: ベース名比較のみ → プレフィックス照合必要
- `ancestor_at_depth` が None のとき `return` → src/main.py がスキップされる

<!-- GREEN/REFACTOR エントリをここに追記 -->
