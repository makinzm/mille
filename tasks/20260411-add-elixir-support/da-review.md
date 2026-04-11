## DA レビュー - Round 1

**レビュー対象**: feat/pr94-add-elixir-support (4 commits, main..HEAD)
**レビュー日時**: 2026-04-11

### 指摘事項

#### [重要度: 高] `parse_elixir_names` が関数名に引数リストを含めてしまう
- **該当箇所**: `src/infrastructure/parser/elixir.rs:45-74`
- **問題**: `def new(id, name, email) do ... end` を解析すると、Symbol 名として `"new"` ではなく `"new(id, name, email)"` が抽出される。`defmodule MyApp.Domain.User do ... end` も同様に `"MyApp.Domain.User"` 全体が Symbol として push される。`defp helper(x)` → `"helper(x)"`、`defmacro foo(x, y)` → `"foo(x, y)"` 等、全 def/defp/defmacro/defmacrop 系で同じ不具合が発生する。
- **検証**: 実際に `parse_elixir_names` を fixture 上で動かして確認済み:
  ```
  tests/fixtures/elixir_sample/lib/domain/user.ex
  === SYMBOLS (3) ===
    line 1: "MyApp.Domain.User"
    line 4: "new(id, name, email)"
    line 8: "create(attrs)"
  ```
- **理由**: tree-sitter-elixir の AST では、`def new(id, name, email)` は
  ```
  call (identifier "def") (arguments
    call (identifier "new") (arguments (...)))   ← first_arg は call ノード
  ```
  という構造で、`args.child(0)` (first_arg) は単なる `identifier` ではなく内部の `call` ノードになっている。現在の実装は `first_arg.utf8_text` を無条件に name として push しているため、ソーステキスト全体（`new(id, name, email)`）が抽出される。Python パーサー（`python.rs:51-63`）では `node.child_by_field_name("name")` で関数名ノードのみを取得しており、他言語と一貫性がない。
- **影響**:
  1. `name_deny` の部分一致では問題が表面化しにくいが、報告/JSON 出力・GitHub Actions アノテーションに `"new(id, name, email)"` のようなノイズが載る。
  2. 厳密一致や将来の exact-match チェック追加時に確実に壊れる。
  3. 他言語（Python/Rust 等）の Symbol 抽出との整合性が失われる。
  4. エラーメッセージが Elixir 開発者にとって理解しづらい。
- **提案**:
  1. `def` / `defp` / `defmacro` / `defmacrop` の場合、`first_arg` が `call` ノードなら更に `child(0)` の `identifier` を取って関数名のみを抽出する。
  2. `defmodule` の場合、`alias` ノードの末尾セグメント（`User`）を抽出するか、仕様として dotted path を保持するかをドキュメントで明示する（現状は暗黙の判断）。
  3. **ユニットテストを追加**: `test_parse_names_def_function`, `test_parse_names_defmodule`, `test_parse_names_defp_helper` 等を `elixir.rs` の `#[cfg(test)] mod tests` に追加し、関数名だけが抽出されることを保証する。現状 `parse_elixir_names` に対するユニットテストは **0 件** で、この不具合は見落とされていた。

#### [重要度: 高] `parse_elixir_names` のユニットテスト不在
- **該当箇所**: `src/infrastructure/parser/elixir.rs:200-262`
- **問題**: `tests` モジュール内には `parse_elixir_imports` のテスト (7 件) しかなく、`parse_elixir_names` / `parse_elixir_call_exprs` のテストが 1 件もない。
- **理由**: Parser trait の 3 メソッドのうち 1 つしかテストしていない。CLAUDE.md の「テストファースト」「TDD は全対象に対して」原則に反する。上記の高重要度バグ（関数名に引数リストが混入）もユニットテストがあれば事前に検出できたはず。
- **提案**: 最低限、以下のテストを追加:
  - `test_parse_names_defmodule_extracts_module_name`
  - `test_parse_names_def_extracts_function_name_only`
  - `test_parse_names_defp_extracts_private_function_name`
  - `test_parse_names_comment_extracts_content`
  - `test_parse_names_string_literal_extracts_content`

