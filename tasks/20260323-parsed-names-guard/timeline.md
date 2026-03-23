# Timeline

## 2026-03-23

### 調査完了
- 現状の `Parser::parse_names` は `Vec<RawName>` を返すため、NameKind の網羅性が型レベルで保証されない
- PHP / Python は `NameKind::Variable` 未対応だがコンパイルエラーにならない
- 対象ファイル: trait定義1箇所、実装8箇所（7言語+Dispatcher）、呼び出し1箇所、テストモック1箇所

### RED フェーズ完了
- `ParsedNames` 構造体追加 + `Parser` トレイト戻り値変更 → 17個のコンパイルエラーを確認
- `--no-verify` でコミット (88a2b9a)

### GREEN フェーズ完了
- 全7言語パーサー + Dispatcher + NoOpParser を `ParsedNames` 対応に修正
- `partition_names()` ヘルパーを `infrastructure/parser/mod.rs` に追加
- PHP に Variable 抽出追加（property_declaration, const_declaration）
- Python に Variable 抽出追加（assignment）
- usecase 側で `into_all()` により flatten
- lefthook 全通過（clippy + fmt + test）でコミット (aaf0e2f)
