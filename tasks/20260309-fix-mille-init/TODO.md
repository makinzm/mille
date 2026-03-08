# Fix: mille init の3つのバグ修正

## 概要

`mille init` が生成するコンフィグに3つの問題がある。本PRで全て修正する。

## タスクリスト

- [x] Fix 1: 同名ディレクトリをマージしない (`src/usecase/init.rs`)
- [x] Fix 2: .venv スキャン除外 (`src/infrastructure/repository/fs_source_file_repository.rs`)
- [x] Fix 3: Python サブモジュールの external_allow マッチング (`src/domain/service/violation_detector.rs`)
- [x] RED commit: テストのみ (`--no-verify`)
- [x] GREEN commit: 実装 (lefthook 通過)
- [x] REFACTOR: docs/README/TODO 更新
- [ ] PR 作成

## 修正詳細

### Fix 1: 同名ディレクトリをマージしない

**問題**: `crawler/src/domain` と `ingest/src/domain` のように異なるサブプロジェクト配下の同名ディレクトリが1つのレイヤーにまとめられる。

**修正**: 同じ base name のディレクトリが複数ある場合、必ず区別する。最初に異なるセグメントを prefix として使う。

### Fix 2: .venv スキャン除外

**問題**: `apps/**` のような glob パターンが `mille check` 時に `.venv` 配下の non-UTF-8 ファイルを読んでエラー。

**修正**: glob 展開後のファイルリストに `.venv` 等を含むパスをフィルタリング。

### Fix 3: Python サブモジュール external_allow マッチング

**問題**: `external_allow = ["matplotlib"]` なのに `matplotlib.pyplot` が violation になる。

**修正**: Python ファイルの場合は `.` 区切りで crate_name を抽出する。

## 実装状況サマリー

- Fix 1: `find_distinguishing_prefix()` 関数を追加し `infer_layers()` の Pass 1 を置き換え。同名ディレクトリが複数ある場合は必ず prefix を付けて区別する。
- Fix 2: `has_excluded_component()` フィルターを `collect()` の全ブランチに追加。`.venv`/`venv`/ドットディレクトリを除外。
- Fix 3: `detect_external()` で `.py` ファイルは `.` 区切り、それ以外は `::` 区切りで crate_name を抽出。
