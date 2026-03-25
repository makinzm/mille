# Timeline: PR#78

## 2026-03-25

### 調査フェーズ
- allow_call_patterns のmain限定制約を調査 → spec/docs にのみ記載、コードには制約なし
- E2E テスト構造を調査 → 失敗ケースは存在するが inline TOML パターン、multi-file main の fixture なし
- ブランチ `feat/pr78-allow-call-patterns-all-layers-and-multifile-main` 作成
- TODO.md 作成

### テストフェーズ (RED→GREEN)
- `tests/fixtures/rust_multifile_main/` fixture 作成（main.rs + runner.rs パターン）
- `tests/e2e_multifile_main.rs` に 8 テスト作成
  - 正常系: clean exits zero, both files in main layer
  - 失敗系 (dep): runner.rs が infrastructure を import → 違反検出
  - 失敗系 (call pattern on main): runner.rs の禁止メソッド呼び出し検出
  - 失敗系 (call pattern on non-main): usecase の禁止メソッド呼び出し検出
- 初回: TempConfig が fixture の mille.toml を上書きする方式 → 並列テストで競合、リストア失敗のリスク
- 修正: fixture dir に一意な名前の temp TOML を書いて --config で指定する方式に変更
- greet.rs が struct literal を使っていたため call pattern 検出されず → User::new() に変更
- 全 8 テスト通過

### ドキュメント・CI フェーズ
- spec.md, website docs (ja/en), AGENTS.md から main 限定記述を削除
- README.md の allow_call_patterns 説明を更新
- ci.yml に rust_multifile_main の dogfooding ステップ追加
- docs/TODO.md 更新
- 全テスト通過確認（397 unit + 207 E2E）
