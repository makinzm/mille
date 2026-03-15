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

### 次: GREEN フェーズ

`py_pkg_names` 計算ロジックに namespace_pkgs の収集処理を追加する。
