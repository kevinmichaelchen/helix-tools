# helix-storage

Trait-based storage abstraction for helix-tools with project-local persistence.

## Why

Multiple helix-tools need persistent storage with vector search:
- **helix-decisions** — Store decision embeddings for semantic search
- **hbd** — Store issue embeddings (planned)
- **helix-map** — Store symbol index

This crate provides a unified abstraction so each tool doesn't reinvent storage logic.

## Storage Modes

| Mode | Location | Use Case |
|------|----------|----------|
| **Project-local** | `.helix/data/{tool}/` | Decisions, issues, code index |
| **Global** | `~/.helix/data/{tool}/` | Documentation cache (helix-docs) |

Most tools use project-local storage. Data lives **in the git repo**, not globally.

## Usage

```rust
use helix_storage::{Storage, StorageConfig, JsonFileBackend};

// Project-local storage (default)
let config = StorageConfig::project_local("decisions")?;
let storage = JsonFileBackend::open(&config)?;

// Store a document with embedding
storage.insert(StorageNode {
    id: "doc-123".to_string(),
    data: serde_json::json!({"title": "My Decision"}),
    embedding: Some(vec![0.1, 0.2, 0.3]),
    content_hash: "abc123".to_string(),
})?;

// Search by embedding
let results = storage.search(&query_embedding, 10)?;

// Get by ID
let node = storage.get("doc-123")?;
```

## Backends

### JsonFileBackend (Current)

Simple JSON file storage. Works immediately, no external dependencies.

- **Persistence:** Single JSON file per tool
- **Search:** In-memory cosine similarity
- **Pros:** Simple, portable, git-friendly
- **Cons:** Loads entire index into memory

### HelixDBBackend (Planned)

Native HelixDB integration for graph + vector workloads.

- **Persistence:** HelixDB data directory
- **Search:** Native vector index
- **Pros:** Fast, scalable, graph traversal
- **Cons:** Requires HelixDB integration (in progress)

## Project Hash

For project-local storage, we need to identify which project we're in:

```rust
use helix_storage::project_hash;

// Get hash from git root
let hash = project_hash()?;  // e.g., "a1b2c3"

// Storage location: .helix/data/decisions/{hash}/
```

## Configuration

Storage respects helix-config hierarchy:

```toml
# ~/.helix/config/config.toml (global)
[storage]
base = "~/.helix/data"  # Only for global storage

# .helix/config.toml (project) — not typically needed
```

## Consumers

| Crate | Storage Mode | Tool Name |
|-------|--------------|-----------|
| helix-decisions | project-local | `decisions` |
| hbd | project-local | `hbd` |
| helix-map | project-local | `map` |
| helix-docs | global | `docs` |

## License

MIT
