# PR#80: 複数言語 fixture テスト + DispatchingResolver リファクタ

## 目的
- 複数言語（TS + PY + Go）が混在するプロジェクトの E2E テストを追加
- テストで安全網を張ったうえで DispatchingResolver をリファクタ

## Phase 1: 複数言語 fixture テスト

- [x] `tests/fixtures/multilang_mixed_sample/` 作成（TS/PY/Go レイヤー内混在）
- [x] `tests/fixtures/multilang_split_sample/` 作成（ts/py/go 言語別ディレクトリ分離）
- [x] `tests/e2e_multilang_mixed.rs` — 11 テスト全通過
- [x] `tests/e2e_multilang_split.rs` — 11 テスト全通過
- [x] CI dogfooding: ci.yml に multilang fixture の `mille check` ステップ追加

## Phase 2: DispatchingResolver リファクタ

- [x] Registry パターンへ移行（HashMap<&str, Box<dyn Resolver>>）
- [x] from_resolve_config() に languages 引数追加、必要な言語のみ登録
- [x] 既存テスト全通過確認（397 unit + 全 E2E）
- [x] multilang テスト通過確認

## Phase 3: ドキュメント・仕上げ

- [x] docs/TODO.md 更新
- [x] README.md 更新（不要 — 外部 API 変更なし）
