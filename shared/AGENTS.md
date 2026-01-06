# Shared Crates

Shared infrastructure for helix-tools. These crates provide common functionality used across multiple tools.

## Crates

| Crate | Purpose | Consumers |
|-------|---------|-----------|
| [helix-config](./helix-config/) | Hierarchical config loading | All tools |
| [helix-id](./helix-id/) | Hash-based ID generation | hbd, helix-decisions |
| [helix-storage](./helix-storage/) | Trait-based vector storage with persistence | helix-decisions, hbd, helix-docs |
| [helix-embeddings](./helix-embeddings/) | Semantic embeddings via fastembed | helix-decisions, hbd, helix-docs |
| [helix-discovery](./helix-discovery/) | Git root and project marker discovery | helix-decisions, hbd |

## Design Principles

1. **Loose coupling** — Tools depend on traits, not implementations
2. **Project-local by default** — Data lives in the repo (`.helix/data/`), not globally
3. **Config-driven** — Behavior configured via `~/.helix/config/` and `.helix/`
4. **Minimal dependencies** — Each crate depends only on what it needs

## Adding a New Shared Crate

1. Create directory: `shared/helix-{name}/`
2. Add `Cargo.toml`, `README.md`, `src/lib.rs`
3. Add to workspace `Cargo.toml` members
4. Update this AGENTS.md
