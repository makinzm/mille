# PR ワークフローに関するルール

## PR 作成前の必須チェックリスト

PR を作成する前に以下を必ず完了させる。順序も守る。

1. **docs/TODO.md を更新する**
   - 実装した PR のチェックボックスを ✅ に変更
   - 実装状況サマリーに追加
   - ❌ の項目から削除（README に掲載しないよう管理されているもの）

2. **README.md を更新する**
   - 新しい設定項目や CLI オプションを Configuration Reference に追記
   - 使用例を追加する

3. **AGENTS.md を更新する**（指摘・質問があった場合）
   - やり取りで発生した質問・指摘を汎用的なルールとして追記

4. **変更内容を 1 コミットにまとめてから `gh pr create` する**

**NG:** PR を作ってから後追いで README や TODO を更新する
**OK:** README 更新 → TODO 更新 → commit → `gh pr create`

---

## 機能実装後の UX 確認

新しい出力フォーマットや CLI オプションを追加したとき、以下を確認する:

- **空出力にならないか?** ユーザーが「何も起きていない」と混乱する状態を避ける
  - 例: `--format github-actions` で違反なしのとき → `::notice::` で確認メッセージを出す
- **`--help` でオプションが確認できるか?** `mille help <subcommand>` で説明が表示されるか確認する

---

## コミット順序（再確認）

```
[test]    ... because of ...   # --no-verify で先にコミット
[fix]     ... because of ...   # lefthook が通ることを確認してコミット
[refactor] ... because of ...  # ドキュメント整理など
```

ドキュメント更新（README, TODO.md）は `[refactor]` コミットに含める。
