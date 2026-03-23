# StringLiteral name_deny サポート

## 背景
`name_deny` は現在 File / Symbol / Variable / Comment のみチェックする。
文字列リテラル内に禁止キーワードが含まれていても素通りしてしまう。
mille の哲学として「リテラルだから見逃す」は許容しない。

## タスク

### Phase 1: Domain 型追加 (RED — コンパイルエラーをわざと出す)
- [ ] `NameKind::StringLiteral` 追加 (`name.rs`)
- [ ] `NameTarget::StringLiteral` 追加 + `all()` に含める (`layer.rs`)
- [ ] `ParsedNames` に `string_literals: Vec<RawName>` フィールド追加 (`name.rs`)
- [ ] `into_all()` に `string_literals` を含める
- [ ] `violation_detector.rs` の `detect_naming` で StringLiteral → "string_literal" 表示追加

### Phase 2: テスト追加 (RED)
- [ ] `violation_detector.rs` に StringLiteral の name_deny テスト追加
- [ ] `name_targets` で StringLiteral をオプトアウトできるテスト追加

### Phase 3: 全言語パーサーで string literal 抽出実装 (GREEN)
- [ ] Rust parser
- [ ] Go parser
- [ ] Python parser
- [ ] TypeScript parser
- [ ] Java parser
- [ ] Kotlin parser
- [ ] PHP parser
- [ ] C parser

### Phase 4: mille 自身の mille.toml 更新
- [ ] usecase / domain の `name_targets` または `name_allow` を調整

### Phase 5: ドキュメント・仕上げ
- [ ] docs/TODO.md 更新
- [ ] README.md 更新
- [ ] spec.md 更新（該当箇所があれば）
