---
id: dec-1d2c8f
type: decision
title: 'ADR-004: Trait-Based Storage Architecture'
status: accepted
date: 2026-01-06
created_at: 2026-01-18T23:33:16Z
updated_at: 2026-01-18T23:33:16Z
created_by: Kevin Chen
tags:
- architecture
- storage
- helixdb
- dependency-injection
---

> Migrated from `.decisions/004-trait-based-storage-architecture.md` into `.ixchel/decisions/`.

# ADR-004: Trait-Based Storage Architecture

**Status:** Accepted\
**Date:** 2026-01-06\
**Deciders:** Kevin Chen\
**Tags:** architecture, storage, helixdb, dependency-injection

## Context and Problem Statement

helix-tools uses HelixDB for persistent graph and vector storage. The question is: how should tools integrate with HelixDB?

**Option A (rejected):** Every tool directly depends on HelixDB

- Tight coupling
- Tools can't be tested without HelixDB
- Shared crates become HelixDB-aware

**Option B (accepted):** Trait-based architecture with dependency injection

- Loose coupling
- Tools define their own storage interfaces
- HelixDB is an implementation detail, not a dependency

## Decision Drivers

1. **Testability** — Tools should be testable without external dependencies
2. **Loose coupling** — Avoid forcing HelixDB into every crate
3. **Dependency inversion** — High-level modules shouldn't depend on low-level modules
4. **Flexibility** — Easy to add alternative implementations (memory, mock, etc.)
5. **Clarity** — Each tool's storage needs should be explicitly defined

## Decision

**Each tool defines its own storage trait. HelixDB implementations are provided separately.**

### Pattern

```
tool/
├── src/
│   ├── storage/
│   │   ├── mod.rs        # trait DecisionStore { ... }
│   │   ├── helix.rs      # impl DecisionStore for HelixBackend
│   │   └── memory.rs     # impl DecisionStore for MemoryBackend (tests)
│   └── lib.rs
```

### Example: helix-decisions

```rust
// storage/mod.rs — The trait (no HelixDB dependency)
pub trait DecisionStore: Send + Sync {
    fn insert(&self, decision: &Decision) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<Decision>>;
    fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    fn get_chain(&self, id: &str) -> Result<Vec<Decision>>;
    fn get_related(&self, id: &str) -> Result<Vec<(Decision, RelationType)>>;
}

// storage/helix.rs — HelixDB implementation
pub struct HelixDecisionStore {
    engine: HelixGraphStorage,
}

impl DecisionStore for HelixDecisionStore {
    fn insert(&self, decision: &Decision) -> Result<()> {
        // Uses HelixDB's native graph storage
        // LMDB persistence is built-in
    }
    // ...
}

// storage/memory.rs — In-memory implementation for tests
pub struct MemoryDecisionStore {
    decisions: HashMap<String, Decision>,
    embeddings: HashMap<String, Vec<f32>>,
}

impl DecisionStore for MemoryDecisionStore { ... }
```

### Shared Crates: No HelixDB Dependencies

| Crate            | Purpose               | HelixDB Dependency        |
| ---------------- | --------------------- | ------------------------- |
| helix-config     | Configuration loading | NO — just documents paths |
| helix-id         | ID generation         | NO — pure utility         |
| helix-embeddings | Text embeddings       | NO — returns `Vec<f32>`   |
| helix-discovery  | Git root discovery    | NO — pure utility         |

Consumers decide what to do with embeddings. helix-embeddings doesn't know or care about storage.

## Consequences

### Positive

- **Testability** — Tests use `MemoryDecisionStore`, no HelixDB needed
- **Loose coupling** — Tools work with trait interface, not HelixDB directly
- **Clear contracts** — Each tool's storage needs are explicitly defined
- **Flexibility** — Easy to add backends (SQLite, Postgres, etc.) later
- **No shared HelixDB dependency** — Shared crates stay pure

### Negative

- **More code** — Each tool defines its own trait (but traits are small)
- **No shared storage trait** — Tools can't share a single `VectorStore` trait

### Neutral

- **helix-storage removal** — We don't need a shared storage abstraction; each tool defines its own

## HelixDB's Built-In Persistence

HelixDB already handles persistence via LMDB:

```rust
// From helix-db/src/helix_engine/storage_core/mod.rs
let graph_env = unsafe {
    EnvOpenOptions::new()
        .map_size(db_size * 1024 * 1024 * 1024)
        .max_dbs(200)
        .open(Path::new(path))?  // Creates data.mdb + lock.mdb
};
```

This means:

- Data persists automatically across runs
- No JSON serialization needed
- No "index rebuild" on startup
- Tools just open HelixDB at a path and data is there

## Implementation Notes

### Storage Paths

Tools store HelixDB data in project-local directories:

```
{project}/.helix/data/{tool}/
├── data.mdb      # LMDB data file
└── lock.mdb      # LMDB lock file
```

### Configuration

helix-config documents HelixDB settings but doesn't depend on HelixDB:

```toml
# ~/.helix/config/config.toml
[helix_db]
map_size_mb = 1024
max_readers = 200
```

Tools read this config and pass it to their HelixDB backend.

## Alternatives Considered

### Shared `VectorStore` Trait in helix-storage

**Rejected** because:

- Different tools have different storage needs (graph traversal vs simple CRUD)
- Forces a lowest-common-denominator interface
- helix-storage was scaffolding, not a long-term solution

### Direct HelixDB Dependency in All Tools

**Rejected** because:

- Tight coupling
- Can't test without HelixDB
- Violates dependency inversion

## Related Decisions

- ADR-003: Binary Installation Strategy (distribution)
- Future: ADR for embedding model selection

## References

- [Dependency Inversion Principle](https://en.wikipedia.org/wiki/Dependency_inversion_principle)
- [HelixDB Storage Core](https://github.com/HelixDB/helix-db/blob/main/helix-db/src/helix_engine/storage_core/mod.rs)
