# feat: TypeScript / JavaScript サポート

## 背景

Python サポート (PR #25) に続いて、TypeScript / JavaScript の import 解析と依存関係チェックを実装する。

Python 実装時の反省を踏まえ、以下を最初から対応する:
- `packages/npm/mille.toml` を最初から作成（dogfood 用）
- `.gitignore` に build artifacts を最初から追加
- E2E チェックリスト全項目をカバー（dep opt-in/opt-out + external opt-in/opt-out）
- README 更新
- CI dogfooding ステップ追加

---

## 実装方針

### 対象ファイル拡張子
- `.ts`, `.tsx`, `.js`, `.jsx`

### import 分類ルール（TypeScriptResolver）
- 相対 import（`./`, `../`）→ **Internal**
- `tsconfig.json` の `paths` / `baseUrl` で解決できるパス → **Internal**（後回し、まず相対のみ）
- それ以外 → **External**

### resolved_path 規則
- `"./domain/user"` → `"domain/user/_.ts"` （layer glob `domain/**` にマッチ）
- `"../domain/user"` → caller のディレクトリから相対解決（簡易版: パスそのまま正規化）

### `[resolve.typescript]` 設定（config.rs の `TsResolveConfig` を活用）
```toml
[resolve.typescript]
tsconfig = "./tsconfig.json"   # 現状は任意（未使用）
```
※ `TsResolveConfig` はすでに `config.rs` に定義済み

---

## タスク一覧

### 準備（Pythonの反省から upfront で対応）
- [x] branch `feat/ts-js-support` 作成
- [ ] `.gitignore` に `node_modules/`, `dist/`, `*.js` (packages/npm build) を追加
- [ ] `tasks/20260306-ts-js-support/timeline.md` 作成

### RED（`--no-verify` でコミット）
- [ ] `tests/fixtures/typescript_sample/` フィクスチャ作成
  - `mille.toml`（domain: opt-in/opt-out, usecase: opt-in/external-opt-in, infrastructure: opt-out/external-opt-out）
  - `domain/user.ts`（外部 import なし）
  - `usecase/user_usecase.ts`（domain import + "some-lib" 外部 import）
  - `infrastructure/db.ts`（domain import + "node:fs" 外部 import）
- [ ] `tests/fixtures/javascript_sample/` フィクスチャ作成（.js + ESM 構文）
  - `mille.toml`（typescript_sample と同じ構成）
  - `domain/user.js`, `usecase/user_usecase.js`, `infrastructure/db.js`
- [ ] `tests/e2e_typescript.rs` 作成（10テスト）:
  - happy path: exits 0, 0 errors
  - dep opt-in broken: usecase.allow=[] → exits 1 (+ mentions "usecase")
  - dep opt-out broken: infrastructure.deny=["domain"] → exits 1 (+ mentions "infrastructure")
  - external opt-in broken: usecase.external_allow=[] → exits 1 (+ mentions "usecase")
  - external opt-out broken: infrastructure.external_deny=["node:fs"] → exits 1 (+ mentions "infrastructure")
- [ ] `tests/e2e_javascript.rs` 作成（同構成、javascript_sample を対象）
- [ ] `src/infrastructure/parser/typescript.rs` スタブ（`todo!()`）
- [ ] `src/infrastructure/resolver/typescript.rs` スタブ（`todo!()`）

### GREEN
- [ ] `Cargo.toml` に `tree-sitter-javascript` + `tree-sitter-typescript` 追加
- [ ] `src/infrastructure/parser/typescript.rs` 実装
  - `import X from "./path"` / `import { X } from "./path"`
  - `require("./path")`
  - `.ts`, `.tsx`, `.js`, `.jsx` 対応
- [ ] `src/infrastructure/resolver/typescript.rs` 実装
  - 相対 import → Internal + resolved_path 計算
  - それ以外 → External
- [ ] `src/infrastructure/parser/mod.rs` に TypeScriptParser を追加（`.ts`, `.tsx`, `.js`, `.jsx`）
- [ ] `src/infrastructure/resolver/mod.rs` に TypeScriptResolver を追加
- [ ] `src/infrastructure/repository/fs_source_file_repository.rs` に `.ts`, `.tsx`, `.js`, `.jsx` 追加
- [ ] `src/main.rs` に TypeScriptResolver の wiring 追加
- [ ] 全テスト GREEN 確認 (`cargo test`)

### dogfooding & ドキュメント
- [ ] `packages/npm/mille.toml` 作成（npm パッケージ自身の check 用）
- [ ] `docs/e2e_checklist.md` に TypeScript 列を追加
- [ ] `README.md` 更新（TypeScript/JavaScript サポート追記）
- [ ] `.github/workflows/ci.yml` に `dogfood-typescript` ジョブ追加
- [ ] `docs/TODO.md` の PR 8.6 (TS/JS サポート) を完了マーク

### PR
- [ ] lefthook 通過確認
- [ ] `gh pr create` で PR 作成

---

## 参考

- Python サポート: PR #25
- `TsResolveConfig` は `src/domain/entity/config.rs` に定義済み
- tree-sitter-javascript: `0.21.4` (tree-sitter 0.22 compatible)
- tree-sitter-typescript: `0.21.2` (tree-sitter 0.22 compatible)
