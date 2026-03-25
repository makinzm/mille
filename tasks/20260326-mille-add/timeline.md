# Timeline: mille add 実装

## 2026-03-26

### コードベース調査
- `src/presentation/cli/args.rs`: Command enum, CommonArgs 構造を確認
- `src/runner.rs`: コマンドディスパッチパターンを確認
- `src/usecase/init.rs`: DirAnalysis, infer_layers, generate_toml を確認
- `src/domain/entity/layer.rs`: LayerConfig 構造体を確認
- `src/infrastructure/repository/toml_config_repository.rs`: TOML 読み込みパターンを確認
