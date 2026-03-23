# ParsedNames 構造体によるコンパイルタイムガード

## 目的
`Parser::parse_names` の戻り値を `Vec<RawName>` から `ParsedNames` 構造体に変更し、
新しい NameKind 追加時に全言語パーサーでコンパイルエラーが発生するようにする。

## タスク

- [ ] `ParsedNames` 構造体を domain/entity/name.rs に追加
- [ ] `Parser` トレイトの `parse_names` 戻り値を `ParsedNames` に変更
- [ ] RED: コンパイルエラーを確認し `--no-verify` でコミット
- [ ] GREEN: 全パーサー（7言語 + Dispatcher + NoOpParser）を修正
- [ ] PHP / Python に Variable 抽出を追加
- [ ] usecase 側で ParsedNames → Vec<RawName> の flatten を実装
- [ ] 全テスト通過を確認
- [ ] REFACTOR: 必要に応じて整理
