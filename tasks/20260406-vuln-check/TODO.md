# TODO: Go / npm / Python 脆弱性チェック自動化

## 目的

Go (govulncheck)、npm (npm audit)、Python (pip-audit) の依存脆弱性を定期スキャンし、
脆弱性発見時に GitHub Issue を自動作成する。
cargo の脆弱性チェックは既存の `security-audit.yml` でカバー済みのため対象外。

## 対象ファイル

- `.github/dependabot.yml` — 新規作成（Go / npm / pip のバージョン自動更新PR）
- `.github/workflows/vulnerability-check.yml` — 新規作成（Go / npm / Python の脆弱性スキャン）

## タスク一覧

- [x] ブランチ作成 (`feat/pr92-vuln-check-go-npm-python`)
- [x] TODO.md / timeline.md 作成
- [ ] `.github/dependabot.yml` 実装
- [ ] `.github/workflows/vulnerability-check.yml` 実装
- [ ] DAレビュー
- [ ] docs/TODO.md / README.md 更新要否確認
- [ ] PR 作成

## 仕様決定事項

1. `npm install --package-lock-only` で lock ファイルを生成してから `npm audit` 実行
2. Python は dev deps (maturin, pytest) のみ対象 — `packages/pypi/` ディレクトリを対象
3. Issue 重複チェックは言語ごとに独立:
   - `[Security] Go ...`
   - `[Security] npm ...`
   - `[Security] Python ...`

## Issue 自動作成条件

- スケジュール実行 (schedule) で脆弱性発見 → Issue 作成
- push / PR 時に脆弱性発見 → ワークフロー fail のみ（Issue ノイズ回避）
