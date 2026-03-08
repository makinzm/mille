# PR 15: `mille report external`

## 概要
各レイヤーが実際にどの外部パッケージを import しているかを一覧表示するサブコマンド。

## 実装ステップ

- [ ] RED: ユニットテスト + E2Eテストのスタブを `--no-verify` でコミット
- [ ] GREEN: usecase / CLI / formatter を実装して全テスト通過
- [ ] REFACTOR: README.md + docs/TODO.md 更新 → PR 作成

## 機能仕様

### CLI
```
mille report external [--config <path>] [--format terminal|json] [--output <path>]
```
- `--config`: デフォルト `mille.toml`
- `--format`: `terminal`（デフォルト）/ `json`
- `--output`: 指定時はファイルへ書き込み（既存ファイルは上書き拒否）

### 出力（terminal）
```
External Dependencies by Layer

  domain          (none)
  usecase         (none)
  infrastructure  database/sql
  cmd             fmt, os
```

### 出力（json）
```json
[
  {"layer": "domain", "packages": []},
  {"layer": "infrastructure", "packages": ["database/sql"]}
]
```

## テスト設計

### ユニット (`src/usecase/report_external.rs`)
- `test_groups_packages_by_layer`
- `test_deduplicates_same_package`
- `test_skips_non_external_imports`
- `test_packages_are_sorted`
- `test_skips_files_not_in_any_layer`
- `test_skips_mod_declarations`

### E2E (`tests/e2e_report_external.rs`)
- `test_e2e_report_external_terminal` (go_sample)
- `test_e2e_report_external_json` (go_sample)
- `test_e2e_report_external_no_external_layers` (go_sample の domain/usecase)
