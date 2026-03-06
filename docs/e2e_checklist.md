# E2E テスト網羅性チェックリスト

> **参照タイミング**: 新しい言語サポートを追加したとき・フィクスチャを変更したとき・設定項目の実装を変更したとき

---

## 原則

- **1 テスト = 1 設定項目の違反**: 他の設定は全て正常にし、当該設定だけを壊す
- **正常系だけでは不十分**: 意図的に壊したときにエラーが出なければ、そのテストはカバレッジとして無価値
- **言語ごとに独立してカバー**: Go と Rust（など）でそれぞれテストを持つ

---

## 設定項目別チェックリスト

### 1. `dependency_mode = "opt-in"` + `allow`

各レイヤーで `allow` から実際に参照しているレイヤーを除いたとき、依存違反が出るか確認する。

| 確認ポイント | テスト内容 |
|---|---|
| usecase が domain を参照できなくなる | `usecase.allow = []` → usecase → domain 違反 |
| infrastructure が domain を参照できなくなる | `infrastructure.allow = []` → infrastructure → domain 違反 |
| cmd/main が下位レイヤーを参照できなくなる | `cmd.allow` から一部除く → cmd → 除いたレイヤー 違反 |

### 2. `dependency_mode = "opt-out"` + `deny`

| 確認ポイント | テスト内容 |
|---|---|
| opt-out レイヤーが実際に参照しているレイヤーを deny したとき違反が出る | `deny = ["<実際に参照しているレイヤー>"]` → 違反検出 |

### 3. `external_mode = "opt-in"` + `external_allow`

各レイヤーで実際に使用している外部パッケージを `external_allow` から外したとき、外部違反が出るか確認する。

**Go の注意**: Go では標準ライブラリ（`fmt`, `os`, `database/sql` 等）も `External` として分類されるため、`external_allow` で明示的に許可が必要。

| 確認ポイント | テスト内容 |
|---|---|
| `external_allow = []` で実際に外部パッケージを使用 | infrastructure/usecase 等 → 該当パッケージが違反として検出される |
| `external_allow` に一部しか列挙していない | `cmd.external_allow = ["fmt"]` (os を除く) → `os` が違反として検出される |

### 4. `external_mode = "opt-out"` + `external_deny`

| 確認ポイント | テスト内容 |
|---|---|
| `external_deny` に実際に使用しているパッケージを設定したとき違反が出る | `external_deny = ["<実際に使用しているパッケージ>"]` → 違反検出 |

### 5. `allow_call_patterns`

`main` レイヤーにのみ定義可能。

| 確認ポイント | テスト内容 |
|---|---|
| `allow_methods` にないメソッドを呼び出したとき違反が出る | `allow_methods = ["new"]` で `.find_user()` 等を呼び出す → CallPatternViolation |

---

## フィクスチャ一覧（現状）

| フィクスチャ | テストファイル | 言語 |
|---|---|---|
| `tests/fixtures/go_sample/` | `tests/e2e_go.rs` | Go |
| mille 自身 (`src/`) | `tests/e2e_check.rs` | Rust |
| `tests/fixtures/python_sample/` | `tests/e2e_python.rs` | Python |
| `tests/fixtures/typescript_sample/` | `tests/e2e_typescript.rs` | TypeScript |
| `tests/fixtures/javascript_sample/` | `tests/e2e_javascript.rs` | JavaScript |

---

## 実装状況

| チェックポイント | Go | Rust | Python | TypeScript | JavaScript |
|---|---|---|---|---|---|
| 1. dep opt-in: usecase.allow=[] | ✅ | — (自己のアーキテクチャが正しいため違反なし) | ✅ `test_python_broken_usecase_exits_one` | ✅ `test_ts_broken_usecase_allow_exits_one` | ✅ `test_js_broken_usecase_allow_exits_one` |
| 1. dep opt-in: infrastructure.allow=[] | ✅ | ✅ `INFRA_BLOCKS_DOMAIN_TOML` | N/A (infrastructure は opt-out) | N/A (infrastructure は opt-out) | N/A (infrastructure は opt-out) |
| 1. dep opt-in: cmd/main.allow から一部除く | ✅ | ✅ `MAIN_FORBIDS_INFRA_TOML` | N/A (fixture に main レイヤーなし) | N/A | N/A |
| 2. dep opt-out: deny=["参照しているレイヤー"] | ☐ | ☐ | ✅ `test_python_broken_infra_deny_domain_exits_one` | ✅ `test_ts_broken_infra_deny_exits_one` | ✅ `test_js_broken_infra_deny_exits_one` |
| 3. external opt-in: external_allow=[] | ✅ | ✅ | N/A (fixture は opt-out) | ✅ `test_ts_broken_external_optin_exits_one` | ✅ `test_js_broken_external_optin_exits_one` |
| 3. external opt-in: cmd.external_allow=[] | ✅ | ☐ | N/A | N/A | N/A |
| 3. external opt-in: 部分的な external_allow | ✅ | N/A | N/A | N/A | N/A |
| 4. external opt-out: external_deny=["使用pkg"] | ☐ | ☐ | ✅ `test_python_broken_external_deny_os_exits_one` | ✅ `test_ts_broken_external_optout_exits_one` | ✅ `test_js_broken_external_optout_exits_one` |
| 5. allow_call_patterns: 禁止メソッドの呼び出し | N/A (未実装) | ✅ `CALL_PATTERN_VIOLATION_TOML` | N/A (Python は未実装) | N/A (未実装) | N/A (未実装) |

---

## 未実装項目（将来対応）

- `allow_call_patterns` の Go / Python / TypeScript / JavaScript E2E テスト（パーサーでの呼び出しチェック未実装）
- `[severity]` の設定項目テスト（未実装）
- `[ignore]` の paths / test_patterns テスト（未実装）
- `[resolve.aliases]` テスト（未実装）
- Go の `dependency_mode = "opt-out"` + `deny` 違反テスト（fixture にそのパターンがない）
- Go / Rust の `external_mode = "opt-out"` + `external_deny` 違反テスト（fixture に該当する外部パッケージ使用例がない）
