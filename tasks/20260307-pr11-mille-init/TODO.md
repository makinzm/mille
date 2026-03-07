# PR 11: `mille init` コマンド実装

## 概要

`mille init` サブコマンドを追加。カレントディレクトリを再帰スキャンしてレイヤーを推論し、`mille.toml` を自動生成する。

## タスク

- [x] `tasks/20260307-pr11-mille-init/TODO.md` 作成（このファイル）
- [x] `tasks/20260307-pr11-mille-init/timeline.md` 作成
- [x] `src/usecase/init.rs` — コアロジック（スキャン・生成）
- [x] `src/usecase/mod.rs` — `pub mod init;` 追加
- [x] `src/presentation/cli/args.rs` — `Command::Init { output, force }` 追加
- [x] `src/main.rs` — `Command::Init` ハンドラ追加
- [x] `tests/e2e_init.rs` — E2E テスト
- [x] RED コミット（`--no-verify`）
- [x] GREEN コミット（lefthook 通過）
- [x] `docs/TODO.md` 更新
- [x] `README.md` 更新
- [ ] `gh pr create`

## コマンド仕様

```
mille init [--output <path>] [--force]
```

| フラグ | デフォルト | 説明 |
|---|---|---|
| `--output` | `./mille.toml` | 出力先パス |
| `--force` | false | 既存ファイルを確認なしで上書き |

## 動作フロー

1. カレントディレクトリを再帰スキャン（深さ3まで）→ レイヤー候補と言語を検出
2. 検出結果を stdout に表示
3. `--output` 先が既存ならエラー終了（`--force` 時は上書き）
4. `mille.toml` を書き出し、成功メッセージを表示
