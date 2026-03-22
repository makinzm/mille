# Implementation Plan: PHP Support

## Branch name
`feat/php-support`

## Commit plan
1. `[test] add PHP parser and resolver tests (stubs) because of PHP support` (--no-verify)
2. `[fix] implement PHP parser and resolver because of PHP support`
3. `[refactor] update README and docs/TODO.md for PHP support`

---

## Files to create

### `src/infrastructure/parser/php.rs`
- `PhpParser` struct implementing `Parser` trait
- `parse_php_imports`: walks tree-sitter AST, extracts `use` statements
- `parse_php_names`: extracts class/function/method names and comments
- `parse_php_call_exprs`: returns empty vec (same as Java approach)

### `src/infrastructure/resolver/php.rs`
- `PhpResolver` struct implementing `Resolver` trait
- Constructor: `from_config(namespace, composer_json_path)` + `new(base_namespace)`
- `classify_php`: Internal if starts with base_namespace, Stdlib if in PHP built-in list, External otherwise
- `read_namespace_from_composer`: parses `composer.json` `autoload.psr-4` keys

## Files to modify

| File | Change |
|---|---|
| `Cargo.toml` | add `tree-sitter-php = "0.24"` |
| `src/infrastructure/parser/mod.rs` | add `php` module, `PhpParser` field, dispatch on `.php` |
| `src/infrastructure/resolver/mod.rs` | add `php` module, `PhpResolver` field, dispatch on `.php` |
| `src/usecase/init.rs` | add `"php" => Some("php")` in `ext_to_language` |
| `README.md` | add PHP to language support table |
| `docs/TODO.md` | update completion status |

---

## Test cases

### Parser tests (`src/infrastructure/parser/php.rs`)

| Test name | Validates | Fixture | Expected |
|---|---|---|---|
| `test_parse_php_simple_use` | `use App\Models\User;` | simple use statement | path = `App\Models\User`, line = 1 |
| `test_parse_php_aliased_use` | `use App\Models\User as UserModel;` | aliased use | path = `App\Models\User`, named_imports = [] |
| `test_parse_php_group_use` | `use App\Services\{Auth, Logger};` | group use | 2 imports: `App\Services\Auth`, `App\Services\Logger` |
| `test_parse_php_function_use` | `use function App\Helpers\format_date;` | function use | path = `App\Helpers\format_date` |
| `test_parse_php_const_use` | `use const App\Config\MAX_RETRIES;` | const use | path = `App\Config\MAX_RETRIES` |
| `test_parse_php_multiple_use` | mixed use statements | 3 use statements | 3 RawImport entries |
| `test_parse_php_no_imports` | no use statements | plain PHP class | empty vec |
| `test_parse_php_names_class` | class declaration | `class UserController` | Symbol "UserController" |
| `test_parse_php_names_function` | function declaration | `function getUserById()` | Symbol "getUserById" |
| `test_parse_php_names_comment` | line comment | `// connect to db` | Comment with "connect to db" |

### Resolver tests (`src/infrastructure/resolver/php.rs`)

| Test name | Validates | Input | Expected |
|---|---|---|---|
| `test_php_internal_is_internal` | import starts with base namespace | `App\Models\User`, base=`App` | Internal |
| `test_php_stdlib_datetime` | PHP stdlib class | `DateTime`, base=`App` | Stdlib |
| `test_php_stdlib_pdo` | PHP stdlib class | `PDO`, base=`App` | Stdlib |
| `test_php_stdlib_exception` | PHP stdlib class | `Exception`, base=`App` | Stdlib |
| `test_php_stdlib_leading_backslash` | `\DateTime` | `\DateTime`, base=`App` | Stdlib |
| `test_php_external_is_external` | third-party vendor | `Illuminate\Http\Request`, base=`App` | External |
| `test_php_resolver_internal_resolved_path` | resolved path uses slashes | `App\Models\User` | `App/Models/User.php` |
| `test_php_resolver_external_no_path` | external has no resolved path | `Illuminate\Http\Request` | None |
| `test_read_namespace_from_composer` | parse composer.json psr-4 | composer.json content | `App\\` → `App` |
| `test_classify_empty_base_namespace` | empty base namespace | any import | External |

---

## Fixture design

- Parser tests: inline PHP source strings, no filesystem access
- Resolver tests: `RawImport` structs built inline (`raw_php(path)` helper)
- `test_read_namespace_from_composer`: composer.json content string (no filesystem)

## PHP stdlib list (initial set)
`DateTime`, `DateTimeImmutable`, `DateInterval`, `DateTimeZone`, `PDO`, `PDOStatement`, `Exception`, `RuntimeException`, `InvalidArgumentException`, `LogicException`, `BadMethodCallException`, `OutOfRangeException`, `stdClass`, `ArrayObject`, `ArrayIterator`, `SplStack`, `SplQueue`, `SplFixedArray`, `Closure`, `Generator`, `Throwable`, `Error`, `TypeError`, `ValueError`

---

## Resolver classification logic
1. Strip leading `\` from path
2. If path (or its root namespace) is in PHP stdlib list → Stdlib
3. If path starts with `base_namespace\` or equals `base_namespace` → Internal
4. Otherwise → External

## Internal resolved path
`App\Models\User` → `App/Models/User.php` (backslashes to slashes, `.php` appended)

---

> Please confirm this plan and I will proceed to Phase 2 (TDD).
