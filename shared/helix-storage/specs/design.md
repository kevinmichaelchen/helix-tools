# helix-storage: Design Specification

**Document:** design.md  
**Status:** Deprecated — Use HelixDB directly  
**Author:** Kevin Chen

> **This crate should not be used.**  
> New tools should use HelixDB directly. This spec exists only to document what helix-storage was and why it's being removed.

## What helix-storage Was

helix-storage provided a trait-based storage abstraction (`VectorStorage<T>`) with JSON file persistence. It was created as scaffolding for early development.

## Why It's Being Removed

| Limitation | Impact |
|------------|--------|
| Full JSON load on startup | O(n) memory, slow for large repos |
| Brute-force vector search | O(n) per query |
| No graph traversal | Must materialize relationships in memory |
| No incremental updates | Full rewrite on every change |
| No secondary indices | Full scan for filtered queries |

**HelixDB provides all of these natively.**

## Migration Path

### For helix-decisions

Replace `PersistentDecisionStorage` (JSON) with `HelixDecisionStorage` (HelixDB native).

Reference:
- `helix-decisions/docs/phase3/PHASE_3_PLAN.md`
- `helix-decisions/docs/phase3/PHASE_3_CORRECTIONS.md`

### For New Tools

Do not use helix-storage. Use HelixDB directly from the start.

## HelixDB API Patterns

When integrating HelixDB, follow the corrected patterns in:
- `helix-decisions/docs/phase3/PHASE_3_CORRECTIONS.md`
- `helix-decisions/docs/phase3/CORRECTIONS_QUICK_REFERENCE.txt`

Key requirements:
- **Edges:** Write to 3 databases (edges_db, out_edges_db, in_edges_db)
- **Nodes:** Use arena allocation + ImmutablePropertiesMap
- **Vectors:** Stored separately, linked via vector_id property
- **Keys:** Use `hash_label()` for adjacency DB keys

## See Also

- [helix-decisions/specs/design.md](../../helix-decisions/specs/design.md) — HelixDB integration reference
- [shared/AGENTS.md](../AGENTS.md) — Shared crates overview
