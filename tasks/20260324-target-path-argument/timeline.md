# Timeline: PR#75 ターゲットディレクトリ指定

## 2026-03-24

### 調査フェーズ
- args.rs, runner.rs, fs_source_file_repository.rs を確認
- 現状: glob 展開は CWD 基準、`--config` パスの親ディレクトリは resolver の外部設定参照用
- 方針: `std::env::set_current_dir(path)` で CWD を変更するのが最小影響

### RED フェーズ
- args unit test 9件 + E2E test 3件を追加（CommonArgs / Command::common() 未実装でコンパイルエラー）
- `--no-verify` でコミット

### GREEN フェーズ
- `CommonArgs` 構造体 + `#[command(flatten)]` で全コマンドに path フィールド埋め込み
- `Command::common()` / `ReportCommand::common()` で exhaustive match → 新コマンド追加時にコンパイルエラー保証
- `runner.rs`: `apply_path()` で CWD 変更、存在しないパスはエラー（exit 3）
- 全 584 テスト通過、lefthook 通過

### ドキュメント更新
- README.md: 各コマンドに PATH 指定の使用例追加
- docs/TODO.md: 実装状況サマリーに追記
