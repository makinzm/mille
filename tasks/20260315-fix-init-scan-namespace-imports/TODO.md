# TODO: Fix mille init scan logic for src/ namespace layout

## 問題

`mille init` が `src/` レイアウトのプロジェクトで全レイヤーを "(no internal dependencies)" と表示する。

### 根本原因

`classify_py_import` が `from src.domain.entity import X` を処理する際、
ドットパスの先頭セグメント (`"src"`) だけを `TryInternal` に渡している。
`resolve_to_known_dir("src", ...)` はレイヤーディレクトリ (`src/domain`, `src/infrastructure` 等) と
一致しないため、`external_pkgs` にフォールバックされ、内部依存エッジが記録されない。

## 修正方針

1. `classify_py_import` を変更してフルドットパスを返す (先頭セグメントだけでなく)
2. `resolve_to_known_dir` を変更してドットパスのすべてのプレフィックスを試す
   (ドットをスラッシュに変換してレイヤーディレクトリと照合)

### 修正後のフロー例

- インポート: `from src.domain.entity import X` (in `src/infrastructure/analyzer.py`)
- `classify_py_import("src.domain.entity")` → `TryInternal("src.domain.entity")`
- `resolve_to_known_dir("src.domain.entity", "src/infrastructure", layer_dirs)` が試すプレフィックス:
  - `"src"` → `"src"` (レイヤーなし)
  - `"src.domain"` → `"src/domain"` (一致!) → `internal_deps += "src/domain"`

## タスク

- [ ] feat/pr63-fix-init-scan-namespace-imports ブランチ作成
- [ ] tasks/TODO.md, timeline.md 作成
- [ ] RED: テスト3件を追加して失敗を確認 (`--no-verify` コミット)
- [ ] GREEN: `classify_py_import` と `resolve_to_known_dir` を修正してテスト通過
- [ ] REFACTOR: docs/TODO.md, README.md 更新
- [ ] PR 作成

## テストケース

### Test 1: `test_classify_py_import_returns_full_path`
- `classify_py_import("src.domain.entity")` → `TryInternal("src.domain.entity")`
- `classify_py_import("domain.entity")` → `TryInternal("domain.entity")`

### Test 2: `test_resolve_to_known_dir_namespace_prefix`
- layer_dirs: `["src/domain", "src/infrastructure"]`
- `resolve_to_known_dir("src.domain.entity", "src/infrastructure", ...)` → `Some("src/domain")`
- `resolve_to_known_dir("src.infrastructure.db", "src/domain", ...)` → `Some("src/infrastructure")`
- `resolve_to_known_dir("src.unknown.thing", "src/domain", ...)` → `None`

### Test 3: `test_scan_detects_src_namespace_internal_deps` (統合テスト)
- 一時ディレクトリに `src/domain/` と `src/infrastructure/` を作成
- `src/infrastructure/analyzer.py` が `from src.domain.entity import X` をインポート
- 期待: `src/infrastructure` の `DirAnalysis.internal_deps` に `"src/domain"` が含まれる
