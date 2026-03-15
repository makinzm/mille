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

### GREEN フェーズ (2026-03-15)

3 件の変更で全テスト通過 (276 passed; 0 failed):

1. `classify_py_import`: 絶対インポートをフルドットパスで返すよう変更
   - 変更前: `path.split('.').next()` → `"src"`
   - 変更後: `path.to_string()` → `"src.domain.entity"`

2. `resolve_to_known_dir`: プレフィックス照合を追加 (4段階戦略)
   - Strategy 1: スラッシュプレフィックス完全一致 + 同一親ディレクトリ
   - Strategy 2: スラッシュプレフィックス完全一致 (任意)
   - Strategy 3: ベース名一致 + 同一親ディレクトリ (Rust/Go の後方互換)
   - Strategy 4: ベース名一致 (任意)
   - NOTE: Strategy 3/4 を残すことで `Internal("domain")` → `"src/domain"` が継続動作

3. `collect_file_imports` (Bug 2): `ancestor_at_depth` が None のとき `dir_rel` をそのまま使用
   - 変更前: `None => return` → src/main.py がスキップされる
   - 変更後: `.unwrap_or_else(|| dir_rel.clone())`

regression 発見: `test_init_with_depth_flag` が Strategy 3/4 なしでは失敗
→ Rust `crate::domain` は `Internal("domain")` を返し known_dirs が `"src/domain"` のためベース名照合が必要

lefthook: clippy/fmt/test すべて通過

<!-- REFACTOR エントリをここに追記 -->
