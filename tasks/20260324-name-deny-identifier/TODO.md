# name_deny: Identifier（属性アクセス）チェック追加

## 背景

`name_deny = ["gcp"]` を設定しても、`cfg.gcp.staging_bucket` のような属性アクセスチェーンの中の `gcp` が検出されない。
現在のパーサーは Symbol（関数/クラス定義名）、Variable（代入ターゲット）、Comment、StringLiteral、File しか抽出しておらず、属性アクセス内の識別子は対象外。

## ゴール

`name_deny` が属性アクセスチェーン内の識別子（`cfg.gcp.staging_bucket` の `gcp`、`staging_bucket` など）もチェックできるようにする。

## タスク

- [x] `NameKind::Identifier` を `name.rs` に追加
- [x] `NameTarget::Identifier` を `layer.rs` に追加（`all()` / `default_name_targets` / `as_name_kind` 更新）
- [x] `ParsedNames` に `identifiers: Vec<RawName>` フィールド追加（`into_all` / `partition_names` 更新）
- [x] `violation_detector.rs` の `detect_naming` に `Identifier` の `target_str` 追加
- [x] Python パーサー: `attribute` ノードの `attribute` フィールド（識別子）を `NameKind::Identifier` として抽出
- [x] 他の全言語パーサー（Rust, TypeScript, Go, Java, Kotlin, PHP, C）にも同等の抽出追加
- [x] docstring が `StringLiteral` として正しく検出されているか検証 → 正常動作確認
- [x] mille.toml の usecase name_allow に "category" 追加（false positive 対策）
- [x] docs/TODO.md 更新
- [x] README.md 更新（name_targets に identifier 追加の旨）

## TDD 順序

1. RED: Python パーサーのユニットテスト（属性アクセスから Identifier 抽出）→ コンパイルエラー ✅
2. RED: violation_detector のユニットテスト（Identifier が name_deny にマッチ）→ コンパイルエラー ✅
3. GREEN: NameKind / NameTarget / ParsedNames / パーサー / detector 実装 ✅
4. GREEN: 全言語パーサー対応 ✅
5. REFACTOR: ドキュメント更新 ✅
