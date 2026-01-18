# IXCHEL AGENTS (ix-storage-helixdb)

## Scope

Applies to the `ix-storage-helixdb/` crate.

## Guidelines

- Keep business rules out of the adapter; implement `ix-core` traits only.
- Prefer explicit boundary types and error mapping.

## Commands

```bash
cargo test -p ix-storage-helixdb
dprint fmt
```
