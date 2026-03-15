# Fix: mille init namespace package names (src/ layout)

## 問題

`mille init` が Python の `src/` レイアウトで `package_names` を誤って生成する。

`src/domain/**` のようなパスから `domain` のみを抽出するため、
実際に `from src.domain...` としてインポートしているプロジェクトでは
`src` が `package_names` に含まれず `external_allow` に紛れ込む。

## 解決方針

`py_pkg_names` 計算時に、いずれかのレイヤーの `external_allow` に含まれており
かつレイヤーパスのコンポーネントでもある名前を「名前空間パッケージ」として昇格する。

## タスク

- [x] ブランチ作成: `feat/pr62-fix-init-namespace-package-names`
- [x] テストを書く（RED）
  - [x] `test_generate_toml_namespace_src_layout_adds_src_to_package_names`
  - [x] `test_generate_toml_flat_layout_unchanged`
  - [x] `test_generate_toml_namespace_only_path_component_promoted`
- [x] 実装（GREEN）
- [x] リファクタ（REFACTOR）
- [x] `docs/TODO.md` 更新
- [x] `README.md` 更新
- [ ] PR 作成

## 完了条件

- 3テストがすべて通る
- 既存テストが壊れない
- `lefthook` が通る