#### [重要度: 中] E2E テスト: `external_mode = "opt-in"` のネガティブケース欠落
- **該当箇所**: `tests/e2e_elixir.rs`, `docs/e2e_checklist.md`
- **問題**: `docs/e2e_checklist.md` の必須項目「external opt-in: `external_allow = []` → 外部違反」に対応するテストが存在しない。現状のテストは以下のみ:
  - dep opt-in (`allow=[]` usecase): ✅ カバー
  - dep opt-out (`deny=["domain"]` infrastructure): ✅ カバー
  - external opt-out (`external_deny=["Ecto"]` infrastructure): ✅ カバー
  - **external opt-in: ❌ 欠落**
- **理由**: `.claude/rules/testing.md` および `docs/e2e_checklist.md` が「必ず全項目をカバーする」ことを要求している。`infrastructure/repo.ex` が `alias Ecto.Repo` を import しているので、infrastructure に `external_mode="opt-in", external_allow=[]` を設定すれば違反が出るはず。
- **提案**: `test_elixir_broken_external_opt_in_exits_one` を追加し、`external_allow = []` で Ecto が違反として検出されることを確認する。

#### [重要度: 中] `docs/TODO.md` の実装状況サマリー未更新
- **該当箇所**: `docs/TODO.md:14` 付近
- **問題**: `.claude/rules/pr-workflow.md` と CLAUDE.md は PR 作成前に「`docs/TODO.md` 更新（完了チェック・実装状況サマリー）」を必須としている。実装状況サマリーに Rust/Go/TypeScript/JavaScript/Python/Java/Kotlin/PHP が列挙されている行があるが、Elixir が追加されていない。C/YAML と同じく独立した完了エントリとして Elixir サポートを明記すべき。
- **理由**: TODO.md は開発者/レビュワー/外部利用者が「いま何がサポートされているか」を確認する一次情報源。`docs/TODO.md:14` の行や PR #57/#69/#71/#76 のような成果エントリに Elixir が無いと、履歴から見て「Elixir は未完成」と誤認される。
- **提案**: コミット `[refactor]` に `docs/TODO.md` の更新を追加する:
  - `14 行目: Rust / Go / ... / PHP / Elixir サポート` へ変更
  - 新しい行として `✅ Elixir 言語サポート — .ex/.exs ファイルの alias/import/require/use パース、...（PR #xx）` を追加

#### [重要度: 中] Website ドキュメントが「allow_call_patterns 非対応」を明記していない
- **該当箇所**: `website/src/content/docs/guides/languages/elixir.md`, `website/src/content/docs/en/guides/languages/elixir.md`
- **問題**: `parse_elixir_call_exprs` は常に空リストを返すため、Elixir では `allow_call_patterns` が機能しない。実装コメント (`elixir.rs:135`) は「Elixir's dynamic dispatch makes static call analysis unreliable」と理由を記載しているが、ユーザー向けドキュメントには一切書かれていない。
- **理由**: 利用者が `mille.toml` に `[[layers.allow_call_patterns]]` を書いても沈黙で無視されるのは最悪の UX。必ず明記すべき。
- **提案**: elixir.md に「制限事項」セクションを追加し、`allow_call_patterns` が Elixir では未サポートである旨と理由を記載する。

#### [重要度: 低] `parse_elixir_call_exprs` で不要なパース実行
- **該当箇所**: `src/infrastructure/parser/elixir.rs:136-144`
- **問題**: 空 `Vec` を返すだけなのに tree-sitter パーサーを起動して `_tree` を捨てている。不要な CPU 消費。
- **提案**: 単に `Vec::new()` を返すだけで良い。パースが必要ないことを明示的なコメントで示す。
  ```rust
  pub fn parse_elixir_call_exprs(_source: &str, _file_path: &str) -> Vec<RawCallExpr> {
      // Elixir's dynamic dispatch makes static call analysis unreliable — return empty.
      Vec::new()
  }
  ```

