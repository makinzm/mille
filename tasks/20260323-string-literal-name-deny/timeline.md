# Timeline: StringLiteral name_deny サポート

## 2026-03-23

### 背景調査
- `name_deny` が文字列リテラルをチェックしていないことを発見
- `usecase/init.rs` に `"go" => Some("go")` 等の言語名リテラルがあるが `name_deny` を素通り
- 原因: `NameKind` に `StringLiteral` がなく、パーサーも文字列リテラルを抽出していない
- ユーザーと方針合意: デフォルト ON（opt-out 方式）、全8言語対応

### Phase 1: Domain 型追加 開始

## 2026-03-24

### Phase 1: RED コミット (d061721)
- `NameKind::StringLiteral` 追加
- `NameTarget::StringLiteral` 追加 + `all()` に含める
- `ParsedNames` に `string_literals` フィールド追加 + `into_all()` に含める
- `detect_naming` で `StringLiteral → "string_literal"` 表示追加
- コンパイルエラー確認: `ParsedNames` のコンパイルタイムガードが全パーサーでエラーを出す

### Phase 2: テスト RED コミット (f7828c8)
- `test_detect_naming_string_literal_violation`: StringLiteral が name_deny でマッチ → NamingViolation
- `test_detect_naming_target_filter_excludes_string_literal`: name_targets に含めないとスキップ

### Phase 3 + 4: GREEN コミット
- `partition_names` に `StringLiteral` 分岐追加
- `strip_string_delimiters` ヘルパー追加（クォート/バッククォート/トリプルクォート/raw string 対応）
- 全8パーサー（Rust/Go/Python/TypeScript/Java/Kotlin/PHP/C）に文字列リテラル抽出追加
- mille.toml の domain/usecase で `name_targets` から `string_literal` を除外（自己dogfood対応）
- clippy collapsible_if 修正
- 全テスト通過（366 unit + 32 E2E）

### Phase 5: ドキュメント更新
- README.md: `name_targets` に `string_literal` 追加、対応言語に C 追加
- Website (ja/en): name_targets テーブル・対応言語テーブル更新、PHP/C 追加
- docs/TODO.md: 実装状況サマリー更新
