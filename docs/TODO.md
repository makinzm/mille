# mille 開発 TODO

本リストは、`spec.md` に定義された仕様に基づき、スクラム開発でイテレーティブに動くもの（価値）を段階的につくることができるよう、PRの粒度で縦切りにしたタスクリストです。

## フェーズ 0: プロジェクト初期設定と名前確保
各パッケージマネージャーでの名前（`mille`等）を早期に確保するため、ダミーパッケージによるプレリリースを行います。

### [x] PR 1: パッケージ名予約のためのダミーCD構築とリリース
- [x] `cargo init` によるRustプロジェクトの初期化と最小の `Cargo.toml` 設定
- [x] GitHub Actions (`.github/workflows/cd-reserve.yml` 等) を用いたプレリリース（空パッケージ）のCI/CDパイプライン構築
- [x] npm, pypi, crates.io, (必要に応じて go/Homebrew 等) への `mille` (または類似の利用可能名) パッケージの初版デプロイ
- [x] ※ この段階で `rust-toolchain.toml` や `lefthook` などの基本的な開発環境もあわせて整備する

## フェーズ 1: 基盤構築と最小PoC (RustをターゲットにしたDogfooding)
mille自身のソースコード（Rust）を解析対象として、最速で「ファイルを入力して依存違反を検出・報告する CLI コマンド」が動くラインを目指します。セルフチェック（Dogfooding）によりTDDを推進します。

### [x] PR 2: 設定ファイル（`mille.toml`）のパースとコアエンティティ
- [x] `domain` レイヤーに `Layer`, `DependencyRule` などのエンティティとRepositoryトレイトを定義
- [x] `infrastructure` に `toml_config_repository` を実装
- [x] mille自身のアーキテクチャを定義した `mille.toml` の作成
- [x] 不正なTOMLの異常系テスト、正常系のパース機能をTDDで実装

### [x] PR 3: tree-sitterによる import 抽出器 (Rust用)
- [x] `domain` に `RawImport` エンティティ定義
- [x] `infrastructure` の `parser::rust` 実装 (tree-sitterを用いたASTからの `use` / `mod` 抽出)
- [x] mille自身のコードに対するAST import文抽出テスト実装

### [x] PR 4: Rustモジュールのパス解決とレイヤー依存違反チェック層の実装
- [x] `RawImport` からパスを正規化し `internal/external` 等を判別するロジック (`resolver`) 実装 (Rustの `crate::`, `super::` などの解決)
- [x] 解決されたパス情報と `Layer` 定義を突き合わせる判定ロジック実装
- [x] 依存ルール (dependency_mode) ベースで違反 (`Violation`) を返す `violation_detector` の実装

### [x] PR 5: `mille check` コマンド(CLI) の結合とエンドツーエンド動作
- [x] `presentation::cli::args` にて `clap` を用いた引数パース実装
- [x] `usecase::check_architecture` で一連のフロー（パース→解決→判定）を実装
- [x] ターミナル用フォーマッター（TerminalFormatter）の実装と違反箇所の標準出力
- [x] 結合テスト（mille自身のコードと `mille.toml` を用いて違反が検出できるかE2Eで確認）
- [x] CIパイプライン (`.github/workflows/ci.yml`) に `mille check` (Dogfooding) を組み込む

---

## フェーズ 2: 精度向上と複数言語(TypeScript/Go)への展開
より実用的な仕様のサポートと、他の主要言語への対応を追加します。

### [x] PR 6: メソッド呼び出しチェック（`allow_call_patterns`）機能の実装
- [x] Rust用 tree-sitter でのメソッド呼び出し（`call_expression`）構文抽出ロジックの追加
- [x] 依存ルールエンジンに `allow_call_patterns` 制約への違反検出機能を追加
- [x] `Violation` / `ViolationKind` に `CallPatternViolation` を追加

### [x] PR 7: 外部ライブラリ依存（`external_mode`）チェックの実装
- [x] `external_allow` / `external_deny` 正規表現での判定ロジック実装
- [x] `external` カテゴリに分類されたインポートの許可/拒否のテスト追加

