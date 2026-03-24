# Timeline: PR#75 ターゲットディレクトリ指定

## 2026-03-24

### 調査フェーズ
- args.rs, runner.rs, fs_source_file_repository.rs を確認
- 現状: glob 展開は CWD 基準、`--config` パスの親ディレクトリは resolver の外部設定参照用
- 方針: `std::env::set_current_dir(path)` で CWD を変更するのが最小影響
