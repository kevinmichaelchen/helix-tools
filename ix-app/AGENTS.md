# IXCHEL AGENTS (ix-app)

## Scope

Applies to the `ix-app/` crate.

## Guidelines

- Keep this crate “wiring only”: backend selection + orchestration.
- Do not put domain rules here; keep them in `ix-core`.
- Prefer small functions callable from CLIs and servers.

## Commands

```bash
cargo test -p ix-app
dprint fmt
```
