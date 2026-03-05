# PR N+2: E2E テスト網羅性 + external_allow 修正

## 背景

- `tests/fixtures/go_sample/mille.toml` で `external_mode = "opt-in"` / `external_allow = []` にしているのに
  `mille check` が violation を検出しない（外部 import チェックが機能していないバグ）
- `packages/go/mille.toml` の `external_allow` も雑で機能していない
- E2E テストが「正常系 + usecase の dependency 違反（Go のみ）」だけで、
  他の設定項目を壊したときの検証がない
- ドキュメントは書いても読まれなければ意味がない → AGENTS.md に参照を追記する

---

## 作業手順

1. **本 TODO + AGENTS.md 更新**（チェックリストを読まれる状態にする）
2. **既存テストの網羅性確認**（Rust + Go、両方）
3. **不足テストの追加**（RED commit）
4. **バグ修正 / 実装修正**（GREEN commit）
5. **リファクタリング**（必要に応じて）

---

## チェックリスト

### 0. ドキュメント整備

- [x] `tasks/20260305-external-allow-e2e/TODO.md` 作成（本ファイル）
- [x] `AGENTS.md` に「E2E テスト追加・変更時は本チェックリストを参照すること」を追記

---

### 1. E2E テスト網羅性チェックリスト

**原則**: 1 テスト = 1 設定項目の違反。他の設定は全て正常にし、当該設定だけを壊す。

対象フィクスチャ:
- Go: `tests/fixtures/go_sample/` + `tests/e2e_go.rs`
- Rust: mille 自身 + `tests/e2e_check.rs`

#### 1-1. `dependency_mode = "opt-in"` + `allow`

| ケース | 壊す設定 | 期待する違反 | Go | Rust |
|--------|----------|-------------|-----|------|
| usecase が domain を参照できない | `usecase.allow = []` | usecase → domain 違反 | ✅ 既存 | [ ] |
| infrastructure が domain を参照できない | `infrastructure.allow = []` | infrastructure → domain 違反 | ✅ 追加済 | ✅ 既存 (INFRA_BLOCKS_DOMAIN_TOML) |
| cmd/main が下位レイヤーを参照できない | `cmd.allow` から一部除く | cmd → 除いたレイヤー 違反 | ✅ 追加済 | ✅ 既存 (MAIN_FORBIDS_INFRA_TOML) |

#### 1-2. `dependency_mode = "opt-out"` + `deny`

| ケース | 壊す設定 | 期待する違反 | Go | Rust |
|--------|----------|-------------|-----|------|
| opt-out レイヤーが deny したレイヤーを参照 | `deny = ["<実際に参照しているレイヤー>"]` | 違反検出 | [ ] | [ ] |

#### 1-3. `external_mode = "opt-in"` + `external_allow`

| ケース | 壊す設定 | 期待する違反 | Go | Rust |
|--------|----------|-------------|-----|------|
| external_allow=[] なのに外部 pkg を使用 | `infrastructure.external_allow = []` | external 違反 | ✅ 追加済 | ✅ 追加済 |
| external_allow=[] なのに外部 pkg を使用 | `cmd.external_allow = []` | external 違反 | ✅ 追加済 | [ ] |
| 許可リストにない外部 pkg を使用 | `cmd.external_allow = ["fmt"]` (os を除く) | os が external 違反 | ✅ 追加済 | N/A |

#### 1-4. `external_mode = "opt-out"` + `external_deny`

| ケース | 壊す設定 | 期待する違反 | Go | Rust |
|--------|----------|-------------|-----|------|
| opt-out で特定の外部 pkg を禁止 | `external_deny = ["<実際に使用している pkg>"]` | external 違反 | [ ] | [ ] |

---

### 2. バグ修正: external_mode = opt-in が機能していない

- [x] 実装コード（`classify_go()`）で Go stdlib が `Stdlib` として扱われ external チェックをスキップしていることを確認
- [x] `classify_go()` を修正: Go では全非内部 import を `External` として扱う
- [x] `packages/go/mille.toml` の `paths` と `external_allow` を修正

---

## 実施しない項目（将来対応）

- `allow_call_patterns` の Go E2E テスト（Go パーサーでの呼び出しチェック未実装）
- `[severity]` の設定項目テスト（未実装）
- `[ignore]` の paths / test_patterns テスト（未実装）
- `[resolve.aliases]` テスト（未実装）
- Python / TypeScript フィクスチャの E2E テスト網羅性（各言語サポート追加時に対応）
- `dependency_mode = "opt-out"` + `deny` 違反テスト（Go・Rust ともに fixture がそのパターンに対応していない）
- `external_mode = "opt-out"` + `external_deny` 違反テスト（fixture に外部 pkg の deny 設定例がない）
