---
title: PHP
description: mille configuration for PHP projects
---

## Configuration Example

```toml
[project]
name      = "my-laravel-app"
root      = "."
languages = ["php"]

[resolve.php]
namespace      = "App"
composer_json  = "composer.json"   # auto-detect from PSR-4

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

## Composer PSR-4 Auto-Detection

If your `composer.json` defines PSR-4 autoloading, the base namespace is detected automatically:

```json
{
  "autoload": {
    "psr-4": {
      "App\\": "src/"
    }
  }
}
```

To specify it manually, set `namespace` in `[resolve.php]`:

```toml
[resolve.php]
namespace = "App"
```

## Import Classification

| Import | Classification |
|---|---|
| `use App\Domain\User` | Internal (matches `namespace`) |
| `use App\UseCase\CreateUser` | Internal |
| `use Illuminate\Http\Request` | External (Composer package) |
| `use PDO`, `use \DateTime` | Stdlib |

## Group Use Statements

Group use statements are expanded into individual imports:

```php
use App\Services\{Auth, Logger};
// → Resolved as App\Services\Auth and App\Services\Logger
```

## Function / Const Use

`use function` and `use const` statements are also detected as imports:

```php
use function App\Helpers\format_date;
use const App\Config\MAX_RETRIES;
```
