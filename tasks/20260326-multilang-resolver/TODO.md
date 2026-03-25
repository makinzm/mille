# PR#80: 複数言語 fixture テスト + DispatchingResolver リファクタ

## 目的
- 複数言語（RS + TS + PY）が混在するプロジェクトの E2E テストを追加
- テストで安全網を張ったうえで DispatchingResolver をリファクタ

## Phase 1: 複数言語 fixture テスト

- [ ] `tests/fixtures/multilang_sample/` 作成（RS + TS + PY 混在）
- [ ] `mille.toml` — 3言語のファイルが各レイヤーに混在する構成
- [ ] `tests/e2e_multilang.rs` — E2E テスト作成
  - [ ] happy path: 正常な設定で exit 0
  - [ ] dep opt-in broken: 違反検出で exit 1
  - [ ] dep opt-out broken: 違反検出で exit 1
  - [ ] external opt-in broken: 違反検出で exit 1
  - [ ] external opt-out broken: 違反検出で exit 1
  - [ ] allow_call_patterns broken: 違反検出で exit 1
- [ ] CI dogfooding: ci.yml に multilang fixture の `mille check` ステップ追加

## Phase 2: DispatchingResolver リファクタ

- [ ] Registry パターンへ移行（拡張子 → Resolver のマップ）
- [ ] 必要な言語の Resolver だけ登録する設計
- [ ] 既存テスト全通過確認
- [ ] multilang テスト通過確認

## Phase 3: ドキュメント・仕上げ

- [ ] docs/TODO.md 更新
- [ ] README.md 更新（必要に応じて）
