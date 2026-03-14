# Timeline — Java サポート追加

## 2026-03-14

### 計画フェーズ
- 既存の Go パーサー/リゾルバーを参照して実装方針を決定
- `tree-sitter-java` クレートを使用（他言語と同系列）
- fixture 構成と E2E テストケースを設計

### GREEN フェーズ (完了)
- `src/infrastructure/parser/java.rs`: tree-sitter-java を使った実装（通常 import + static import 対応）
- `src/infrastructure/resolver/java.rs`: module_name prefix で Internal/External を分類
- `src/infrastructure/repository/fs_source_file_repository.rs`: `SOURCE_EXTENSIONS` に `"java"` 追加
- `src/usecase/init.rs`: `ext_to_language` に `"java"` 追加
- `src/domain/entity/config.rs`: `JavaResolveConfig` 追加
- `cargo test`: 全テスト通過 (255 unit + 7 e2e_java + 他 E2E すべて)

```
test result: ok. 255 passed (lib)
test result: ok. 7 passed (e2e_java)
```

### RED フェーズ (完了)
- スタブ実装 (`todo!()`) とテストを作成
- `cargo test` で Java 関連 10 テストが期待通り失敗: `not yet implemented`
  - `infrastructure::parser::java::tests::test_parse_java_*` (4 テスト)
  - `infrastructure::resolver::java::tests::test_java_*` (6 テスト)
- 既存 245 テストは全て通過（デグレなし）
- `--no-verify` でコミット実施

エラーログ:
```
failures:
    infrastructure::parser::java::tests::test_parse_java_multiple_imports
    infrastructure::parser::java::tests::test_parse_java_no_imports
    infrastructure::parser::java::tests::test_parse_java_single_import
    infrastructure::parser::java::tests::test_parse_java_static_import
    infrastructure::resolver::java::tests::test_java_external_is_external
    infrastructure::resolver::java::tests::test_java_internal_is_internal
    infrastructure::resolver::java::tests::test_java_resolver_external_resolve
    infrastructure::resolver::java::tests::test_java_resolver_ignores_own_crate_param
    infrastructure::resolver::java::tests::test_java_resolver_internal_resolve
    infrastructure::resolver::java::tests::test_java_stdlib_is_external
test result: FAILED. 245 passed; 10 failed
```

