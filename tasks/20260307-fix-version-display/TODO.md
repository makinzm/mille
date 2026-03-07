# Fix: version display が常に 0.0.1 になる問題

## 背景

`go install github.com/makinzm/mille/packages/go/mille@v0.0.9` でインストール後、
`mille --version` が `mille 0.0.1` と表示される。

## 根本原因

- `Cargo.toml` の `version = "0.0.1"` が clap にコンパイル時に埋め込まれる
- `release.yml` の `build`/`build-deb` ジョブはビルド前に `Cargo.toml` のバージョンを更新しない
- WASM バイナリも `0.0.1` で埋め込まれるため、Go/npm ラッパー経由でも同じ問題が発生

## 対策

### 1. Go ラッパーで `--version` をインターセプト
- `runtime/debug.ReadBuildInfo()` でモジュールバージョンを取得
- WASM に渡す前に `--version`/`-V` を処理

### 2. npm ラッパーで `--version` をインターセプト
- `package.json` からバージョンを読み取り出力

### 3. `release.yml` の `build`/`build-deb` ジョブを修正
- ビルド前に `Cargo.toml` のバージョンを `sed` で更新
- ネイティブバイナリ成果物も正しいバージョンを表示

## チェックリスト

- [ ] Go ラッパーのテスト追加（RED）
- [ ] Go ラッパーの実装（GREEN）
- [ ] npm ラッパーの修正
- [ ] `release.yml` の修正
- [ ] timeline.md 更新
