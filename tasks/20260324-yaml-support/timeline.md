# Timeline: YAML サポート

## 2026-03-24

### Phase 0: セットアップ
- ブランチ `feat/pr76-yaml-support` 作成
- TODO.md, timeline.md 作成

### Phase 1: RED
- E2E テスト 7 件作成 (tests/e2e_yaml.rs)
- fixture 作成 (tests/fixtures/yaml_sample/)
- CI dogfooding ステップ追加 (.github/workflows/ci.yml)
- テスト結果: 4 passed, 3 failed (YAML ファイルが収集されず `0 file(s)`)
- `--no-verify` でコミット

### Phase 2: GREEN
- tree-sitter-yaml 0.6 追加 (Cargo.toml)
- YamlParser 実装 (src/infrastructure/parser/yaml.rs)
  - キー → Symbol, 値 → StringLiteral, コメント → Comment
- DispatchingParser 登録 (mod.rs)
- SOURCE_EXTENSIONS に yaml/yml 追加 (fs_source_file_repository.rs)
- fixture の mille.toml を修正 (name_deny はデフォルトに含めない)
- テスト結果: 7/7 通過、全 397 ユニットテスト + 全 E2E テスト通過

### Phase 3: REFACTOR
- Website ドキュメント作成 (YAML ガイド ja/en, ベストプラクティス ja/en)
- astro.config.mjs サイドバー更新
- index.mdx 言語リスト更新 (ja/en)
- README.md 更新 (Languages, Configuration Reference, 設定例)
- docs/TODO.md 更新
- docs/e2e_checklist.md YAML 注記追加
