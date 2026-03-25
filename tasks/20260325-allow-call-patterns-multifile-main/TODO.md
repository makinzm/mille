# PR#78: allow_call_patterns 全レイヤー開放 + multi-file main fixture

## 背景

1. `allow_call_patterns` が spec/docs で「mainレイヤーのみ」と記載されているが、コード上は制約なし → ドキュメントを実態に合わせて全レイヤー開放を明示
2. 現状の fixture は main レイヤーがディレクトリ (`main/**`) or 単一ファイル (`src/main.rs`) のみ。現実には `src/main.rs`（薄い）+ `src/runner.rs`（実質エントリ）のようなパターンが多い → fixture とテストを追加
3. 失敗ケースの CLI レベル Large Test が不足 → 違反検出を検証するテストを追加

## タスク

### ドキュメント修正（allow_call_patterns 全レイヤー開放）
- [ ] spec.md: main限定の記述を削除、全レイヤーで使える旨に変更
- [ ] website/src/content/docs/configuration/layers.mdx（ja）: 同上
- [ ] website/src/content/docs/en/configuration/layers.mdx（en）: 同上
- [ ] AGENTS.md: main限定の記述を削除

### E2E テスト: allow_call_patterns を main 以外で使うケース
- [ ] テスト設計をユーザーに提示・承認
- [ ] テスト作成（RED: --no-verify）
- [ ] 実装確認（GREEN: 既にコードは対応済みなのでテストが通るはず）

### multi-file main fixture + 失敗ケース Large Test
- [ ] `tests/fixtures/rust_multifile_main/` 作成（main.rs + runner.rs パターン）
- [ ] テスト設計をユーザーに提示・承認
- [ ] 正常系テスト: 両ファイルが main レイヤーとして認識される
- [ ] 失敗テスト: runner.rs が禁止レイヤーを import → 違反検出
- [ ] 失敗テスト: allow_call_patterns が runner.rs にも適用される

### 仕上げ
- [ ] CI dogfooding: 必要があれば ci.yml に追加
- [ ] docs/TODO.md 更新
- [ ] README.md 更新（allow_call_patterns の説明修正）
- [ ] 全テスト通過確認
