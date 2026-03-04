あなたは、開発者です。

テスト駆動開発をしてください。
また、テストをする唯一の方法は自動化だと把握してください。
加えて、自信を持って変更をするためにテストを書いていることに留意をしてRegressionテストを意識してください。

---

仕様書は ./spec.md に記載されています。

---

なにかテストの枠組みを追加する場合は、
lefthook及び .github/workflows/ を適当に設定して、CI/CDを構築してください。

---

やり取りをする中で質問された内容は返信をするだけではなく、README.mdやdocs/に追記してください。また、MECEを意識して、適当な粒度で分割して追記してください。

---

commitの粒度は小さくし、メッセージに対してなぜその変更をしたのかを明確にしてください。

そして、commitの順序は必ず次の順番で行い、
RED -> GREEN -> REFACTORの順序を守ってください。

- `[test] <テスト内容> because of そのテストが必要な理由`
- `[fix] <修正内容> because of <なぜその修正が妥当なのか>`
- `[refactor] <リファクタリング内容> because of <なぜそのリファクタリングが妥当なのか>`

ただ、１つ目のcommitについてはlefthookなどは必ず失敗してしまうため、`--no-verify` をつけてコミットしてください。

---

あなたは、私とのやり取りの中で、必ずなにか指摘されたり質問されたりします。

その内容はプロセスとして二度と繰り返さないように、汎用的な内容にして AGENTS.md を更新してください。

---

## 開発ワークフロー上のルール

### TDD の進め方（Rust での具体的な手順）

Rust はコンパイルが通らないとテストが走らないため、RED フェーズは「コンパイルは通るがテストが失敗する」状態にします。

1. **RED commit（`--no-verify`）**: エンティティ定義とスタブ関数（`todo!()`）を書き、テストケースを追加する。`cargo test` は `todo!()` でパニックする。
2. **GREEN commit**: スタブを実装し、全テストをパスさせる。lefthook が通ることを確認してコミット。
3. **REFACTOR commit**: 必要であればリファクタリング。

> ⚠️ 実装をテストと同一コミットに含めると RED フェーズがスキップされる。ルール上はスタブ → テスト → 実装の順序を守ること。

### TODO.md の更新タイミング

PR に含める実装が完了したら、**同じブランチ内**で `docs/TODO.md` の該当チェックボックスを更新してコミットに含める。マージ後に別途更新しない。

### 実装漏れの確認（spec との整合性）

PR を完成させる前に、spec.md や `LayerConfig` などの既存エンティティに定義済みのフィールド・機能が**すべて実装されているか**確認する。

今回の例: `LayerConfig.allow_call_patterns` は定義済みだが `ViolationDetector` でチェックされていなかった。
→ このような「データ構造はあるが動作していない」漏れは PR 説明の **注意事項** セクションに明記し、対応する TODO 番号を記録する。

### Dogfooding の E2E テスト設計原則

ツール自身のコードを検査する「dogfooding」テストは、**ハッピーパス（正常系）だけでなく、意図的にエラーになる設定でも動作確認を行う**。具体的には：

1. **正常系**: 正しい `mille.toml` を使ったとき、違反が 0 件であることを確認する。
2. **異常系（レイヤー設定を壊す）**: 各レイヤーの `allow` / `deny` を意図的に誤りにして、期待通りの違反が検出されることを確認する。
   - 例: `main` レイヤーの `allow` から `infrastructure` を除いたとき → `src/main.rs` の `infrastructure` インポートが違反として検出される。
3. **レイヤーごとのバリエーション**: `domain` のみ、`usecase` のみ、`main` のみなど、層ごとに独立したテストケースを用意する。

> ⚠️ 正常系のみでは「ツールが実際に機能しているか」を確認できない。意図的に壊したときにエラーが出なければ、そのツールはテストとして無価値。

---

### 自クレートインポートの分類（lib + bin 分割時の注意）

Rust で `src/lib.rs` と `src/main.rs` が共存するプロジェクトでは、`main.rs` はライブラリクレートを `<crate_name>::` プレフィックス付きで参照する（例: `use mille::infrastructure::…`）。

**問題**: インポート分類器が `crate::` しか「内部」として認識しない場合、`mille::infrastructure::…` は「外部クレート」として扱われ、**依存関係違反が検出されない**。

**対策**: Resolver（または分類器）には、プロジェクトの自クレート名（`mille.toml` の `project.name`）を渡し、`<crate_name>::` で始まるパスも `ImportCategory::Internal` として分類する。

実装パターン:
- `Resolver` トレイトに `resolve_for_project(&self, import, own_crate)` を追加（デフォルト実装は `resolve()` に委譲）
- `RustResolver` でオーバーライドして `<own_crate>::` を `crate::` と同等に扱う
- `check_architecture::check()` 内で `config.project.name` を `resolve_for_project` に渡す

**また、`main.rs` は二段階インポート（`use mille::infrastructure; use infrastructure::…`）を避け、完全修飾パス（`use mille::infrastructure::parser::…`）を使用する**。二段階インポートでは tree-sitter が `infrastructure::…` を外部クレートと誤認する。

---

## 開発環境・CI/CD構築時のルール

1. **特定の環境に依存させない（Devboxの利用）**
   - 言語のバージョン指定やツールチェイン（`uv`, `volta`, `rustup`, `go` など）の導入は、可能な限り `devbox` を経由して行います。
   - `rust-toolchain.toml` などにおいて `stable` のような暗黙の浮動バージョンは避け、マイナーバージョンまで固定（例: `1.85.0`）することで再現性を高めてください。

2. **CI/CD周りの設定とドキュメント化**
   - CI/CDパイプライン（とくにパッケージのPublishなどを伴うCD）を構築する際は、必ず **実行に必要なトークンや権限・取得元のURL** を `docs/administrator/` 以下のドキュメント（例: `cd_setup.md`）に明記してください。
   - `lefthook` などのpre-commitフックを設定する場合は、LintやFormatだけでなく、テスト（例: `cargo test`）も含めて Regression テストを意識した構成にしてください。
