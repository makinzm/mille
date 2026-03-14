# Kotlin サポート実装

## 目的

Java と同じアーキテクチャで Kotlin 言語サポートを追加する。
tree-sitter-kotlin（crates.io: 0.3.8）を使用。
`mille check` と `mille init` の両方で動作すること。

## チェックリスト

### 依存関係
- [ ] `Cargo.toml` に `tree-sitter-kotlin = "0.3"` を追加

### パーサー
- [ ] `src/infrastructure/parser/kotlin.rs` 実装
- [ ] `src/infrastructure/parser/mod.rs` に KotlinParser 登録
- [ ] `src/infrastructure/parser/dispatching.rs`（または相当箇所）で `.kt` → KotlinParser

### リゾルバー
- [ ] `src/infrastructure/resolver/java.rs` の `resolve_for_project` が `.kt` にも対応済み確認
  - 前PR で `is_jvm` / `classify_java_import_for_init` は `.kt` 込み実装済み

### 設定
- [ ] `[resolve.java]` を `.kt` ファイルにも適用（同一リゾルバー）

### フィクスチャ
- [ ] `tests/fixtures/kotlin_sample/` — flat レイアウト
- [ ] `tests/fixtures/kotlin_gradle_sample/` — Gradle レイアウト

### テスト
- [ ] `tests/e2e_kotlin.rs` — E2E テスト（valid/broken/mille init）
- [ ] ユニットテスト in `kotlin.rs`

### CI
- [ ] `.github/workflows/ci.yml` の `dogfood-rust` に Kotlin fixture self-check を追加

### ドキュメント
- [ ] `README.md` に Kotlin サポート追記
- [ ] `docs/TODO.md` 更新

## grammar ノード種別調査メモ

Kotlin の import 文:
```kotlin
import com.example.myapp.domain.User
import com.example.myapp.domain.*
```

tree-sitter-kotlin のノード種別（調査後に記入）:
- import: `import_header`
- パス: `identifier` 子ノード
