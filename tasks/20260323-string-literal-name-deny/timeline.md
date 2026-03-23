# Timeline: StringLiteral name_deny サポート

## 2026-03-23

### 背景調査
- `name_deny` が文字列リテラルをチェックしていないことを発見
- `usecase/init.rs` に `"go" => Some("go")` 等の言語名リテラルがあるが `name_deny` を素通り
- 原因: `NameKind` に `StringLiteral` がなく、パーサーも文字列リテラルを抽出していない
- ユーザーと方針合意: デフォルト ON（opt-out 方式）、全8言語対応

### Phase 1: Domain 型追加 開始
