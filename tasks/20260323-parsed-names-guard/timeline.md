# Timeline

## 2026-03-23

### 調査完了
- 現状の `Parser::parse_names` は `Vec<RawName>` を返すため、NameKind の網羅性が型レベルで保証されない
- PHP / Python は `NameKind::Variable` 未対応だがコンパイルエラーにならない
- 対象ファイル: trait定義1箇所、実装8箇所（7言語+Dispatcher）、呼び出し1箇所、テストモック1箇所

### RED フェーズ開始
- `ParsedNames` 構造体追加 + `Parser` トレイト戻り値変更 → コンパイルエラーを確認
