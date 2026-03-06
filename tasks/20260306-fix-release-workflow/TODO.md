# fix: release workflow の二重起動と PyPI manylinux 対応

## 背景

v0.0.5 リリース時に2つの問題が発覚。

---

## 問題 1: release workflow が二重起動する

### 原因
`on: push: tags:` と `on: release: published` の両方を設定しているため、
GitHub UI からリリースを作成するとタグ作成による `push` イベントと
リリース公開による `release: published` イベントが同時に発火する。

### 対応
- [ ] `on: push: tags:` を削除し `on: release: published` のみに統一する

---

## 問題 2: PyPI へのアップロードが `linux_x86_64` タグで失敗する

### 原因
`python -m build` で生成した wheel は `linux_x86_64` タグを持つが、
PyPI は移植性のない plain linux wheel を拒否する。
PyPI が受け付けるのは `manylinux_*` タグの wheel のみ。

`packages/pypi` は maturin プロジェクトのため、`PyO3/maturin-action` を
使って manylinux Docker コンテナ内でビルドする必要がある。

### 対応
- [ ] `publish-pypi` ジョブを `PyO3/maturin-action@v1` の `publish` コマンドに置き換える
- [ ] `python -m build` + `twine upload` の手順を削除する
- [ ] バージョン同期を maturin-action の引数で処理する

---

## 参考

- [maturin-action](https://github.com/PyO3/maturin-action)
- PyPI エラー: `Binary wheel 'mille-0.0.5-cp310-cp310-linux_x86_64.whl' has an unsupported platform tag 'linux_x86_64'`
- 失敗した run: https://github.com/makinzm/mille/actions/runs/22747045073
