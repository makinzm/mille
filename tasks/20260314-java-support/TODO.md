# Java サポート追加 (PR #57)

## 目的

`mille` に Java 言語サポートを追加する。`.java` ファイルの import 文を tree-sitter-java で解析し、レイヤー間の依存ルールを検証できるようにする。

## 変更対象

1. `Cargo.toml` - `tree-sitter-java` 依存関係追加
2. `src/infrastructure/parser/java.rs` - Java パーサー新規作成
3. `src/infrastructure/resolver/java.rs` - Java リゾルバー新規作成
4. `src/infrastructure/parser/mod.rs` - DispatchingParser に Java 追加
5. `src/infrastructure/resolver/mod.rs` - DispatchingResolver に Java 追加
6. `src/domain/entity/config.rs` - `JavaResolveConfig` 追加
7. `tests/e2e_java.rs` - E2E テスト追加
8. `tests/fixtures/java_sample/` - Java fixture 作成
9. `docs/TODO.md` - 更新
10. `README.md` - Java 設定リファレンス追加

## テスト計画

### ユニットテスト (parser)
- `test_parse_java_single_import`: 通常 import 文の抽出
- `test_parse_java_static_import`: static import 文の抽出
- `test_parse_java_multiple_imports`: 複数 import の抽出
- `test_parse_java_no_imports`: import なしのソース
- `test_parse_java_call_exprs_empty`: call_exprs は空 Vec を返す

### ユニットテスト (resolver)
- `test_java_internal_is_internal`: module_name で始まる import は Internal
- `test_java_external_is_external`: それ以外は External
- `test_java_stdlib_is_external`: java.util.* 等は External

### E2E テスト
- `test_java_valid_config_exits_zero`: 正常 fixture は exit code 0
- `test_java_broken_usecase_exits_one`: usecase が domain を allow しない場合に違反検出
- `test_java_infra_empty_external_allow_exits_one`: infra が外部ライブラリをインポートして external_allow=[] の場合に違反検出

## fixture 構成 (java_sample)

```
tests/fixtures/java_sample/
├── mille.toml
└── src/
    ├── domain/
    │   └── User.java          (import なし)
    ├── usecase/
    │   └── UserService.java   (import com.example.javasample.domain.User)
    └── infrastructure/
        └── UserRepo.java      (import com.example.javasample.domain.User; import java.util.List)
```

## 完了条件

- [ ] `cargo test` 全テスト通過
- [ ] lefthook 通過
- [ ] E2E テスト通過（正常系 + 異常系の両方）
- [ ] `docs/TODO.md` 更新済み
- [ ] `README.md` Java 設定リファレンス追加済み
