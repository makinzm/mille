# CI/CD トークン設定ガイド

本プロジェクトでは GitHub Actions を用いて、リリースタグ (`v*.*.*`) をトリガーに各種パッケージマネージャへの自動デプロイを行っています。

---

## リリースフロー概要

```
Cargo.toml の version を更新してコミット
         ↓
git tag v1.2.3 && git push origin v1.2.3
         ↓
.github/workflows/release.yml が起動
         ↓
┌─────────────────────────────────────────┐
│ build           : クロスコンパイル (5 target)  │
│ build-deb       : .deb パッケージ (amd64/arm64) │
│         ↓                               │
│ release         : GitHub Release 作成   │
│         ↓ (並列)                        │
│ publish-crates  → crates.io             │
│ publish-npm     → npm (@makinzm/mille)  │
│ publish-pypi    → PyPI (mille)          │
│ update-homebrew → homebrew-tap リポジトリ │
│ publish-nix     → Cachix バイナリキャッシュ │
└─────────────────────────────────────────┘
```

### バージョニングルール

- `Cargo.toml` の `version` が **正規のバージョン**。
- リリース前に `Cargo.toml` のバージョンを更新してコミットする。
- タグ名は `v{version}` 形式（例: `v1.2.3`）。
- npm / PyPI のバージョンは CI が自動で Cargo.toml の値に合わせて上書きする。

---

## 必要なシークレット一覧

リポジトリの **Settings > Environments** にて、各ジョブに対応した Environment を作成して登録してください。

| シークレット名          | Environment 名         | 用途                                  | 取得元 URL                                                                 | 推奨スコープ                             |
|------------------------|------------------------|---------------------------------------|----------------------------------------------------------------------------|------------------------------------------|
| `CARGO_REGISTRY_TOKEN` | `CARGO_REGISTRY_TOKEN` | crates.io への `cargo publish`         | [crates.io API Tokens](https://crates.io/settings/tokens)                  | `publish-update`（該当パッケージのみ）   |
| `NPM_TOKEN`            | `NPM_TOKEN`            | npm への `npm publish`                 | [npm Access Tokens](https://www.npmjs.com/settings/tokens)                 | `Automation`（2FA 回避のため）           |
| `PYPI_TOKEN`           | `PYPI_TOKEN`           | PyPI への `twine upload`               | [PyPI Account Settings](https://pypi.org/manage/account/)                  | 該当パッケージのみに scope 限定          |
| `HOMEBREW_TAP_TOKEN`   | *(Repository secret)*  | `makinzm/homebrew-tap` へのプッシュ    | [GitHub Personal Access Tokens](https://github.com/settings/tokens)       | `repo` スコープ（`homebrew-tap` リポのみ）|
| `CACHIX_AUTH_TOKEN`    | *(Repository secret)*  | Cachix バイナリキャッシュへのプッシュ  | [Cachix Dashboard](https://app.cachix.org/)                                | キャッシュ名 `mille` への write 権限     |

> **注意:** `HOMEBREW_TAP_TOKEN` と `CACHIX_AUTH_TOKEN` は特定の Environment に紐づけず、Repository secrets に登録してください。

---

## 各シークレットの発行手順

### 1. `CARGO_REGISTRY_TOKEN`
1. [crates.io](https://crates.io) にサインイン
2. [API Tokens](https://crates.io/settings/tokens) → "New Token"
3. スコープ: `publish-update`（初回は `publish-new` も必要）

### 2. `NPM_TOKEN`
1. [npm](https://www.npmjs.com) にサインイン
2. [Access Tokens](https://www.npmjs.com/settings/tokens) → "Generate New Token"
3. タイプ: **Automation**（2FA が有効な場合に Publish 時 OTP を回避）

### 3. `PYPI_TOKEN`
1. [PyPI](https://pypi.org) にサインイン
2. [Account settings](https://pypi.org/manage/account/) → "Add API token"
3. 初回パッケージ作成時は "Entire account"。作成後は `mille` パッケージのみに scope を制限したトークンを再発行する。

### 4. `HOMEBREW_TAP_TOKEN`
1. GitHub で `makinzm/homebrew-tap` リポジトリを作成（空で可）
2. [Personal Access Tokens](https://github.com/settings/tokens) → "Generate new token"
3. スコープ: `repo`（`homebrew-tap` リポジトリへの `Read and Write` 権限）
4. Repository secret として登録

### 5. `CACHIX_AUTH_TOKEN`
1. [Cachix](https://app.cachix.org/) にサインイン
2. "Create cache" → キャッシュ名 `mille`（パブリックで可）
3. "Auth Tokens" → "Create Token" → write 権限
4. Repository secret として登録

---

## 各パッケージマネージャからのインストール方法

### cargo（Rust ユーザー向け）
```sh
cargo install mille
```

### npm
```sh
npm install --save-dev @makinzm/mille
npx mille check
```

### PyPI（uv / pip）
```sh
uv add --dev mille
uv run mille check
# または
pip install mille
```

### Homebrew（macOS / Linux）

> **前提:** `makinzm/homebrew-tap` リポジトリが存在し、`Formula/mille.rb` が配置されていること。

```sh
brew tap makinzm/tap
brew install mille
```

### apt（Debian / Ubuntu）

GitHub Releases から `.deb` を直接インストールします。

```sh
VERSION=1.2.3  # 対象バージョンに書き換える
curl -LO "https://github.com/makinzm/mille/releases/download/v${VERSION}/mille_${VERSION}_amd64.deb"
sudo dpkg -i "mille_${VERSION}_amd64.deb"
```

### Nix / devbox

**`nix profile` で直接インストール（タグ指定）:**
```sh
nix profile install github:makinzm/mille/v1.2.3
```

**`nix run` で一時実行:**
```sh
nix run github:makinzm/mille -- check
```

**devbox プロジェクトへ追加:**
```sh
devbox add github:makinzm/mille/v1.2.3#mille
```

**Cachix キャッシュを有効化（コンパイル不要にする）:**
```sh
cachix use mille
# その後 nix profile install / nix run が高速になる
```

---

## Go パッケージ（go install）

Go モジュールプロキシはタグを自動検出するため、追加の CI/CD 設定は不要です。
ただし、バージョンタグは `v{version}` 形式（例: `v1.2.3`）で push してください。

```sh
go install github.com/makinzm/mille/packages/go@latest
```

---

## 設定完了後の確認手順

1. `Cargo.toml` の `version` を更新してコミット
2. `git tag v{version} && git push origin v{version}` を実行
3. GitHub Actions の **Release** ワークフローが起動することを確認
4. 全ジョブが green になったら各パッケージマネージャからインストールして動作確認
