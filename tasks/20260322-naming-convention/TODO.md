# PR65: Clean Architecture Naming Convention Check

## 目的
`mille` に、レイヤーごとのネーミング規則チェック機能を追加する。
各レイヤーで禁止キーワード (`name_deny`) を設定し、ファイル名・シンボル名・変数名・コメントに禁止キーワードが含まれている場合に `NamingViolation` を報告する。

## config 仕様

```toml
[[layers]]
name = "usecase"
name_deny = ["gcp", "aws", "azure", "mysql", "postgres"]
name_targets = ["file", "symbol", "variable", "comment"]  # 省略時は全対象
```

- `name_deny`: 禁止キーワード（大文字小文字区別なし・部分一致）
- `name_targets`: チェック対象。省略時は全対象
- `severity.naming_violation`: デフォルト `"error"`

## 変更ファイル

- [ ] `src/domain/entity/layer.rs` — `LayerConfig` に `name_deny`, `name_targets` 追加
- [ ] `src/domain/entity/config.rs` — `SeverityConfig` に `naming_violation` 追加
- [ ] `src/domain/entity/violation.rs` — `ViolationKind::NamingViolation` 追加
- [ ] `src/domain/entity/name.rs` — `RawName`, `NameKind` 新規追加
- [ ] `src/domain/entity/mod.rs` — `name` モジュール公開
- [ ] `src/domain/repository/parser.rs` — `parse_names()` メソッド追加
- [ ] `src/infrastructure/parser/rust.rs` — `parse_names()` 実装
- [ ] `src/infrastructure/parser/typescript.rs` — `parse_names()` 実装
- [ ] `src/infrastructure/parser/python.rs` — `parse_names()` 実装
- [ ] `src/infrastructure/parser/go.rs` — `parse_names()` 実装
- [ ] `src/infrastructure/parser/java.rs` — `parse_names()` 実装
- [ ] `src/infrastructure/parser/kotlin.rs` — `parse_names()` 実装
- [ ] `src/domain/service/violation_detector.rs` — `detect_naming()` メソッド追加
- [ ] `src/presentation/formatter/terminal.rs` — `NamingViolation` フォーマット追加
- [ ] `src/presentation/formatter/json.rs` — `NamingViolation` JSON フォーマット追加
- [ ] `src/presentation/formatter/github_actions.rs` — `NamingViolation` GA フォーマット追加
- [ ] `src/usecase/check_architecture.rs` — `detect_naming()` 呼び出し追加
- [ ] `tests/e2e_naming.rs` — E2E テスト
- [ ] `tests/fixtures/naming/` — E2E fixture
- [ ] `docs/TODO.md` — 完了チェック
- [ ] `README.md` — 新設定項目の追記

## TDD サイクル

### サイクル 1: 型定義・config パース
- [x] RED: `NamingViolation` 型定義・config パーステストを追加
- [x] GREEN: 型定義・config パース実装
- [x] REFACTOR: 整理

### サイクル 2: detect_naming() ユニットテスト
- [x] RED: `detect_naming()` ユニットテストを追加
- [x] GREEN: `detect_naming()` 実装
- [x] REFACTOR: 整理

### サイクル 3: パーサー parse_names() ユニットテスト
- [x] RED: 各言語パーサーの `parse_names()` テストを追加
- [x] GREEN: 各言語パーサーの `parse_names()` 実装
- [x] REFACTOR: 整理

### サイクル 4: E2E テスト
- [x] RED: E2E テスト追加
- [x] GREEN: `check_architecture` に `detect_naming()` を組み込み
- [x] REFACTOR: フォーマッター・ドキュメント整備

## 注意事項
- Java/Kotlin の comment node type は実装時に tree-sitter で確認する。未対応の場合は TODO として明記
- website ドキュメント (`website/`) の更新はこの PR では不要
