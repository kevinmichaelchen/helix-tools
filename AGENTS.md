# HELIX-TOOLS AGENTS

Workspace guide for the helix-tools Rust monorepo. Each crate now has its own
`AGENTS.md` next to its `Cargo.toml` for crate-specific guidance.

## Overview

- Tools: `hbd`, `hbd-ui`, `ixchel`
- Shared HelixDB helpers: `ix-helixdb-ops`
- Docs site (Next.js/Fumadocs) lives in `docs/`

## Workspace Layout

```
helix-tools/
├── hbd/                    # Git-first issue tracker CLI
├── hbd-ui/                 # Svelte UI for hbd (frontend)
│   └── src-tauri/           # Tauri shell (Rust)
├── ix-helixdb-ops/          # HelixDB graph helper crate
├── ix-cli/                  # Ixchel CLI (binary: ixchel)
├── ix-config/               # Global + project config loading helpers
├── ix-core/                 # Ixchel core library (git-first, markdown-canonical)
├── ix-daemon/               # Background daemon + IPC (binary: ixcheld)
├── ix-embeddings/           # Embedding providers + Embedder API
├── ix-id/                   # Hash-based id helpers (prefix-hash ids)
├── ix-mcp/                  # Ixchel MCP server (binary: ixchel-mcp)
├── ix-storage-helixdb/      # Ixchel HelixDB-backed index/cache adapter
└── docs/                    # Next.js documentation site
```

## Conventions

- Rust edition is `2024` for workspace crates unless overridden.
- Workspace lints are enforced via `Cargo.toml` in the repo root.
- Prefer shared Ixchel libs (`ix-config`, `ix-id`, `ix-embeddings`, etc.) where appropriate.
- Use Conventional Commits for all commit messages.

## Where To Look

| Task                              | Location             |
| --------------------------------- | -------------------- |
| Workspace members and shared deps | `Cargo.toml`         |
| CI workflows                      | `.github/workflows/` |
| Docs site                         | `docs/AGENTS.md`     |
| Shared crates overview            | `shared/AGENTS.md`   |

## Commands

```bash
dprint check
cargo build --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```
