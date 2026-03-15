# Timeline

## 2026-03-15

### 準備フェーズ

- `main` を最新化（v0.0.12 タグのバージョン更新を取り込み）
- ブランチ `feat/pr62-fix-init-namespace-package-names` 作成
- `src/usecase/init.rs` の `py_pkg_names` 計算箇所（398-410行目）を確認
- `generate_toml` のシグネチャと既存テスト構造を確認
- タスクディレクトリ・TODO.md・timeline.md 作成

### RED フェーズ

3つのテストを `src/usecase/init.rs` の `#[cfg(test)]` ブロックに追加。

失敗確認:
```
test usecase::init::tests::test_generate_toml_namespace_src_layout_adds_src_to_package_names ... FAILED
test usecase::init::tests::test_generate_toml_namespace_only_path_component_promoted ... FAILED
```

失敗メッセージ（Test 1）:
```
src は package_names に昇格されるべき
package_names = ["domain", "usecase"]  ← src が含まれていない
external_allow = ["src", "dataclasses"] ← src がまだ external_allow にある
```

`test_generate_toml_flat_layout_unchanged` は既存動作と一致するため PASSED（正常）。

### GREEN フェーズ

`src/usecase/init.rs` の `py_pkg_names` 計算ロジックを修正:
- `base`: 従来通り last path component を収集
- `all_components`: 全パスコンポーネントを収集（例: `src/domain/**` → `["src", "domain"]`）
- `namespace_pkgs`: `external_allow` に含まれ、かつ `all_components` にも含まれるものを昇格

```
test usecase::init::tests::test_generate_toml_namespace_src_layout_adds_src_to_package_names ... ok
test usecase::init::tests::test_generate_toml_flat_layout_unchanged ... ok
test usecase::init::tests::test_generate_toml_namespace_only_path_component_promoted ... ok
全36テスト通過、lefthook 通過
```

### REFACTOR フェーズ

- README.md に Python `src/` layout の説明を追加
- docs/TODO.md に PR #62 完了項目を追加
- tasks/TODO.md を更新
