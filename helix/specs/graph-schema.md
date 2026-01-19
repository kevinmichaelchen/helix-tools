# Graph Schema Specification (HelixDB Cache)

Ixchel uses HelixDB as a **rebuildable cache** for fast semantic search and graph
queries. Markdown under `.ixchel/` is the source of truth.

This document describes:

- What the current `ix-storage-helixdb` backend stores today (MVP)
- What we intend to store as the system grows (planned)

## Current Schema (MVP)

### Node Label

All entities are stored with a single node label:

```
IXCHEL_ENTITY
```

Entity kind is stored in a `kind` property (for example `decision`, `issue`).

### Node Properties

| Property       | Type   | Required | Notes                                      |
| -------------- | ------ | -------- | ------------------------------------------ |
| `id`           | String | Yes      | `{prefix}-{6..12 hex}`                     |
| `kind`         | String | Yes      | `decision                                  |
| `title`        | String | Yes      | From frontmatter                           |
| `status`       | String | No       | From frontmatter                           |
| `file_path`    | String | Yes      | Path relative to repo root                 |
| `content_hash` | String | Yes      | BLAKE3 hash of the canonical Markdown file |
| `vector_id`    | String | Yes      | Internal HelixDB vector identifier         |
| `tags`         | String | No       | JSON-encoded string list                   |

### Edge Labels

Edges are derived from Markdown frontmatter relationship keys:

- Frontmatter key `implements` becomes edge label `IMPLEMENTS`
- Values must look like entity IDs (`{prefix}-{hex}`)
- Unknown prefixes are reported by `ixchel check`

Edge properties are not stored yet (full rebuild on `sync`).

### Vector Index

- One embedding per entity (title + body + tags), stored as a vector
- Default embedding model: `BAAI/bge-small-en-v1.5` (384 dims)
- Distance: cosine similarity (via HelixDB vector config)

## Planned Extensions

### Typed Node Labels

We may evolve to typed labels (`DECISION`, `ISSUE`, etc.) once the schema
stabilizes. The current single-label approach keeps the MVP simple.

### Edge Properties

Planned edge properties (especially for agent-driven workflows):

| Property           | Type     | Description                              |
| ------------------ | -------- | ---------------------------------------- |
| `created_at`       | DateTime | When the relationship was created        |
| `created_by`       | String   | Who created it                           |
| `confidence`       | Float    | 0..1 strength (for suggested/auto-mined) |
| `lease_expires_at` | DateTime | For `CLAIMS` edges to prevent deadlocks  |
| `note`             | String   | Optional annotation                      |

### Validity Rules (Strict Mode)

Ixchel intends to support a strict validity matrix for edges (per type pair),
with a permissive mode for custom relationship keys.

Examples:

- `IMPLEMENTS`: Issue → Decision
- `BLOCKS`/`DEPENDS_ON`: Issue → Issue
- `SUPERSEDES`: Decision → Decision
- `CITES`: Decision/Report → Source/Citation

## Data Integrity

- Markdown is canonical; the cache is rebuildable
- `ixchel check` validates file/ID consistency and broken links
- Future: incremental sync keyed by `content_hash` and model version
