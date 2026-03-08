# タイムライン: PR 13 `mille analyze`

## 2026-03-08

### 設計フェーズ
- コードベース調査: `src/usecase/check_architecture.rs`, `src/runner.rs`, `src/presentation/cli/args.rs`, `tests/e2e_*.rs` を読んだ
- ui-ux-pro-max スキルでデザインシステム取得: dark theme (#0F172A), green accent (#22C55E), monospace
- ユーザーから「画像での表示もしてほしい」 → SVG フォーマット追加を決定
- テスト設計 10 ケースをユーザーに提示 → 承認取得

### RED フェーズ
- `tests/e2e_analyze.rs` を作成（10 テストケース）
- `src/usecase/analyze.rs` スタブ（`todo!()`）
- `src/presentation/formatter/svg.rs` スタブ（`todo!()`）
- `src/presentation/cli/args.rs` に `Analyze` サブコマンド + `AnalyzeFormat` 追加
- `src/runner.rs` に `Command::Analyze` ディスパッチ追加（`todo!()` で）
- `--no-verify` コミット完了

### GREEN フェーズ
- `src/usecase/analyze.rs` 実装: config → ファイル収集 → import 解析 → レイヤーレベルエッジ集計
- `src/presentation/formatter/svg.rs` 実装: トポロジカルソート + Kahn's algorithm でレイアウト、SVG XML 生成
- `src/runner.rs` の `todo!()` を `format_analyze_terminal / json / dot` インライン関数で置換
- `cargo test` 全 294 テスト通過、clippy クリーン

### REFACTOR フェーズ
- `README.md` に `mille analyze` セクション追加（`--format svg` 使用例含む）
- `docs/TODO.md` PR 13 チェックボックス更新
- `tasks/20260308-analyze/TODO.md` 実装状況反映
