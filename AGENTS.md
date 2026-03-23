あなたは開発者です。仕様書は `./spec.md`、詳細ルールは `.claude/rules/` を参照。

## 基本原則

- **TDD**: テストを書いてから実装。テストは自動化のみ。Regression を常に意識する
- **コミット順序**: RED（`--no-verify`）→ GREEN → REFACTOR
  - `[test] <内容> because of <理由>` / `[fix] ...` / `[refactor] ...`
- **ブランチ**: `feat/pr<N>-<説明>` を切ってから作業。`main` への直接コミット禁止
- **質問・指摘**: README.md / docs/ に追記（MECE）し、AGENTS.md にプロセスとして記録

## テスト

**テスト実行前に内容（ケース名・fixture 設計・期待結果）をユーザーに提示し承認を得る。AutoApprove でも省略しない。**

**Rust TDD の手順:**
1. RED: スタブ（`todo!()`）＋テストを書いて `--no-verify` でコミット
2. GREEN: 実装して全テスト通過 → lefthook 通過
3. REFACTOR: 必要なら整理

**E2E fixture 設計:**
- テスト対象レイヤーだけ違反が出るよう、他レイヤーは `dependency_mode="opt-out"` / `external_mode="opt-out"` にする
- `external_allow=[]` を安易に使うと serde 等で他レイヤーが誤検知する
- 設定項目の追加・変更時は `docs/e2e_checklist.md` を確認し、**意図的に壊したとき失敗する**テストが揃っているか検証する（正常系のみは無価値）

## 言語追加チェックリスト

新しい言語サポートを追加するとき、以下を **すべて** 完了する。parser/resolver だけでは未完成。

1. **CI dogfooding**: `.github/workflows/ci.yml` の `dogfood-rust` ジョブに新 fixture の `mille check` ステップ追加 ← **最初にやる**
2. **E2E fixture テスト**: `tests/fixtures/<lang>_sample/` + `tests/e2e_<lang>.rs` — `docs/e2e_checklist.md` の全項目をカバー
3. **Website ドキュメント**: `website/src/content/docs/guides/languages/<lang>.md`（ja + en）、`index.mdx` のサポート表更新、`astro.config.mjs` のサイドバー追加
4. **README.md**: フィーチャーマトリックスに言語列追加

TODO.md を書く段階でこの 4 点を明示的にタスクとして含める。

## PR 作成前チェックリスト（順序厳守）

1. `docs/TODO.md` 更新（完了チェック・実装状況サマリー）
2. `README.md` 更新（新機能のリファレンス・使用例）
3. コミット → `gh pr create`

後追い push 禁止。PR 実施順序の変更を指示されたら **即座に** TODO.md を書き換えてコミット。
テストフレームワーク追加時は lefthook と `.github/workflows/` も設定する。

## 実装上の注意

**自クレートインポート（lib + bin 分割）**: `main.rs` は `mille::infrastructure::…` 形式でインポート。`Resolver` に `project.name` を渡し `<crate_name>::` を `Internal` に分類させる。二段階インポート禁止（tree-sitter が外部クレートと誤認する）。

**パブリック API 変更時**: `packages/` 以下の全ラッパーも同コミットまたは直後に更新。`grep -r "変更した関数名"` で全呼び出し箇所を確認してから push。

**allow_call_patterns**: `main` レイヤーにのみ定義可。他レイヤーに書くと設定エラー。

**実装漏れ確認**: PR完成前に spec.md の全フィールドが動作しているか確認。「構造体にフィールドはあるが動作していない」漏れは PR 説明の注意事項に明記し TODO 番号を記録。

## CI/CD

- バージョンは Devbox 経由・マイナーバージョンまで固定（`stable` 等の浮動指定禁止）
- CD 構築時はトークン・権限・取得元 URL を `docs/administrator/` に明記
- コミットメッセージ（title・body）に `[skip ci]` / `[ci skip]` を書かない（CI がスキップされる）。誤った場合は `git commit --allow-empty` で再トリガー
- インラインコメントは `NOTE: なぜ・なければどうなるか・参考リンク（実証済みのみ）` 形式
- CI 修正はプロジェクト設定ファイル（`.cargo/config.toml` 等）を変えず CI 内フラグで完結。修正は最小限
- npm・PyPI publish 前に `cp README.md packages/<pkg>/README.md`（CI で生成、commit しない）
