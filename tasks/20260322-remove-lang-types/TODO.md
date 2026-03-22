# Remove language-specific types from domain/usecase

## Goal
Remove all language-specific types, variables, comments, and test names from domain and usecase layers to pass naming convention checks.

## Steps
1. [ ] Create `ResolveConfigGenerator` trait in domain + impl in infrastructure
2. [ ] Move resolve-specific logic from init.rs to infrastructure
3. [ ] Remove language-specific types from domain/entity/config.rs
4. [ ] Update toml_config_repository.rs with two-pass parsing
5. [ ] Update DispatchingResolver to accept toml::Value
6. [ ] Update runner.rs wiring
7. [ ] Fix all comments and test names in domain/usecase
8. [ ] Verify: cargo test passes, mille check shows zero naming violations
