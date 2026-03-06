# PyPI パッケージセットアップガイド (`packages/pypi/`)

## 概要

`packages/pypi/` は **maturin (PyO3)** を使った Rust → Python ネイティブ拡張です。
純粋な Python パッケージではなく、Rust でコンパイルされた `.so` を含むバイナリホイール (`*.whl`) として配布します。

---

## ローカル開発手順

```bash
cd packages/pypi

# 依存インストール（.venv 作成 + maturin も含む）
uv sync --dev

# Rust コンパイル → mille/mille.cpython-XXX.so 生成
uv run maturin develop

# テスト実行
uv run pytest tests/ -v
```

`maturin develop` が成功すると `mille/mille.cpython-312-x86_64-linux-gnu.so` が生成されます。
この `.so` は `.gitignore` で除外されているため、コミットしないでください。

---

## ビルド成果物と gitignore

| ファイル | 説明 | git 管理 |
|---|---|---|
| `mille/__init__.py` | `.so` からの再エクスポート | **管理する** |
| `mille/mille.cpython-*.so` | Rust コンパイル成果物 | **除外 (.gitignore)** |
| `mille/__pycache__/` | Python バイトコードキャッシュ | 除外 |
| `Cargo.toml` | クレート設定 | **管理する** |
| `Cargo.lock` | 依存関係ロック | **管理する** |
| `uv.lock` | Python 依存関係ロック | **管理する** |
| `.venv/` | 仮想環境 | 除外 |

---

## モジュール構造

```
packages/pypi/
├── Cargo.toml          # maturin クレート設定 (crate-type = ["cdylib"])
├── pyproject.toml      # build-backend = "maturin"
├── uv.lock             # Python 依存関係ロック
├── src/
│   └── lib.rs          # PyO3 バインディング (#[pymodule] fn mille)
└── mille/
    ├── __init__.py     # from .mille import check, _main, ...
    └── mille.cpython-*.so  # (gitignore) maturin develop で生成
```

`mille/__init__.py` が native extension (`mille.mille`) の公開シンボルを再エクスポートすることで、
`import mille; mille.check(...)` が動作します。

---

## CI での Python dogfooding

`.github/workflows/ci.yml` の `dogfood-python` ジョブが以下を実行します:

1. `uv sync --dev` — 依存インストール
2. `uv run maturin develop` — Rust コンパイル
3. `uv run pytest tests/ -v` — Python テスト
4. `uv run mille check ../../mille.toml` — mille 自身を検査
5. `uv run mille check` in `tests/fixtures/python_sample` — Python fixture の検査

---

## PyPI へのリリース

リリースは `.github/workflows/release.yml` の `publish-pypi` ジョブが担当します。
必要なシークレット: `PYPI_TOKEN`（詳細は `docs/administrator/cd_setup.md` を参照）

maturin はクロスプラットフォームのホイールビルドに対応しています:

```yaml
# release.yml の publish-pypi ジョブ例
- uses: PyO3/maturin-action@v1
  with:
    command: publish
    args: --manifest-path packages/pypi/Cargo.toml
  env:
    MATURIN_PYPI_TOKEN: ${{ secrets.PYPI_TOKEN }}
```

> **注意:** maturin でビルドしたホイールはプラットフォーム固有 (`linux_x86_64`, `macos_arm64` 等)。
> 複数プラットフォーム対応には `maturin-action` の `target` マトリクスが必要。

---

## トラブルシューティング

### `module 'mille' has no attribute 'check'`

`mille/__init__.py` が存在しない、または `.so` がコンパイルされていない可能性があります。

```bash
uv run maturin develop  # 再コンパイル
```

### `maturin` コマンドが見つからない

`uv sync --dev` で maturin がインストールされます。
`uv run maturin develop` として実行してください（`maturin develop` 単体では動かないことがあります）。
