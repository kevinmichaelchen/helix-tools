# AGENTS.md - demo-got

AI agent instructions for the Game of Thrones family tree HelixDB demo.

## Overview

`demo-got` is a demonstration crate showing how to use HelixDB as an embedded graph database. It stores ~30 Game of Thrones characters as nodes and their family relationships as edges.

## Quick Reference

| Task                  | Location         | Notes                                     |
| --------------------- | ---------------- | ----------------------------------------- |
| Add CLI command       | `src/main.rs`    | Clap derive, `Commands` enum              |
| Add person property   | `src/types.rs`   | Update `Person` struct, then `storage.rs` |
| Add relationship type | `src/types.rs`   | Add to `RelationType` enum                |
| Modify YAML schema    | `src/loader.rs`  | Update `RelationshipDef` enum             |
| Add graph query       | `src/query.rs`   | Follow BFS patterns                       |
| HelixDB operations    | `src/storage.rs` | `GotStorage` struct                       |

## Code Map

| Symbol             | Location   | Role                                                |
| ------------------ | ---------- | --------------------------------------------------- |
| `Person`           | types.rs   | Node data structure                                 |
| `House`            | types.rs   | Enum: Stark, Targaryen, Baratheon, Tully, Lannister |
| `RelationType`     | types.rs   | Edge labels: ParentOf, SpouseOf, SiblingOf          |
| `GotStorage`       | storage.rs | HelixDB wrapper with CRUD operations                |
| `FamilyTree`       | loader.rs  | YAML deserialization struct                         |
| `find_ancestors`   | query.rs   | BFS traversal following PARENT_OF inward            |
| `find_descendants` | query.rs   | BFS traversal following PARENT_OF outward           |
| `GotError`         | error.rs   | Error types with exit codes                         |

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

## Data File

Family tree data is in `data/westeros.yaml`:

- **30 people** across 5 houses
- **~20 relationship definitions** expanding to 88 edges
- Relationships: parent_of, spouse_of, sibling_of

## Commands

```bash
# Build
cargo build -p demo-got

# Run checks
cargo fmt -p demo-got -- --check
cargo clippy -p demo-got -- -D warnings
cargo test -p demo-got

# Run CLI
cargo run -p demo-got -- ingest
cargo run -p demo-got -- query ancestors jon-snow
cargo run -p demo-got -- stats
```

## Anti-Patterns

| Don't                           | Do Instead                                                      |
| ------------------------------- | --------------------------------------------------------------- |
| Store booleans as `Value::Bool` | Use `Value::String("true"/"false")` - HelixDB doesn't have Bool |
| Forget bidirectional edges      | For SPOUSE_OF/SIBLING_OF, create edges in both directions       |
| Skip secondary indices          | Use them for efficient lookups by id/house                      |

## Dependencies

Key crates from workspace:

- `helix-db` - Graph database
- `clap` - CLI parsing
- `serde` / `serde_yaml` - YAML deserialization
- `bumpalo` - Arena allocator for HelixDB
- `heed3` - LMDB bindings
