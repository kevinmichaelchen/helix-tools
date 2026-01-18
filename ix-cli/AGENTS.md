# IXCHEL AGENTS (ix-cli)

## Scope

Applies to the `ix-cli/` crate.

## Guidelines

- Keep CLI “thin”: argument parsing + formatting only.
- No direct storage/backend usage; call into `ix-core`.

## Commands

```bash
cargo test -p ix-cli
dprint fmt
```
