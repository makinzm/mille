# Python サポート（maturin/PyO3）

## 背景

Go ラッパーは WASM/wazero を使っているが、Python は maturin（PyO3）を使いネイティブ
拡張モジュールとして配布する。`pip install mille` で platform wheel がインストールされ、
ネイティブ速度で `mille.check()` が呼べる。

## 方針

- `packages/pypi/` の dummy hatchling 実装を maturin ベースに置き換え
- `mille` Python モジュールとして PyO3 拡張を公開
- ライブラリ API: `mille.check(config_path)` → `CheckResult`
- CLI 経由: `mille check [--config path]`（`pip install` 後に使える）

## チェックリスト

- [x] tasks/20260305-python-support/TODO.md 作成
- [ ] RED: Python テスト追加（拡張未ビルドで失敗）
- [ ] GREEN: packages/pypi/ に Cargo.toml + src/lib.rs 実装
- [ ] devbox.json に maturin 追加
- [ ] CI に Python ビルド・テストステップ追加
- [ ] docs/TODO.md の PR 8.5 を更新
