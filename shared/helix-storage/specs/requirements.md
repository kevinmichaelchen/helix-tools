# helix-storage: Requirements Specification

**Document:** requirements.md  
**Status:** Deprecated — Use HelixDB directly  
**Author:** Kevin Chen

> **This crate should not be used.**  
> These requirements document the original scope. New tools should use HelixDB directly.

## Original Vision

helix-storage was intended to provide trait-based storage abstraction for helix-tools, enabling semantic search with project-local persistence.

## Why HelixDB Instead

HelixDB provides everything helix-storage attempted, plus:
- Native HNSW vector indexing
- Graph traversal (edges, relationships)
- LMDB persistence (no full reload)
- Secondary indices for filtered queries
- Incremental updates (no full rewrite)

## See Also

- [design.md](./design.md) — Architecture and migration notes
- [helix-decisions/docs/phase3/](../../helix-decisions/docs/phase3/) — HelixDB integration reference
