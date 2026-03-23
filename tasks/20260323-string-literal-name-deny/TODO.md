# StringLiteral name_deny サポート

## 背景
`name_deny` は現在 File / Symbol / Variable / Comment のみチェックする。
文字列リテラル内に禁止キーワードが含まれていても素通りしてしまう。
mille の哲学として「リテラルだから見逃す」は許容しない。

## タスク

### Phase 1: Domain 型追加 (RED — コンパイルエラーをわざと出す)
- [x] `NameKind::StringLiteral` 追加 (`name.rs`)
- [x] `NameTarget::StringLiteral` 追加 + `all()` に含める (`layer.rs`)
- [x] `ParsedNames` に `string_literals: Vec<RawName>` フィールド追加 (`name.rs`)
- [x] `into_all()` に `string_literals` を含める
- [x] `violation_detector.rs` の `detect_naming` で StringLiteral → "string_literal" 表示追加

### Phase 2: テスト追加 (RED)
- [x] `violation_detector.rs` に StringLiteral の name_deny テスト追加
- [x] `name_targets` で StringLiteral をオプトアウトできるテスト追加

### Phase 3: 全言語パーサーで string literal 抽出実装 (GREEN)
- [x] Rust parser
- [x] Go parser
- [x] Python parser
- [x] TypeScript parser
- [x] Java parser
- [x] Kotlin parser
- [x] PHP parser
- [x] C parser

### Phase 4: mille 自身の mille.toml 更新
- [x] usecase / domain の `name_targets` から `string_literal` を除外

### Phase 5: ドキュメント・仕上げ
- [x] docs/TODO.md 更新
- [x] README.md 更新
- [x] Website ドキュメント更新（ja + en）
