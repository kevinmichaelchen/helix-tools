# Shared Crates

Shared infrastructure for helix-tools. These crates provide common functionality used across multiple tools.

## Crates

| Crate | Purpose | Consumers | Status | Specs |
|-------|---------|-----------|--------|-------|
| [helix-config](./helix-config/) | Hierarchical config loading | All tools | Active | [specs](./helix-config/specs/) |
| [helix-id](./helix-id/) | Hash-based ID generation | hbd, helix-decisions | Active | [specs](./helix-id/specs/) |
| [helix-embeddings](./helix-embeddings/) | Semantic embeddings via fastembed | helix-decisions, hbd, helix-docs | Active | [specs](./helix-embeddings/specs/) |
| [helix-discovery](./helix-discovery/) | Git root and project marker discovery | helix-decisions, hbd | Active | [specs](./helix-discovery/specs/) |
| [helix-storage](./helix-storage/) | Trait-based vector storage | — | **Remove** | [specs](./helix-storage/specs/) |

## helix-storage Removal

> **Do not use helix-storage.** Use HelixDB directly.

helix-storage was scaffolding for early development. HelixDB provides everything it attempted:
- Native HNSW vector indexing
- Graph traversal (edges, relationships)
- LMDB persistence
- Secondary indices
- Incremental updates

See `helix-storage/specs/design.md` for migration notes.

## HelixDB API Patterns

When integrating HelixDB, follow the corrected patterns in:
- `helix-decisions/docs/phase3/PHASE_3_CORRECTIONS.md`
- `helix-decisions/docs/phase3/CORRECTIONS_QUICK_REFERENCE.txt`

Key requirements:
- **Edges:** Write to 3 databases (edges_db, out_edges_db, in_edges_db)
- **Nodes:** Use arena allocation + ImmutablePropertiesMap
- **Vectors:** Stored separately, linked via vector_id property
- **Keys:** Use `hash_label()` for adjacency DB keys

## Design Principles

1. **Loose coupling** — Tools depend on traits, not implementations
2. **Project-local by default** — Data lives in the repo (`.helix/data/`), not globally
3. **Config-driven** — Behavior configured via `~/.helix/config/` and `.helix/`
4. **Minimal dependencies** — Each crate depends only on what it needs
5. **Use HelixDB** — For graph/vector storage, use HelixDB directly

## Adding a New Shared Crate

1. Create directory: `shared/helix-{name}/`
2. Add `Cargo.toml`, `README.md`, `src/lib.rs`
3. Create `specs/` with `design.md` and `requirements.md`
4. Add to workspace `Cargo.toml` members
5. Update this AGENTS.md

## See Also

- `helix-decisions/specs/design.md` — HelixDB integration reference
- `helix-decisions/docs/phase3/` — Detailed implementation plans