### [x] PR 8: Go言語サポートの追加
- [x] `infrastructure::parser::go` 実装 (tree-sitter-go)
- [x] `infrastructure::resolver::go` 実装 (stdlib/internal/external 判定)
- [x] `DispatchingParser` / `DispatchingResolver` で拡張子ベースのルーティング
- [x] `FsSourceFileRepository` で `.go` ファイルも収集
- [x] Go フィクスチャ (`tests/fixtures/go_sample/`) と E2E テスト (`tests/e2e_go.rs`) 追加
- [x] `go.mod` に対応したパス解決と結合テスト

### PR 8.5: Python support

### PR 9: TypeScript / JavaScript サポートの追加
- [ ] `infrastructure::parser::typescript` 実装
- [ ] `tsconfig.json` の `paths` / `baseUrl` パースおよびエイリアス解決のサポート追加
- [ ] TSのダミープロジェクトを用いた結合テスト

---

## フェーズ 3: 出力フォーマットの拡充と分析機能

### PR 10: 出力フォーマットの多角化（JSON / GitHub Actions）
- [ ] CLI オプション `--format` 対応
- [ ] JSON形式およびGitHub Actions ( `::error` ) 形式の Formatter 実装

### PR 11: 分析機能とレポート機能の追加
- [ ] `mille analyze` コマンドの実装（DOT形式による依存グラフ出力）
- [ ] `mille report external` コマンドの実装

### PR 12...: 他言語サポート (Python / Java / Dart等)
- [ ] Python / Java / Dart 等のパーサとパスリゾルバの実装追加

---

## フェーズ 4: CI/CD・エコシステムパッケージの本格展開

### PR X: mille self-check の lefthook / CI への組み込み
> ⚠️ Secret（GITHUB_TOKEN 等）の発行が必要なため、権限整備後に実施する。

- [ ] `lefthook.yml` の pre-commit フックに `cargo run -- check` (mille 自己チェック) を追加
- [ ] `.github/workflows/ci.yml` に `mille check` ステップを追加し、mille 自身の依存ルールを CI で常時検証する
- [x] `mille.toml` の `allow_call_patterns` を整備し、main レイヤーの DI 呼び出し制約を宣言する

### PR N: 配布用パッケージ群の完成と自動リリース
- [ ] GitHub Actionsでのクロスコンパイル対応とGitHub Releasesへのバイナリ配布設定
- [ ] ダミーパッケージだった npm, PyPI (uv), Go, Cargo 用のラッパー CLI パッケージを、実際のバイナリをDL・実行する実装に更新

### PR N+1: wazero CompilationCache の導入（Go ラッパー起動高速化）
- 背景: wazero は起動のたびに .wasm を JIT なしで機械語に変換するため、
  ネイティブ `cargo build` バイナリに比べ起動が大幅に遅い（1/3〜1/10 程度）。
  CompilationCache を使えば 2 回目以降の起動で変換済みキャッシュを再利用でき、
  コールドスタートのコストが解消される。
- [ ] `wazero.NewRuntimeWithConfig` + `wazero.NewRuntimeConfigCompilationCache` を使用
- [ ] キャッシュディレクトリを `os.UserCacheDir()` 配下（例: `~/.cache/mille/wazero`）に設定
- [ ] キャッシュを使ったときと使わないときの起動時間を比較するベンチマークテストを追加
- [ ] `docs/administrator/wasm_build.md` にキャッシュの仕組みと場所を記載

### [x] PR N+2: E2E テスト網羅性向上 + Go external_allow バグ修正 + UX 改善
- 背景: Go の `external_mode = "opt-in"` が stdlib に対して機能しないバグ、E2E テスト不足、UX 上の問題を一括修正。
- [x] `classify_go()` が stdlib を `Stdlib` に分類していたバグを修正（全非内部 import を `External` として扱う）
- [x] `external_allow` / `external_deny` のマッチングを正規表現→完全一致に変更（`github.com` のようなエスケープ不要の記述へ）
- [x] `packages/go/mille.toml` の `external_allow` を全使用インポートで整備（wazero サブパッケージ含む）
- [x] Go・Rust 両方の E2E テストに「意図的に壊す」ケースを追加（dep opt-in / external opt-in）
- [x] `ExternalViolation` メッセージを英語化
- [x] 0 ファイルレイヤーを `⚠️` で表示（設定ミス検知）
- [x] `paths = ["*.go"]` のような bare glob パターンに対応
- [x] `docs/e2e_checklist.md` を新規作成（永続チェックリスト）
- [x] `README.md` の Go セクションを WASM/wazero 方式の説明に更新
