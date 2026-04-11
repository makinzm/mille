# Timeline: Go / npm / Python 脆弱性チェック自動化

## 2026-04-06

### 計画フェーズ
- ユーザーへ確認事項を提示
- 回答受領:
  1. npm: `npm install --package-lock-only` → `npm audit` の方針
  2. Python: dev deps のみ (maturin, pytest) — `packages/pypi/` を対象
  3. Issue 重複チェックは言語ごとに独立

### 調査
- `security-audit.yml` のパターン確認（Issue作成・スケジュール・push/PRトリガー）
- Go: `packages/go/mille/go.mod` に `wazero` と `golang.org/x/sys` が依存
- npm: `packages/npm/package.json` — dependencies なし、`mille.wasm` バイナリ配布形式
- Python: `packages/pypi/pyproject.toml` — dev deps: `maturin>=1.12.6`, `pytest>=8`
- `packages/pypi/uv.lock` が存在 (uv で管理)

### ブランチ作成
- `feat/pr92-vuln-check-go-npm-python` を作成

### 次のステップ
- `dependabot.yml` 実装
- `vulnerability-check.yml` 実装
