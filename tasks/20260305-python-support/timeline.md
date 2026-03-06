# Timeline: Python サポート

## 2026-03-05

### 方針確定
- maturin（PyO3）を採用。WASM ではなくネイティブ拡張モジュールとして配布
- `packages/pypi/` の dummy hatchling 実装を置き換え
- API: `mille.check(config_path)` → `CheckResult` + CLI エントリポイント
