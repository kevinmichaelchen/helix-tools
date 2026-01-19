# HELIX-TOOLS AGENTS

Workspace guide for the helix-tools Rust monorepo. Each crate now has its own
`AGENTS.md` next to its `Cargo.toml` for crate-specific guidance.

## Overview

- Tools: `hbd`, `helix-docs`, `helix-map`, `helix-repo`, `hbd-ui`, `ixchel`
- Shared HelixDB helpers: `helix-graph-ops`
- Shared crates live under `shared/`
- Docs site (Next.js/Fumadocs) lives in `docs/`

## Workspace Layout

```
helix-tools/
├── hbd/                    # Git-first issue tracker CLI
├── hbd-ui/                 # Svelte UI for hbd (frontend)
│   └── src-tauri/           # Tauri shell (Rust)
├── helix-graph-ops/         # HelixDB graph helper crate
├── helix-docs/              # Global docs cache CLI + library
├── helix-map/               # Codebase indexer CLI + library
├── helix-repo/              # Repo clone manager CLI + library
├── ix-cli/                  # Ixchel CLI (binary: ixchel)
├── ix-core/                 # Ixchel core library (git-first, markdown-canonical)
├── ix-mcp/                  # Ixchel MCP server (binary: ixchel-mcp)
├── ix-storage-helixdb/      # Ixchel HelixDB-backed index/cache adapter
├── shared/                  # Shared Rust crates
└── docs/                    # Next.js documentation site
```

## Conventions

- Rust edition is `2024` for workspace crates unless overridden.
- Workspace lints are enforced via `Cargo.toml` in the repo root.
- Prefer shared crates (`helix-config`, `helix-id`, `helix-embeddings`, etc.) where appropriate.
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
