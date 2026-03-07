# Timeline: PR 11 `mille init`

## 2026-03-07

### タスク開始

- プラン作成。設計方針：新規依存ゼロ（stdlib のみ）、`--force` フラグで E2E テスト可能、コアは純粋関数

### RED フェーズ（テスト先行）

テストを実装してから `--no-verify` でコミット。

**単体テスト（`src/usecase/init.rs`）:**
- `test_scan_layers_empty_dir` — 層らしいディレクトリが無ければ空 vec
- `test_scan_layers_detects_domain` — `src/domain/` があれば name="domain" を返す
- `test_scan_layers_detects_multiple` — domain + usecase + infrastructure を検出
- `test_detect_languages_rust` — .rs ファイルがあれば `["rust"]`
- `test_detect_languages_multiple` — .rs + .ts → `["rust", "typescript"]`
- `test_generate_toml_contains_project_section` — 生成 TOML に `[project]` が含まれる
- `test_generate_toml_contains_layer_sections` — 生成 TOML に `[[layers]]` が含まれる

**CLI args テスト（`src/presentation/cli/args.rs`）:**
- `test_parse_init_default_output` — `mille init` → output="mille.toml", force=false
- `test_parse_init_custom_output` — `--output custom.toml` → output="custom.toml"
- `test_parse_init_force_flag` — `--force` → force=true

**E2E テスト（`tests/e2e_init.rs`）:**
- `test_init_creates_toml_from_layer_dirs` — 一時ディレクトリに domain/ usecase/ infrastructure/ + .rs → exit=0, mille.toml 生成
- `test_init_with_output_flag` — `--output custom.toml` で custom.toml が生成
- `test_init_existing_file_without_force_exits_error` — mille.toml が既存 → exit!=0
- `test_init_existing_file_with_force_overwrites` — `--force` → exit=0, ファイル更新

### RED コミット実行

テスト書いて `--no-verify` でコミット。コンパイルエラーが期待値。

### GREEN フェーズ（実装）

`usecase::init` の純粋関数群を実装、`args.rs` に `Command::Init` 追加、`main.rs` にハンドラ追加。

### GREEN コミット

lefthook が通ることを確認してコミット。

### ドキュメント更新

`docs/TODO.md` + `README.md` 更新して PR 作成。
