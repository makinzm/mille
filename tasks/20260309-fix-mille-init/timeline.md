# タイムライン

## 2026-03-09

### 調査フェーズ

- `src/usecase/init.rs`: `infer_layers()` の Pass 1 でサブプロジェクト間の同名ディレクトリがマージされる問題を確認
  - 現状: `by_parent` のキーは immediate parent の base name のみ → `crawler/src/domain` と `ingest/src/domain` が同じ parent `src` として扱われ1レイヤーに合流
- `src/infrastructure/repository/fs_source_file_repository.rs`: `collect()` で glob 展開後に `.venv` 除外がないことを確認
- `src/domain/service/violation_detector.rs`: `detect_external()` が `split("::")` のみで Python の `.` 区切りに未対応

### RED フェーズ

テスト作成 (`--no-verify` でコミット):
- `test_infer_layers_groups_dirs_by_base_name`: マージされず別レイヤーになることを検証 (Fix 1)
- `test_infer_layers_separate_same_name_dirs_different_subproject`: crawler/ingest パターンの別レイヤー化を検証 (Fix 1)
- `test_collect_skips_venv_paths`: `.venv` を含むパスが除外されることを検証 (Fix 2)
- `test_detect_external_python_submodule_allowed`: `matplotlib.pyplot` が `["matplotlib"]` で許可されることを検証 (Fix 3)
- `test_detect_external_python_submodule_violation`: `unknown.submodule` が violation になることを検証 (Fix 3)
- `test_detect_external_rust_colon_still_works`: 既存の Rust `::` 区切りが引き続き動作することを確認 (Fix 3 regression)

### GREEN フェーズ

実装:
- Fix 1: `find_distinguishing_prefix()` 関数を追加し、同名ディレクトリの区別ロジックを実装
- Fix 2: `has_excluded_component()` フィルターを `collect()` に追加
- Fix 3: ファイル拡張子で `.py` を判定し、`.` 区切りで crate_name を抽出
