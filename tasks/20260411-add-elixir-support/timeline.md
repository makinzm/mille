# タイムライン: Elixir 言語サポート追加

## 2026-04-11

### 計画・調査フェーズ
- 既存パーサー（Python）・リゾルバー・DispatchingParser/Resolver を調査
- E2Eテストパターン（e2e_python.rs）を確認
- CI dogfooding ステップパターンを ci.yml で確認
- Website ドキュメント構造を確認（ja/en 両方存在）
- task ディレクトリ作成、TODO.md・timeline.md 作成

### 次のステップ
1. CI dogfooding ステップを ci.yml に追加
2. E2E fixture 作成
3. E2E テストスケルトン作成（--no-verify でコミット）
4. 実装フェーズへ
