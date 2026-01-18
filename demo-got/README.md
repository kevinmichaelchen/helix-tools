# demo-got

A Game of Thrones family tree demo showcasing HelixDB's graph and vector search capabilities.

## Overview

This crate demonstrates how to use HelixDB as an embedded graph database for storing and querying family relationships. It ingests ~30 characters from Game of Thrones (Houses Stark, Targaryen, Baratheon, Tully, Lannister) and supports both graph traversal queries and semantic search over character biographies.

## Installation

```bash
cargo build -p demo-got
```

## Usage

### Ingest Data

Load the family tree from YAML into HelixDB with embeddings:

```bash
cargo run -p demo-got -- ingest
```

This creates a `.data/` directory inside the crate with the persisted graph data. By default, it also:
- Loads character biographies from `data/*.md` files
- Generates embeddings using the local embedding model
- Stores vectors for semantic search

For faster iteration during development, skip embedding generation:

```bash
cargo run -p demo-got -- ingest --skip-embeddings
```

To clear existing data and re-ingest:

```bash
cargo run -p demo-got -- ingest --clear
```

### Query Commands

**Find ancestors** (follows PARENT_OF edges in reverse):

```bash
cargo run -p demo-got -- query ancestors jon-snow
```

Output reveals Jon's true parentage (R+L=J):

```
Ancestors of jon-snow:
  Lyanna Stark "The She-Wolf" (House Stark)
  Rhaegar Targaryen "The Last Dragon" (House Targaryen)
    Rickard Stark (House Stark)
    Aerys II Targaryen "The Mad King" (House Targaryen)
    ...
```

**Find descendants** (follows PARENT_OF edges forward):

```bash
cargo run -p demo-got -- query descendants ned-stark
```

**List house members:**

```bash
cargo run -p demo-got -- query house stark
```

**Get person details with immediate family:**

```bash
cargo run -p demo-got -- query person ned-stark
```

**Show database statistics:**

```bash
cargo run -p demo-got -- stats
```

### Semantic Search

Search for characters using natural language queries:

```bash
cargo run -p demo-got -- search "the mad king"
```

Output:

```
Search results for: "the mad king"

1. Aerys II Targaryen "The Mad King" (House Targaryen) - score: 0.847
2. Viserys Targaryen (House Targaryen) - score: 0.623
3. Daenerys Targaryen "Mother of Dragons" (House Targaryen) - score: 0.598
```

More examples:

```bash
# Find dragon-related characters
cargo run -p demo-got -- search "mother of dragons"

# Find characters by their story arcs
cargo run -p demo-got -- search "characters who died tragically"

# Limit results
cargo run -p demo-got -- search "dragon riders" --limit 3
```

### JSON Output

Add `--json` for machine-readable output:

```bash
cargo run -p demo-got -- --json query ancestors jon-snow
```

## Data Model

### Nodes (PERSON)

| Property  | Type   | Description                              |
| --------- | ------ | ---------------------------------------- |
| id        | String | Unique identifier (e.g., "jon-snow")     |
| name      | String | Full name (e.g., "Jon Snow")             |
| house     | String | House affiliation                        |
| titles    | JSON   | Array of titles held                     |
| alias     | String | Common nickname                          |
| is_alive  | String | "true" or "false"                        |
| vector_id | String | Links to embedding vector (if generated) |

### Edges

| Label      | Direction      | Description                |
| ---------- | -------------- | -------------------------- |
| PARENT_OF  | Parent → Child | Biological/adoptive parent |
| SPOUSE_OF  | Bidirectional  | Marriage relationship      |
| SIBLING_OF | Bidirectional  | Sibling relationship       |

## Architecture

```
demo-got/
├── src/
│   ├── main.rs      # CLI entry point (clap)
│   ├── lib.rs       # Module exports
│   ├── types.rs     # Person, House, RelationType, SearchResult
│   ├── error.rs     # GotError enum
│   ├── loader.rs    # YAML + bio markdown parsing
│   ├── storage.rs   # HelixDB wrapper with vector support
│   └── query.rs     # BFS graph traversal
└── data/
    ├── westeros.yaml   # Family tree seed data
    └── *.md            # Character biographies (30 files)
```

### Vector Search

Semantic search uses:
- **HNSW index**: `m=16`, `ef_construction=128`, `ef_search=64`
- **Embedding model**: `BAAI/bge-small-en-v1.5` (384 dimensions)
- **Composite text**: `{name} ({alias})\nTitles: {titles}\n\n{bio}`

## License

MIT
