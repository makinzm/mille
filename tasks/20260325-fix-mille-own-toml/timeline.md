# Timeline: PR#79

## 2026-03-25

### 調査
- src/main.rs は 1 行（`mille::runner::run_cli()`）
- src/runner.rs は全レイヤー import、clap 使用、どのレイヤーにも未所属
- runner.rs が使うメソッド: infrastructure (DispatchingParser::new, DispatchingResolver::from_resolve_config 等), usecase (check, analyze, report_external, detect_languages, infer_layers, generate_toml, is_excluded_dir 等), presentation (Cli::parse, format_* 等)

### 実装
- main レイヤーを entrypoint + runner に分離
  - entrypoint: src/main.rs — runner のみ依存許可（厳格）
  - runner: src/runner.rs — 全レイヤー + clap + allow_call_patterns
- mille check 通過確認（entrypoint 1 file, runner 1 file）
- mille analyze --format svg で mille.svg + website/src/assets/mille.svg 更新
- 全テスト通過
