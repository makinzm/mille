# mille 開発 TODO

本リストは、`spec.md` に定義された仕様に基づく残タスクです。
完了済みのPRは削除済み（git履歴で参照可）。

---

## 実装状況サマリー

現在の `mille check` は以下を正常に動作させています：
- ✅ 内部レイヤー依存チェック (`dependency_mode`)
- ✅ 外部ライブラリ依存チェック (`external_mode`)
- ✅ DIエントリーポイントのメソッド呼び出しチェック (`allow_call_patterns`)
- ✅ Rust / Go / TypeScript / JavaScript / Python サポート
- ✅ `[resolve.typescript]` tsconfig.json paths エイリアス解決
- ✅ cargo / npm(WASM) / go install / pip パッケージ配布
- ✅ リリース後のバージョン自動同期（`update-version` ジョブ）— `mille --version` がリリースタグと一致
- ✅ `--format terminal / json / github-actions` 出力フォーマット切り替え（PR 10）
- ✅ `[ignore]` セクション — `paths` / `test_patterns` 適用（PR 12）
- ✅ `mille init` コマンド — プロジェクトスキャンして `mille.toml` 自動生成（PR 11）、必須フィールド `external_mode` の生成漏れ修正済み
- ✅ `mille analyze` — 依存グラフ可視化 `terminal / json / dot / svg`（PR 13）
- ✅ `[severity]` — 違反種別ごとの重大度設定 + `--fail-on` オプション（PR 14）
- ✅ `mille report external` — 外部ライブラリ依存をレイヤーごとにテーブル/JSON出力（PR 15）
- ✅ `mille init` 精度改善 — 異サブプロジェクトの同名ディレクトリ分離、`.venv` スキャン除外、Python サブモジュール `external_allow` マッチング修正（PR #55）
- ✅ `mille init` Go+TypeScript 対応改善 — `go.mod` から `module_name` 自動検出・生成、Go external_allow に完全パス使用、TypeScript サブパス (`vitest/config` → `vitest`) のマッチング修正（PR #56）
- ✅ Java 言語サポート — `.java` ファイルのパース・Internal/External 分類、`[resolve.java] module_name` 設定、E2E テスト追加（PR #57）

以下は **設定ファイルにフィールドが存在しても、まだ動作していない** 項目です（README に掲載しないよう修正済み）：
（現在なし）

---

## フェーズ 3: 出力・CI連携（バズりやすい順）

### PR 10: GitHub Actions アノテーション出力 (`--format github-actions`) ✅ 完了

**バズりポイント**: PRレビュー画面に `::error file=...` が差し込まれるため、mille を使っているリポジトリのPRを見た人が「これ何？」となりやすい。CI を通じたパッシブな口コミ効果が最大。

- [x] CLI に `--format` オプションを追加（`terminal` / `json` / `github-actions`）
- [x] GitHub Actions (`::error file=<path>,line=<n>::<msg>`) フォーマッターの実装
- [x] JSON フォーマッターの実装
- [x] CI ドキュメントに GitHub Actions 設定例を追記（`docs/github-actions-usage.md`）

### PR 11: `mille init` コマンド（インタラクティブ設定生成）✅ 完了

**バズりポイント**: 「`mille init` を叩くだけで始められる」というオンボーディング体験は口コミで広まりやすい。Time-to-first-value の短縮が採用数に直結する。

- [x] `mille init` サブコマンドの追加
- [x] 実際のインポート文を解析してレイヤーと依存関係を推論する（トポロジカルソート）
- [x] `--output <path>` / `--force` フラグのサポート
- [x] `mille.toml` 自動生成（副作用なし純粋関数 + E2E テスト）、`external_allow` も実インポートから生成
- [x] `--depth N` フラグ + 自動深度検出（深いネスト構造を正しくロールアップ）

### PR 12: `[ignore]` セクションの実装 ✅ 完了

**バズりポイント**: テストファイルの除外は必須ユースケース。これがないと「mille 使えない」という評価につながる。採用の障壁を下げるために優先度高。

- [x] `check_architecture::check()` で `ignore.paths` のグロブパターンを除外
- [x] `check_architecture::check()` でテストファイルに対して依存ルールを緩める（`test_patterns`）
- [x] E2E テストの追加

### PR 13: `mille analyze` コマンド（依存グラフ可視化）✅ 完了

**バズりポイント**: DOT/SVG グラフはスクリーンショットとして SNS に貼りやすく、「自分のプロジェクトのアーキテクチャが可視化された」という体験は Twitter/X やブログで紹介されやすい。

- [x] `mille analyze` サブコマンドの追加
- [x] DOT 形式での依存グラフ出力 (`--format dot`)
- [x] レイヤー間エッジの集計（ファイルレベルではなくレイヤーレベル）
- [x] SVG 形式での自己完結グラフ画像出力 (`--format svg`)
- [x] JSON 形式出力 (`--format json`)

### PR 14: `[severity]` 設定の実装 ✅ 完了

**バズりポイント**: warning/error の区別は段階的導入を可能にし、「既存プロジェクトへの追加しやすさ」を向上させる。採用率に寄与。

- [x] `ViolationDetector` に `SeverityConfig` を渡すようにする（`with_severity()` コンストラクタ）
- [x] `detect()` / `detect_external()` / `detect_call_patterns()` で severity を設定値から取得する
- [x] `detect_unknown()` — `ImportCategory::Unknown` を `unknown_import` severity で報告
- [x] `--fail-on warning` オプションで warning でも exit code 1 にする
- [x] E2E テストの追加（`tests/e2e_severity.rs`）

### PR 15: `mille report external` コマンド ✅ 完了

- [x] `mille report external` サブコマンドの追加
- [x] 外部ライブラリ依存をレイヤーごとにテーブル形式で出力
- [x] `--format json` / `--output <path>` オプション対応

---

## フェーズ 4: 言語・エコシステム拡張

### PR 16: Java サポート ✅ 完了 (PR #57)

- [x] `infrastructure::parser::java` 実装 (tree-sitter-java)
- [x] `infrastructure::resolver::java` 実装
- [x] Java E2E テスト追加
- [ ] Kotlin サポート（別 PR）

---

## 優先度の考え方（バズりやすい順）

| 順位 | PR | 理由 |
|---|---|---|
| 1 | PR 10 (GitHub Actions) | CI経由のパッシブ口コミ。PRレビュー画面への露出が最大 |
| 2 | PR 11 (`mille init`) | オンボーディング摩擦の除去。「試してみた」投稿が増える |
| 3 | PR 12 (`[ignore]`) | 採用障壁の除去。テストファイル除外は必須 |
| 4 | PR 13 (`mille analyze`) | ビジュアルなデモ素材になる。SNS・ブログ投稿向き |
| 5 | PR 14 (`[severity]`) | 既存プロジェクトへの段階的導入を可能にする |
| 6 | PR 15 (`report external`) | 高度なユーザー向け |
| 7 | PR 16 (Java/Kotlin) | エンタープライズ層への展開 |