#### [重要度: 低] E2E テストの broken config が親ディレクトリを汚染する可能性
- **該当箇所**: `tests/e2e_elixir.rs:80-128, 130-176, 182-230, 237-286`
- **問題**: 各テストが `elixir_fixture_dir().join("mille_broken_*.toml")` に config を書き込み、最後に `remove_file(...).ok()` で削除している。しかし、テストが panic/assert 失敗すると `remove_file` が実行されず、ゴミファイルが残る。また、並列実行時に同名ファイル衝突のリスクはないが（各テストで別名を使用）、fixture ディレクトリ自体を変更するのはお行儀が悪い。
- **理由**: 他言語の e2e テスト（例: `e2e_python.rs` など）がどうしているか確認して統一すべき。理想は `tempfile::TempDir` を使うか、fixture 側に broken config を事前に配置する。
- **提案**:
  1. `scopeguard` 等で必ず cleanup する、または
  2. `tempfile::tempdir()` でテスト専用の作業ディレクトリを作成し、そこに fixture をコピーしてから config を書く、または
  3. broken config を `tests/fixtures/elixir_sample_broken_usecase/` のようなディレクトリに事前に配置する（最もシンプル）。

#### [重要度: 低] `tree-sitter-elixir` のバージョン選定理由が未記録
- **該当箇所**: `Cargo.toml`（新規追加 `tree-sitter-elixir = "0.2.0"`）, `tasks/20260411-add-elixir-support/timeline.md:30`
- **問題**: timeline.md では「`0.3.5` 追加」と書かれているのに、実際に追加されているのは `0.2.0`。なぜバージョンを落としたのか記録がない。CLAUDE.md「バージョンは Devbox 経由・マイナーバージョンまで固定」の文言との整合確認も必要。
- **提案**: timeline.md または PR 説明に「0.3.5 は tree-sitter core の互換性問題で動かなかったため 0.2.0 を採用」等、選定理由を追記する。

---

### 良い点

1. **コミット順序が TDD に準拠している**: `[test] ...` → `[fix] ...` → `[refactor] ...` の流れが正しく、メッセージも `because of ...` 形式に従っている。`--no-verify` での RED コミット確認も timeline.md に明記されている。
2. **CI dogfooding を最初に追加**: `.claude/rules/new-language.md` の通り、CI ステップ (`ci.yml dogfood-rust` ジョブ) が E2E fixture と同時に整備されている。
3. **E2E fixture のレイヤー設計が testing.md に従っている**: テスト対象以外のレイヤーは `opt-out + []` に設定されており、`external_allow=[]` による誤検知を避けている。
4. **Website ドキュメント (ja+en)**: ja/en 両方・サイドバー・index.mdx のサポート表まで更新されており、ドキュメント整備は充実している。
5. **parser 7 件・resolver 11 件のユニットテスト**: `parse_elixir_imports` と `classify_elixir`/`elixir_module_to_path` のテストは網羅性が高く、`alias ... as:`（alias rename）や空 app_name の edge case までカバーしている。
6. **リゾルバーの実装がシンプルで読みやすい**: `classify_elixir` / `elixir_module_to_path` の関数分離と docstring の例示が良い。
7. **README の Configuration Reference も更新**: `pr-workflow.md` の要求「README 更新」が実施されている。

---

### 総評

PR 全体の「骨格」（ブランチ運用・コミット順序・CI dogfooding・ドキュメント整備・リゾルバー実装）は高品質で、Elixir サポート追加としての枠組みは整っている。しかし **`parse_elixir_names` に明確な実装バグがあり、そのバグがユニットテスト不在のため見逃されている** 点が critical。加えて `external_mode="opt-in"` の E2E カバレッジ不足と `docs/TODO.md` 未更新により、CLAUDE.md のプロセス原則に部分的に違反している。

Parser の naming 抽出バグは partial match ベースの `name_deny` では表面化しにくいが、将来 exact match やレポート出力が増えると必ず顕在化する。また他言語との挙動不整合は保守負債になる。いま直すのが最小コスト。

### 判定

- [ ] LGTM（問題なし、マージ可能）
- [x] 要修正（指摘対応後、再レビュー）
- [ ] 要相談（人間の判断が必要）

**必須対応（高）**:
1. `parse_elixir_names` の関数名抽出バグを修正し、ユニットテストを追加する。
2. `parse_elixir_names` のユニットテスト（defmodule/def/defp/comment/string_literal）を追加する。

**推奨対応（中）**:
3. `test_elixir_broken_external_opt_in_exits_one` を追加する。
4. `docs/TODO.md` の実装状況サマリーを更新する。
5. Website ドキュメントに「`allow_call_patterns` 非対応」を明記する。

**任意対応（低）**:
6. `parse_elixir_call_exprs` の不要なパース削除。
7. E2E テストの broken config 配置方法の改善。
8. `tree-sitter-elixir` バージョン選定理由の記録。
