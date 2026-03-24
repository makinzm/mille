# PR#75: 位置引数でターゲットディレクトリを指定可能にする

## 概要

`mille check [PATH]` のように位置引数でプロジェクトディレクトリを指定できるようにする。
デフォルトは `.`（現行動作と完全互換）。

## タスク

- [ ] args.rs: `CommonArgs` 構造体（`path` フィールド）を作成し、全コマンドに `#[command(flatten)]` で埋め込み
- [ ] args.rs: `Command::common()` メソッド（exhaustive match）でコンパイル時保証
- [ ] runner.rs: `common().path` で CWD 変更 + config パス解決
- [ ] args.rs の unit test 追加
- [ ] E2E テスト追加（別ディレクトリからの check 実行）
- [ ] README.md 更新
- [ ] docs/TODO.md 更新
