

<!-- Source: AGENTS.md -->

# HELIX-TOOLS AGENTS

Workspace guide for the helix-tools Rust monorepo. Each crate now has its own
`AGENTS.md` next to its `Cargo.toml` for crate-specific guidance.

## Overview

- Tools: `hbd`, `helix-decisions`, `helix-docs`, `helix-map`, `helix-repo`, `hbd-ui`
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
├── helix-decisions/         # Decision graph CLI + library
├── helix-docs/              # Global docs cache CLI + library
├── helix-map/               # Codebase indexer CLI + library
├── helix-repo/              # Repo clone manager CLI + library
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
cargo build --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```



<!-- Source: .ruler/AGENTS.md -->

# HELIX-TOOLS AGENTS

Workspace guide for the helix-tools Rust monorepo. Each crate now has its own
`AGENTS.md` next to its `Cargo.toml` for crate-specific guidance.

## Overview

- Tools: `hbd`, `helix-decisions`, `helix-docs`, `helix-map`, `helix-repo`, `hbd-ui`
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
├── helix-decisions/         # Decision graph CLI + library
├── helix-docs/              # Global docs cache CLI + library
├── helix-map/               # Codebase indexer CLI + library
├── helix-repo/              # Repo clone manager CLI + library
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
cargo build --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```



<!-- Source: .ruler/rules/01-commits.md -->

# Git Commit Hygiene

## 1) Conventional Commit spec

Use the Conventional Commits format:

```
<type>(<scope>): <short, imperative summary>
```

Allowed types (common):
- feat
- fix
- docs
- refactor
- test
- chore
- ci
- build
- perf

Rules:
- Use present-tense, imperative mood ("add", "fix", "refactor").
- Keep the subject concise but meaningful (aim for 50-72 chars).
- Use a scope when the change is localized (crate, module, subsystem).
- If a breaking change exists, add `!` after the type/scope or include a
  `BREAKING CHANGE:` footer in the body.

## 2) High level of detail

Every commit must include a body with concrete, scannable detail:

```
<type>(<scope>): <short summary>

- bullet 1: what changed and why
- bullet 2: key behavior or API change
- bullet 3: notable edge case / follow-up / limitation
```

Guidelines:
- Prefer 3-6 bullets.
- Mention user-facing behavior changes explicitly.
- Include rationale when it is not obvious from the diff.
- Call out migrations, config updates, or data shape changes.

## 3) Exemplary commit

```
feat(helix-embeddings): add provider abstraction with fastembed support

- introduce EmbeddingProvider trait to decouple model implementation
- add fastembed provider with dynamic dimension detection and validation
- expose provider/model metadata for downstream consumers
- document new embedding config fields (provider, dimension)
```
