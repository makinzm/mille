# PR#79: mille 自身の mille.toml を実態に合わせる

## 背景

- `src/main.rs` は `mille::runner::run_cli()` を呼ぶだけの 1 行ラッパー
- `src/runner.rs` は全レイヤーを import + clap を使う実質的なエントリポイント
- 現状 runner.rs はどのレイヤーにも属しておらず、mille の自己チェックから漏れている

## タスク

- [x] mille.toml を修正: entrypoint（main.rs）と runner（runner.rs）に分離
- [x] entrypoint は runner のみ依存許可（厳格）
- [x] runner は全レイヤー依存 + allow_call_patterns でメソッド制限
- [x] mille.svg + website/src/assets/mille.svg を更新
- [x] `mille check` が通ることを確認
- [x] 全テスト通過確認
