# `mille add --depth` 実装

## 概要
`mille add <target_path> --depth N` でサブディレクトリを個別レイヤーとして複数追加。
例: `mille add conf --depth 1` → `cloud`, `competition`, `executor` 等を別レイヤーに。

## 設計

### CLI
```
mille add conf --depth 1     # conf/cloud, conf/competition 等を個別レイヤーに
mille add conf --depth 2     # conf/competition/inference 等をレイヤーに
mille add conf               # 従来通り conf を1レイヤーとして追加（depth なし）
```

### 動作フロー（--depth あり）
1. target_path 配下のソースファイルを持つディレクトリを収集
2. depth 相対でレイヤーディレクトリを決定（`ancestor_at_depth` を活用）
3. `infer_layers` でレイヤー構造を推論
4. 各レイヤーを重複チェック → 追記/置換

## タスク

- [ ] CLI args テスト（`--depth` オプション追加）
- [ ] CLI args 実装
- [ ] Runner: depth ありのスキャン・複数レイヤー追加ロジック
- [ ] E2E テスト
- [ ] README 更新
