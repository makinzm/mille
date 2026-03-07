# テストに関するルール

## テスト実行前にユーザーへ確認する

テスト内容（何を、どのように検証するか）をユーザーに示し、承認を得てから実行する。
AutoApprove モードであっても省略しない。

**確認すべき内容:**
- テストケース名と検証する振る舞いの一覧
- 使用する fixture の設計（どんな状態が前提か）
- 期待する成功/失敗のパターン

**NG:** テストを黙って書いて実行する
**OK:** 「以下のテストを追加する予定です。よろしければ続けます。」→ 確認 → 実装・実行

---

## E2E fixture 設計原則

### 「テスト対象のレイヤーだけ違反する」設計にする

E2E テストで「あるレイヤーの設定を壊したとき違反が出る」ことを確認するとき、
**他のレイヤーが誤って違反を出さないよう** にする。

**推奨パターン:**
- テスト対象以外のレイヤーは `dependency_mode = "opt-out"` + `deny = []` にする
- テスト対象以外のレイヤーは `external_mode = "opt-out"` + `external_deny = []` にする

**NG の例:**
```toml
# domain に external_allow = [] を設定すると
# domain ファイルが serde を import しているため ExternalViolation が発生し
# infrastructure 以外のレイヤーも違反を出してしまう
[[layers]]
name = "domain"
external_mode = "opt-in"
external_allow = []   # ← serde を使っている domain ファイルで誤検知
```

**OK の例:**
```toml
[[layers]]
name = "domain"
dependency_mode = "opt-out"
deny = []
external_mode = "opt-out"   # ← 何でも OK にする
external_deny = []
```

### RED フェーズと GREEN フェーズを分ける

RED commit: テストを書く（スタブ実装 or `todo!()` で失敗する状態にする）
GREEN commit: テストを通す
REFACTOR commit: 整理する

**NG:** テストと実装を同じコミットに含める
**OK:** テストだけ先にコミット（`--no-verify`）→ 実装コミット → lefthook 通過確認

---

## fixture が誤検知していないか確認する

テストが失敗したとき、まず以下を確認する:

1. 期待通りのレイヤーだけが違反を出しているか
2. 他のレイヤーが想定外の理由で違反を出していないか（external_allow=[] の誤設定など）
3. テスト名と実際の検証内容が一致しているか
