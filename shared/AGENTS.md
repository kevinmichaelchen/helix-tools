# SHARED CRATES AGENTS

Shared Rust crates used across multiple helix-tools. Each crate also has its
own `AGENTS.md` next to its `Cargo.toml` for details.

## Overview

- Shared crates should stay lightweight and reusable.
- Avoid direct HelixDB dependencies here; tools own storage traits/backends.

## Crates

| Crate                     | Purpose                        |
| ------------------------- | ------------------------------ |
| `shared/helix-config`     | Hierarchical config loading    |
| `shared/helix-id`         | Hash-based ID generation       |
| `shared/helix-embeddings` | fastembed wrapper              |
| `shared/helix-discovery`  | Git/project marker discovery   |
| `shared/helix-daemon`     | IPC client/server for helixd   |

## Where To Look

| Task                    | Location             |
| ----------------------- | -------------------- |
| Shared crate specs      | `shared/*/specs/`    |
| Crate-specific guidance | `shared/*/AGENTS.md` |
