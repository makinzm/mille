# Timeline: mille add 実装

## 2026-03-26

### コードベース調査
- `src/presentation/cli/args.rs`: Command enum, CommonArgs 構造を確認
- `src/runner.rs`: コマンドディスパッチパターンを確認
- `src/usecase/init.rs`: DirAnalysis, infer_layers, generate_toml を確認
- `src/domain/entity/layer.rs`: LayerConfig 構造体を確認
- `src/infrastructure/repository/toml_config_repository.rs`: TOML 読み込みパターンを確認

### RED phase
- CLI args テスト 5件: test_parse_add_basic, with_config, with_name, with_force, default_target
- Usecase unit テスト 8件: find_conflict (3), build_layer_config (3), layer_to_toml_string, replace_layer_in_table
- E2E テスト 8件: add_new_layer, preserves_existing, preserves_resolve, conflict_without_force, conflict_with_force, custom_name, config_not_found, target_not_directory
- `todo!()` で E2E テスト失敗確認

### GREEN phase
- `Command::Add` バリアント実装（args.rs）
- `add_layer.rs`: find_conflict, build_layer_config, layer_to_toml_string, replace_layer_in_table 実装
- `runner.rs`: scan_single_dir + Command::Add ディスパッチ実装
- 全 21 テストパス（5 CLI + 8 unit + 8 E2E）
- 既存 e2e_check テスト 5件は main でも失敗（無関係）

### ドキュメント
- README.md に `mille add` セクション追加
- TODO.md 完了チェック

### バグ修正（ユーザー指摘）
- `is_source_file` が `.yaml`/`.yml`/`.php`/`.c`/`.h` を含んでおらず、後発言語がスキャン対象外だった → 全言語を追加
- dogfood テスト失敗を見落とし:
  1. `mille.toml` の `allow_call_patterns` に add_layer の関数が未登録 → CallPatternViolation
  2. テスト内の文字列 `"rust"` が usecase の `name_deny` に引っかかる → `"lang_a"` に変更
- **教訓**: `cargo test` 全体を通してから `--no-verify` でコミットすべき
