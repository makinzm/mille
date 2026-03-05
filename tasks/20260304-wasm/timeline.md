# Timeline: Wasm/WASI リファクタリング

## 2026-03-05

### 設計調査・計画

- 既存コードベースを調査:
  - `src/main.rs`: `clap` CLI → WASI "command" モジュールとして自然にマッピング可能
  - `src/usecase/check_architecture.rs`: `std::fs::read_to_string` を使用 → WASI preopen で透過解決
  - `packages/go/main.go`: GitHub Releases からバイナリをダウンロードするブートストラップ方式
  - `Cargo.toml`: `tree-sitter`（C ライブラリ）が最大のクロスコンパイル障壁
- wasi-sdk-30 が最新であることを確認（計画では v24 を想定していたが更新）
- **設計決定**: WASI "Command" モジュール方式 → Rust コア無変更

---

### [RED] テスト先行コミット

（ここに RED フェーズのエラーログを記録する）

---

### [GREEN] 実装コミット

（ここに GREEN フェーズの結果を記録する）

---

### [REFACTOR] リファクタリングコミット

（必要であれば記録する）
