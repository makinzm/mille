# mille 仕様書

> Architecture Checker — Rust製・多言語対応アーキテクチャlinter

---

## 目次

1. [概要](#1-概要)
2. [チェック機能](#2-チェック機能)
3. [opt-in / opt-out モデル](#3-opt-in--opt-out-モデル)
4. [import解決の設計](#4-import解決の設計)
5. [設定ファイル（mille.toml）仕様](#5-設定ファイルmilletoml仕様)
6. [CLIインターフェース](#6-cliインターフェース)
7. [出力フォーマット](#7-出力フォーマット)
8. [既存ツールとの比較](#8-既存ツールとの比較)

---

## 1. 概要

### プロジェクト概要

`mille`は、クリーンアーキテクチャ・オニオンアーキテクチャ・ヘキサゴナルアーキテクチャなど、**レイヤードアーキテクチャの依存ルールを静的解析するCLIツール**。

Rustで実装し、複数言語のコードベースに対応する。TOMLの設定ファイルでルールを定義し、CI/CDに組み込むことができる。

### 設計方針

- **言語非依存なコアエンジン** — レイヤー・ルール判定ロジックは言語を問わず共通
- **tree-sitterによる構文解析** — 正規表現ではなくASTベースでimport文を抽出することで精度を担保
- **opt-in / opt-out モデル** — レイヤーの性質に合わせた依存ルールの記述方式。内部依存・外部ライブラリ依存の両方に同一モデルを適用
- **段階的導入** — まず可視化・分析し、次にルール適用するワークフローをサポート
- **mille自身もクリーンアーキテクチャで実装** — dogfooding

---

## 2. チェック機能

### 2-1. 内部レイヤー依存チェック（コア機能）

レイヤーごとに `dependency_mode` を設定し、内部レイヤー間の依存違反を検出する。
`paths` に定義されたファイルがどのレイヤーに属するかを判定し、許可されていないレイヤーへのimportをエラーとする。
opt-in / opt-outモデルの詳細は [セクション3](#3-opt-in--opt-out-モデル) を参照。

### 2-2. 外部ライブラリ依存チェック

レイヤーごとに `external_mode` を設定し、外部ライブラリへの依存違反を検出する。
内部レイヤー依存と同じopt-in / opt-outモデルで記述する。

### 2-3. メソッド呼び出しチェック

DI組み立て以外の目的でinfrastructureのメソッドを直接呼び出すことを禁止する。

`allow_call_patterns` は他のレイヤーに記述した場合は設定エラーとなる。

```toml
[[layers]]
name            = "main"
paths           = ["src/main.rs"]
dependency_mode = "opt-in"
allow           = ["infrastructure", "usecase", "presentation"]

# callee_layerのメソッドのうち、allow_methodsに列挙したものだけ呼び出せる。設定がない場合は呼び出しをすべて許可する。
[[layers.allow_call_patterns]]
callee_layer  = "infrastructure"
allow_methods = ["new", "build", "create", "init", "setup"]
```

これにより以下のような違反を検出できる。

```rust
// OK: infrastructureのインスタンス生成（allow_methodsに該当）
let repo = UserRepositoryImpl::new();
let usecase = UserUsecase::new(repo);
usecase.execute();

// NG: infrastructureのビジネスロジックを直接呼び出し
repo.find_user(1);  // ❌ allow_methodsに該当しない
repo.save(&user);   // ❌ allow_methodsに該当しない
```

---

## 3. opt-in / opt-out モデル

内部レイヤー間の依存（`dependency_mode`）と外部ライブラリへの依存（`external_mode`）の両方に、同一のモデルを適用する。

| モード | デフォルト | 書くもの | 向いているレイヤー |
|---|---|---|---|
| `opt-in` | 全てNG | 許可するものを `allow` / `external_allow` に列挙 | domain, usecase, presentation |
| `opt-out` | 全てOK | 禁止するものを `deny` / `external_deny` に列挙 | infrastructure |

### 内部依存（dependency_mode）

```toml
[[layers]]
name            = "domain"
dependency_mode = "opt-in"
allow           = []               # 何にも依存しない

[[layers]]
name            = "usecase"
dependency_mode = "opt-in"
allow           = ["domain"]       # domainのみ参照可

[[layers]]
name            = "infrastructure"
dependency_mode = "opt-out"        # 内部レイヤーは全部OK
deny            = []               # 特定レイヤーを禁止する場合のみ列挙

[[layers]]
name            = "presentation"
dependency_mode = "opt-in"
allow           = ["usecase", "domain"]
```

### 外部ライブラリ依存（external_mode）

`external_allow` / `external_deny` の値はパッケージ名の正規表現で指定する。
これにより `"sqlx|sea-orm|diesel"` のようにまとめて記述できる。

```toml
[[layers]]
name           = "domain"
external_mode  = "opt-in"
external_allow = []                          # 外部ライブラリは原則NG

[[layers]]
name           = "usecase"
external_mode  = "opt-in"
external_allow = ["serde", "uuid", "chrono"] # 許可するライブラリのみ列挙

[[layers]]
name           = "infrastructure"
external_mode  = "opt-out"                   # 何でも使ってOK
external_deny  = []                          # 禁止したいものだけ列挙

[[layers]]
name           = "presentation"
external_mode  = "opt-in"
external_allow = ["clap", "serde"]
```

---

## 4. import解決の設計

### 4-1. 3段階の解決モデル

```
① テキスト抽出（全言語共通）
     tree-sitterでimport文の文字列を取得
     例: "../../infrastructure/db"  /  "myapp/infra/postgres"
              ↓
② パス正規化（言語別ロジック）
     相対パス → 絶対パス変換
     エイリアス解決（tsconfig paths, Go module名など）
     internal / external / stdlib / unknown の判定
              ↓
③ レイヤーマッピング（設定ベース）
     解決済みパスをmille.tomlのlayer定義に照合
     どのレイヤーからどのレイヤーへの依存かを特定
```

### 4-2. 解決モード

| モード | 説明 | 推奨言語 |
|---|---|---|
| `path` | ファイルパスベース（デフォルト） | Python, 全言語共通 |
| `module` | モジュール名ベース | Go, Rust |
| `hybrid` | パス・モジュール名の両方を解決 | TypeScript |

### 4-3. 言語別エイリアス解決

```toml
[resolve.typescript]
tsconfig = "./tsconfig.json"           # paths / baseUrl を自動読み取り

[resolve.go]
module_name = "github.com/myorg/myapp" # go.modから自動読み取り or 手動指定

[resolve.python]
src_root = "src"                       # from myapp.domain → src/myapp/domain に解決

[resolve.rust]
# Cargo.tomlから自動読み取り

[resolve.aliases]                      # 手動エイリアス追加
"@domain" = "src/domain"
"@infra"  = "src/infrastructure"
```

### 4-4. importの判定カテゴリ

| カテゴリ | 説明 | 処理 |
|---|---|---|
| `internal` | 自プロジェクト内のパス | レイヤーマッピング → dependency_modeでチェック |
| `external` | node_modules・外部pkg | external_modeでチェック |
| `stdlib` | 標準ライブラリ | デフォルト無視（設定で変更可） |
| `unknown` | 解決できなかったもの | warningを出力 |

### 4-5. 解決優先度

1. `mille.toml` の手動エイリアス指定
2. 言語設定ファイル（tsconfig.json, go.mod, Cargo.toml）から自動読み取り
3. `src_root` からの相対解決

---

## 5. 設定ファイル（mille.toml）仕様

### キー一覧

**`[project]`**

| キー | 説明 |
|---|---|
| `name` | プロジェクト名 |
| `root` | 解析対象のルートディレクトリ |
| `languages` | 対象言語のリスト |

**`[[layers]]`**

| キー | 説明 |
|---|---|
| `name` | レイヤー名 |
| `paths` | このレイヤーに属するファイルのglobパターン |
| `dependency_mode` | 内部レイヤーへの依存方針。`opt-in` / `opt-out` |
| `allow` | dependency_mode=opt-inのとき、参照を許可する内部レイヤー名のリスト |
| `deny` | dependency_mode=opt-outのとき、参照を禁止する内部レイヤー名のリスト |
| `external_mode` | 外部ライブラリへの依存方針。`opt-in` / `opt-out` |
| `external_allow` | external_mode=opt-inのとき、使用を許可するパッケージ名（正規表現）のリスト |
| `external_deny` | external_mode=opt-outのとき、使用を禁止するパッケージ名（正規表現）のリスト |

**`[[layers.allow_call_patterns]]`**

| キー | 説明 |
|---|---|
| `callee_layer` | 呼び出される側のレイヤー名 |
| `allow_methods` | 許可するメソッド名のリスト |

**`[ignore]`**

| キー | 説明 |
|---|---|
| `paths` | チェック対象から除外するglobパターンのリスト |
| `test_patterns` | テストファイルのglobパターン。依存ルールを緩める対象 |

**`[resolve.<language>]`**

| キー | 説明 |
|---|---|
| `tsconfig` | TypeScript: tsconfig.jsonのパス。paths / baseUrl を自動読み取り |
| `module_name` | Go: モジュール名。go.modから自動読み取りも可 |
| `src_root` | Python: ソースルートディレクトリ |

**`[resolve.aliases]`**

任意のキーで手動エイリアスを定義する。`"@domain" = "src/domain"` の形式。

**`[severity]`**

| キー | デフォルト | 説明 |
|---|---|---|
| `dependency_violation` | `"error"` | 内部レイヤー依存違反 |
| `external_violation` | `"error"` | 外部ライブラリ依存違反 |
| `call_pattern_violation` | `"error"` | entrypointのメソッド呼び出し違反 |
| `unknown_import` | `"warning"` | 解決できなかったimport |

値は `"error"` / `"warning"` / `"info"` から選択。

---

### 設定ファイル全体例

```toml
[project]
name      = "my-app"
root      = "."
languages = ["typescript", "go"]

[[layers]]
name            = "domain"
paths           = ["src/domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-in"
external_allow  = []

[[layers]]
name            = "usecase"
paths           = ["src/usecase/**", "src/application/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-in"
external_allow  = ["serde", "uuid", "chrono"]

[[layers]]
name            = "infrastructure"
paths           = ["src/infrastructure/**", "src/adapter/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "presentation"
paths           = ["src/presentation/**", "src/handler/**"]
dependency_mode = "opt-in"
allow           = ["usecase", "domain"]
external_mode   = "opt-in"
external_allow  = ["clap", "serde"]

[[layers]]
name            = "main"
paths           = ["src/main.rs", "cmd/main.go", "main.py"]
dependency_mode = "opt-in"
allow           = ["infrastructure", "usecase", "presentation"]
external_mode   = "opt-in"
external_allow  = ["clap"]

  [[layers.allow_call_patterns]]
  callee_layer  = "infrastructure"
  allow_methods = ["new", "build", "create", "init", "setup"]

[ignore]
paths         = ["**/*.test.ts", "**/*_test.go", "**/mock/**"]
test_patterns = ["**/*.spec.*", "**/*_test.*"]

[resolve.typescript]
tsconfig = "./tsconfig.json"

[resolve.go]
module_name = "github.com/myorg/myapp"

[resolve.python]
src_root = "src"

[resolve.aliases]
"@domain" = "src/domain"
"@infra"  = "src/infrastructure"

[severity]
dependency_violation     = "error"
external_violation       = "error"
call_pattern_violation   = "error"
unknown_import           = "warning"
```

---

## 6. CLIインターフェース

### コマンド構成

```
mille <command> [options]
```

| コマンド | 説明 |
|---|---|
| `mille init` | 対話形式で`mille.toml`を生成 |
| `mille check` | ルール違反を検出してレポート出力 |
| `mille analyze` | 現状の依存グラフをJSON/DOT形式で出力（ルール適用なし） |
| `mille report external` | 外部ライブラリ依存レポートをレイヤーごとに出力 |

### オプション

| オプション | 説明 |
|---|---|
| `--config <path>` | 設定ファイルのパスを指定（デフォルト: `./mille.toml`） |
| `--format <fmt>` | 出力形式: `terminal` / `json` / `github-actions` / `dot` |
| `--fail-on <level>` | 終了コード1にするレベル: `error` / `warning` |
| `--fix` | 自動修正可能な違反を修正（将来対応） |

### 終了コード

| コード | 意味 |
|---|---|
| `0` | 違反なし |
| `1` | error違反あり |
| `2` | warning違反あり（`--fail-on warning`指定時） |
| `3` | 設定ファイルエラー |

---

## 7. 出力フォーマット

### terminal（デフォルト）

```
$ mille check

❌ [ERROR] Dependency violation
   src/usecase/UserUsecase.ts:12
   import { UserRepositoryImpl } from "../../infrastructure/UserRepositoryImpl"
   'usecase' → 'infrastructure' の依存は禁止されています（dependency_mode: opt-in）

❌ [ERROR] External violation
   src/usecase/OrderUsecase.ts:3
   import sqlx from "sqlx"
   'usecase' では 'sqlx' は許可されていません（external_mode: opt-in）

❌ [ERROR] Call pattern violation
   src/main.rs:15
   repo.find_user(1)
   'infrastructure' の 'find_user' は呼び出せません
   許可されているメソッド: new, build, create, init, setup

✅ domain         (12 files,  0 violations)
❌ usecase        ( 8 files,  2 violations)
✅ infrastructure (15 files,  0 violations)
✅ presentation   ( 6 files,  0 violations)
❌ main           ( 1 file,   1 violation)

Summary: 3 errors, 0 warnings
Exit code: 1
```

### json

```json
{
  "summary": { "errors": 3, "warnings": 0 },
  "violations": [
    {
      "severity": "error",
      "rule": "dependency",
      "file": "src/usecase/UserUsecase.ts",
      "line": 12,
      "from_layer": "usecase",
      "to_layer": "infrastructure",
      "import": "../../infrastructure/UserRepositoryImpl"
    },
    {
      "severity": "error",
      "rule": "external",
      "file": "src/usecase/OrderUsecase.ts",
      "line": 3,
      "from_layer": "usecase",
      "import": "sqlx",
      "reason": "external_mode is opt-in, 'sqlx' is not in external_allow"
    },
    {
      "severity": "error",
      "rule": "call_pattern",
      "file": "src/main.rs",
      "line": 15,
      "callee_layer": "infrastructure",
      "method": "find_user",
      "reason": "method 'find_user' is not in allow_methods: [new, build, create, init, setup]"
    }
  ]
}
```

### github-actions

```
::error file=src/usecase/UserUsecase.ts,line=12::'usecase' → 'infrastructure' の依存は禁止されています
::error file=src/usecase/OrderUsecase.ts,line=3::'usecase' では 'sqlx' は許可されていません
::error file=src/main.rs,line=15::'infrastructure' の 'find_user' は呼び出せません
```

---

## 8. 既存ツールとの比較

| ツール | 言語 | 設定方式 | 可視化 | 多言語 | 外部依存チェック |
|---|---|---|---|---|---|
| **mille**（本ツール） | Rust | TOML | ✅ DOT出力 | ✅ | ✅ opt-in/out |
| dependency-cruiser | Node.js | JS/JSON | ✅ グラフ | ❌ JS/TSのみ | △ |
| ArchUnit | Java | コード | ❌ | ❌ Java/Kotlinのみ | ❌ |
| go-arch-lint | Go | YAML | ❌ | ❌ Goのみ | ❌ |
| deptrac | PHP | YAML | △ | ❌ PHPのみ | ❌ |

**milleの差別化ポイント:**

- 多言語を1つのTOMLで統一管理できる唯一のCLIツール
- 内部依存・外部ライブラリ依存を同一のopt-in/opt-outモデルで管理
- entrypointレイヤーのDI強制によるアーキテクチャ保護
- Rust製シングルバイナリによる高速実行

---

*最終更新: 2026-03-07*
