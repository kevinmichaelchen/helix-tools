# IXCHEL AGENTS (ix-core)

## Scope

Applies to the `ix-core/` crate.

## Guidelines

- Keep `ix-core` adapter-free: depend on traits/interfaces, not concrete storage backends.
- Prefer small, composable modules (registry, validation, sync, context).
- Keep public APIs stable; add feature flags before big surface changes.

## Commands

```bash
cargo fmt --all
cargo test -p ix-core
dprint fmt
```
