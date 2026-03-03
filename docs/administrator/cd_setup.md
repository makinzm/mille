# CI/CD トークン設定ガイド

本プロジェクトでは GitHub Actions を用いて、各種パッケージマネージャ (crates.io, npm, PyPI 等) への自動デプロイを行っています。
これらのジョブを成功させるためには、リポジトリの **Settings > Environments** を開き、各ジョブに対応した名前の Environment を作成した上で、**「Environment secrets」** にトークンを登録する必要があります。
（※現在PRにて `Environment secrets` を読み込めるようにワークフローを改修しました）

> **⚠️ 注意: Environmentの名前について**
> 作成する Environment の名前とセットするシークレット名は完全一致する必要があります。もし現在 `NPM_TOKEN` という名前のEnvironmentをご作成済みの場合は、その名前を `npm` に変更するか作り直していただき、その中に `NPM_TOKEN` というシークレットを入れる構造にしてください。

## 必要な Environment と シークレット一覧

| 作成する Environment | 登録するシークレット名 | 取得元 | 権限・スコープの設定 |
|---|---|---|---|
| `crates.io` | `CARGO_REGISTRY_TOKEN` | [crates.io Settings](https://crates.io/settings/tokens) | `publish-update` など該当パッケージの公開権限 |
| `npm`       | `NPM_TOKEN`            | [npm Access Tokens](https://www.npmjs.com/settings/tokens) | `Publish` または `Automation` 権限 |
| `pypi`      | `PYPI_TOKEN`           | [PyPI Account Settings](https://pypi.org/manage/account/) | 該当パッケージ（`mille`等）へのスコープ限定 API トークン |

## トークンの発行手順の概要

### 1. `CARGO_REGISTRY_TOKEN` (crates.io)
1. crates.io にサインインします。
2. アカウント設定 (Account Settings) の [API Tokens](https://crates.io/settings/tokens) ページに移動します。
3. "New Token" をクリックし、名前に `github-actions` 等を入力してトークンを生成します。

### 2. `NPM_TOKEN` (npm)
1. npm にサインインします。
2. アカウントの [Access Tokens](https://www.npmjs.com/settings/tokens) ページを開きます。
3. "Generate New Token" をクリックし、作成タイプとして "Automation" を選択して生成します。
   （※2FAを設定している場合は、Publish時にOTPを回避するために Automation トークンが必要です）

### 3. `PYPI_TOKEN` (PyPI)
1. PyPI にサインインします。
2. [Account settings](https://pypi.org/manage/account/) の API tokens セクションに移動します。
3. "Add API token" をクリックします。初めてパッケージを作成する場合は "Entire account" になりますが、ダミー作成後は該当パッケージのみに Scope を制限したトークンを発行することを推奨します。
4. 発行されたトークン (`pypi-` から始まる文字列) をコピーし、GitHub の **Settings > Environments** にて `pypi` Environment の **Environment secrets** に `PYPI_TOKEN` という名前で保存してください。

### 4. Goパッケージ (go install)
Go パッケージの公開は専用のレジストリ（npmやpypiなど）へのアップロードは不要で、GitHub リポジトリに適切な Git タグ （例: `packages/go/vX.Y.Z`）を Push するだけで完了します。
Goのプロキシサーバーが自動的にリポジトリのタグを検知してモジュールを解決するため、特別な CI/CD ジョブや API トークンは必要ありません。バージョニングの際は開発者が手動でタグを発行し `git push origin packages/go/vX.Y.Z` を行ってください。

---

## 設定が完了したら

`.github/workflows/cd-reserve.yml` などの CD パイプラインが正常に実行され、リリースが作成されるか確認してください。
