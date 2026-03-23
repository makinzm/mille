# 言語追加時のルール

## E2E fixture と CI dogfooding はセットで1タスク

E2E fixture を書いたら、それを CI で走らせるステップも同時に追加する。分離して考えない。

### 必須成果物（parser/resolver 以外）

1. **CI dogfooding**（最初に意識する）
   - `.github/workflows/ci.yml` の `dogfood-rust` ジョブに追加
   - `mille check` が新 fixture に対して正常終了すること
   - **これは E2E テストとは別物** — CI 上で実際のバイナリが fixture を検証する

2. **E2E テスト**
   - `tests/e2e_<lang>.rs` + `tests/fixtures/<lang>_sample/`
   - `docs/e2e_checklist.md` の全項目をカバー（dep opt-in/out, external opt-in/out, naming）
   - 正常系だけでなく、**意図的に壊したときに失敗するテスト** を含める

3. **Website ドキュメント**
   - `website/src/content/docs/guides/languages/<lang>.md`（ja）
   - `website/src/content/docs/en/guides/languages/<lang>.md`（en）
   - `website/src/content/docs/index.mdx` + `en/index.mdx` のサポート表
   - `website/astro.config.mjs` のサイドバー

4. **README.md フィーチャーマトリックス**
   - 全チェック行に新言語の列を追加

### TODO.md の書き方

言語追加の TODO.md を書くとき、以下の順序でタスクを列挙する:

```markdown
- [ ] CI dogfooding ステップ追加（ci.yml）    ← 最初に書く
- [ ] E2E fixture 作成
- [ ] E2E テスト作成
- [ ] Parser 実装
- [ ] Resolver 実装
- [ ] DispatchingParser / DispatchingResolver 登録
- [ ] Website ドキュメント（ja + en + sidebar + index）
- [ ] README.md フィーチャーマトリックス更新
- [ ] docs/TODO.md 更新
```
