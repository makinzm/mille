# Fix: mille init の3つのバグ修正

## 概要

`mille init` が生成するコンフィグに3つの問題がある。本PRで全て修正する。

## タスクリスト

- [ ] Fix 1: 同名ディレクトリをマージしない (`src/usecase/init.rs`)
- [ ] Fix 2: .venv スキャン除外 (`src/infrastructure/repository/fs_source_file_repository.rs`)
- [ ] Fix 3: Python サブモジュールの external_allow マッチング (`src/domain/service/violation_detector.rs`)
- [ ] RED commit: テストのみ (`--no-verify`)
- [ ] GREEN commit: 実装 (lefthook 通過)
- [ ] REFACTOR: docs/README/TODO 更新
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

(実装後に更新)
