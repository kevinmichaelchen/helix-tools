# AGENTS.md - demo-got

AI agent instructions for the Game of Thrones family tree HelixDB demo.

## Overview

`demo-got` is a demonstration crate showing how to use HelixDB as an embedded graph database with vector search. It stores ~30 Game of Thrones characters as nodes, their family relationships as edges, and supports semantic search over character biographies.

## Quick Reference

| Task                   | Location         | Notes                                     |
| ---------------------- | ---------------- | ----------------------------------------- |
| Add CLI command        | `src/main.rs`    | Clap derive, `Commands` enum              |
| Add person property    | `src/types.rs`   | Update `Person` struct, then `storage.rs` |
| Add relationship type  | `src/types.rs`   | Add to `RelationType` enum                |
| Modify YAML schema     | `src/loader.rs`  | Update `RelationshipDef` enum             |
| Add graph query        | `src/query.rs`   | Follow BFS patterns                       |
| HelixDB operations     | `src/storage.rs` | `GotStorage` struct                       |
| Add bio content        | `data/*.md`      | Filename must match person ID             |
| Modify search behavior | `src/storage.rs` | `search_semantic()` method                |
| Modify embedding text  | `src/loader.rs`  | `PersonBio::composite_text()` method      |

## Code Map

| Symbol                         | Location   | Role                                                |
| ------------------------------ | ---------- | --------------------------------------------------- |
| `Person`                       | types.rs   | Node data structure                                 |
| `House`                        | types.rs   | Enum: Stark, Targaryen, Baratheon, Tully, Lannister |
| `RelationType`                 | types.rs   | Edge labels: ParentOf, SpouseOf, SiblingOf          |
| `SearchResult`                 | types.rs   | Semantic search result with score                   |
| `GotStorage`                   | storage.rs | HelixDB wrapper with CRUD + vector operations       |
| `insert_person_with_embedding` | storage.rs | Insert node with vector embedding                   |
| `search_semantic`              | storage.rs | HNSW vector search                                  |
| `FamilyTree`                   | loader.rs  | YAML deserialization struct                         |
| `BioLoader`                    | loader.rs  | Markdown bio file loader                            |
| `PersonBio`                    | loader.rs  | Bio content with composite text generation          |
| `find_ancestors`               | query.rs   | BFS traversal following PARENT_OF inward            |
| `find_descendants`             | query.rs   | BFS traversal following PARENT_OF outward           |
| `GotError`                     | error.rs   | Error types with exit codes                         |

## HelixDB Patterns

### Creating Nodes

```rust
let props: Vec<(&str, Value)> = vec![
    (arena.alloc_str("id"), Value::String(person.id.clone())),
    (arena.alloc_str("name"), Value::String(person.name.clone())),
    // ...
];
let properties = ImmutablePropertiesMap::new(props.len(), props.into_iter(), &arena);
let node = Node { id: node_id, label, version: 1, properties: Some(properties) };
storage.nodes_db.put(&mut wtxn, key, &node.to_bincode_bytes()?)?;
```

### Creating Edges

```rust
let edge = Edge { id, label, version: 1, from_node, to_node, properties: None };
storage.edges_db.put(&mut wtxn, key, &edge.to_bincode_bytes()?)?;
// Also update adjacency lists:
storage.out_edges_db.put(&mut wtxn, &out_key, &out_val)?;
storage.in_edges_db.put(&mut wtxn, &in_key, &in_val)?;
```

### Traversing Edges

```rust
// Get parents (incoming PARENT_OF edges)
let in_key = HelixGraphStorage::in_edge_key(&node_id, &label_hash);
let iter = storage.in_edges_db.prefix_iter(&rtxn, &in_key)?;

// Get children (outgoing PARENT_OF edges)
let out_key = HelixGraphStorage::out_edge_key(&node_id, &label_hash);
let iter = storage.out_edges_db.prefix_iter(&rtxn, &out_key)?;
```

### Vector Search Patterns

#### Inserting Vectors

```rust
use helix_db::helix_engine::vector_core::hnsw::HNSW;

// Convert f32 embeddings to f64 for HNSW
let embedding_f64: Vec<f64> = embedding.iter().map(|&x| f64::from(x)).collect();

// Insert vector with label
let vector = storage.vectors.insert::<fn(...) -> bool>(
    &mut wtxn, label, &embedding_f64, None, &arena
)?;
let vector_id = vector.id;

// Store vector_id in node properties for lookup
props.push((arena.alloc_str("vector_id"), Value::String(vector_id.to_string())));
```

#### Searching Vectors

```rust
// Search returns HVector results with distance
let vector_results = storage.vectors.search::<fn(...) -> bool>(
    &rtxn, &query_f64, limit, label, None, false, &arena
)?;

for hvector in vector_results {
    let vector_id = hvector.id;
    let distance = hvector.get_distance() as f32;
    // Convert distance to similarity (higher = more similar)
    let score = 1.0 / (1.0 + distance);

    // Lookup node by vector_id secondary index
    let node_id = lookup_by_vector_id(&rtxn, vector_id)?;
}
```

#### Bio File Format

Bio files in `data/*.md` are plain markdown. Filename must match person ID:

```
data/jon-snow.md     -> person_id: "jon-snow"
data/aerys-ii.md     -> person_id: "aerys-ii"
```

Content is combined with person metadata for embedding:

```
{name} ({alias})
Titles: {title1}, {title2}

{bio content from markdown file}
```

## Data Files

**Family tree**: `data/westeros.yaml`

- 30 people across 5 houses
- ~20 relationship definitions expanding to 88 edges
- Relationships: parent_of, spouse_of, sibling_of

**Character biographies**: `data/*.md` (30 files)

- One markdown file per character
- Filename matches person ID (e.g., `jon-snow.md`)
- Used for generating embeddings for semantic search

## Commands

```bash
# Build
cargo build -p demo-got

# Run checks
cargo fmt -p demo-got -- --check
cargo clippy -p demo-got -- -D warnings
cargo test -p demo-got

# Run CLI
cargo run -p demo-got -- ingest                    # Ingest with embeddings
cargo run -p demo-got -- ingest --skip-embeddings  # Fast ingest (no vectors)
cargo run -p demo-got -- ingest --clear            # Clear and re-ingest
cargo run -p demo-got -- search "the mad king"     # Semantic search
cargo run -p demo-got -- query ancestors jon-snow  # Graph traversal
cargo run -p demo-got -- stats
```

## Anti-Patterns

| Don't                           | Do Instead                                                      |
| ------------------------------- | --------------------------------------------------------------- |
| Store booleans as `Value::Bool` | Use `Value::String("true"/"false")` - HelixDB doesn't have Bool |
| Forget bidirectional edges      | For SPOUSE_OF/SIBLING_OF, create edges in both directions       |
| Skip secondary indices          | Use them for efficient lookups by id/house/vector_id            |
| Use f32 directly with HNSW      | Convert to f64: `embedding.iter().map(\|&x\| f64::from(x))`     |
| Store vectors without node link | Always store `vector_id` property in node for reverse lookup    |
| Forget HNSW trait import        | Add `use helix_db::helix_engine::vector_core::hnsw::HNSW;`      |

## Dependencies

Key crates from workspace:

- `helix-db` - Graph database with vector support
- `helix-graph-ops` - Shared HelixDB graph helpers
- `helix-embeddings` - Local embedding model (BGE-small-en-v1.5)
- `clap` - CLI parsing
- `serde` / `serde_yaml` - YAML deserialization
- `bumpalo` - Arena allocator for HelixDB
- `heed3` - LMDB bindings
