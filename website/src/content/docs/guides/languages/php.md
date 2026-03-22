---
title: PHP
description: PHP プロジェクトでの mille 設定例
---

## 設定例

```toml
[project]
name      = "my-laravel-app"
root      = "."
languages = ["php"]

[resolve.php]
namespace      = "App"
composer_json  = "composer.json"   # PSR-4 から自動検出

[[layers]]
name            = "domain"
paths           = ["src/Domain/**"]
dependency_mode = "opt-in"
allow           = []
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "usecase"
paths           = ["src/UseCase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]
external_mode   = "opt-out"
external_deny   = []

[[layers]]
name            = "infrastructure"
paths           = ["src/Infrastructure/**"]
dependency_mode = "opt-out"
deny            = []
external_mode   = "opt-out"
external_deny   = []
```

## Composer PSR-4 自動検出

`composer.json` に PSR-4 autoload が定義されていれば、`namespace` を自動で検出します。

```json
{
  "autoload": {
    "psr-4": {
      "App\\": "src/"
    }
  }
}
```

手動で指定する場合は `[resolve.php]` の `namespace` に直接記述します。

```toml
[resolve.php]
namespace = "App"
```

## インポートの分類

| インポート | 分類 |
|---|---|
| `use App\Domain\User` | 内部（`namespace` に一致） |
| `use App\UseCase\CreateUser` | 内部 |
| `use Illuminate\Http\Request` | 外部（Composer パッケージ） |
| `use PDO`, `use \DateTime` | 標準ライブラリ |

## グループ use 文

グループ use 文は個別のインポートとして展開されます。

```php
use App\Services\{Auth, Logger};
// → App\Services\Auth と App\Services\Logger の 2 件として解析
```

## function / const use

`use function` と `use const` もインポートとして検出されます。

```php
use function App\Helpers\format_date;
use const App\Config\MAX_RETRIES;
```
