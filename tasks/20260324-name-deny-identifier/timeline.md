# Timeline

## 2026-03-24

### 調査フェーズ

- ユーザーから報告: `name_deny = ["gcp"]` が `cfg.gcp.staging_bucket`（属性アクセス）を検出しない
- 同様に docstring 内の `cfg.gcp.project` も通過している
- Python パーサー (`python.rs`) を調査: `collect_python_names` は `function_definition`, `class_definition`, `assignment`, `comment`, `string` のみ処理
- `attribute` ノード（ドットアクセス）は処理対象外 → `gcp` が抽出されない原因
- docstring は tree-sitter 上では `string` ノードなので理論的には `StringLiteral` として抽出されるはず → 要検証

### RED フェーズ開始
