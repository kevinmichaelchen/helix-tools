# helix-id

Hash-based ID generation for helix-tools. Provides collision-resistant, deterministic identifiers suitable for distributed/offline-first systems.

## Why

Git-backed tools like `hbd` and `helix-docs` need IDs that:
- Are **collision-resistant** across branches and machines
- Are **short** for human readability (6-8 chars)
- Are **deterministic** when derived from natural keys
- Have **prefixes** for type identification at a glance

## Design

All IDs follow the same format: `{prefix}-{hex}` (e.g., `bd-a1b2c3`)

The library provides three generation strategies—you choose based on your entity's semantics:

| Strategy | Input | Output | Use Case |
|----------|-------|--------|----------|
| `from_key(key)` | Natural key string | Deterministic | Entities with unique identifiers |
| `from_parts(parts)` | Multiple key parts | Deterministic | Hierarchical/composite identities |
| `random()` | None | Unique each call | User-created content, duplicates OK |

All strategies hash their input with Blake3 and take the first N bytes as hex (default: 3 bytes = 6 chars).

## Usage

```rust
use helix_id::define_id;

// Define strongly-typed IDs
define_id!(SourceId, "src");
define_id!(DocId, "doc");
define_id!(IssueId, "bd");

// DETERMINISTIC: Same key → same ID (use for entities with natural keys)
let source = SourceId::from_key("facebook/react");  // Always "src-..." for this repo
let source2 = SourceId::from_key("facebook/react"); // Same ID!
assert_eq!(source, source2);

// COMPOSITE KEYS: For hierarchical entities
let doc = DocId::from_parts(&[source.as_str(), "docs/hooks.md"]);

// RANDOM: Each call produces unique ID (use when duplicates are allowed)
let issue = IssueId::random();  // "bd-a1b2c3"
let issue2 = IssueId::random(); // Different ID

// Wrap existing ID string
let parsed = IssueId::from_string("bd-a1b2c3");
```

## When to Use Each Strategy

| Strategy | Use When | Example |
|----------|----------|---------|
| `from_key()` | Entity has a natural unique identifier | `SourceId::from_key("owner/repo")` |
| `from_parts()` | Entity identity derives from parent + local ID | `DocId::from_parts(&[source_id, path])` |
| `random()` | No natural key, duplicates allowed | `IssueId::random()` for user-created issues |

See [specs/design.md][design] for detailed guidance.

## Features

- `serde` (default) - Serialize/deserialize support

## Consumers

| Crate | ID Types | Strategy |
|-------|----------|----------|
| `hbd` | `IssueId` (`bd-`) | `random()` - duplicates allowed |
| `helix-docs` | `SourceId` (`src-`) | `from_key()` - repos are unique |
| `helix-docs` | `DocId` (`doc-`) | `from_parts()` - doc within source |
| `helix-docs` | `ChunkId` (`chk-`) | `from_parts()` - chunk within doc |
| `helix-map` | `FileId` (`fil-`) | `from_key()` - paths are unique |
| `helix-map` | `SymbolId` (`sym-`) | `from_parts()` - symbol within file |

## Specifications

- [specs/requirements.md][requirements] - Requirements in EARS notation
- [specs/design.md][design] - Design decisions and rationale

<!-- Links -->
[requirements]: ./specs/requirements.md
[design]: ./specs/design.md
