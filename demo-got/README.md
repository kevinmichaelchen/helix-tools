# demo-got

A Game of Thrones family tree demo showcasing HelixDB's graph capabilities.

## Overview

This crate demonstrates how to use HelixDB as an embedded graph database for storing and querying family relationships. It ingests ~30 characters from Game of Thrones (Houses Stark, Targaryen, Baratheon, Tully, Lannister) and supports graph traversal queries.

## Installation

```bash
cargo build -p demo-got
```

## Usage

### Ingest Data

Load the family tree from YAML into HelixDB:

```bash
cargo run -p demo-got -- ingest
```

This creates a `.data/` directory inside the crate with the persisted graph data.

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

### JSON Output

Add `--json` for machine-readable output:

```bash
cargo run -p demo-got -- --json query ancestors jon-snow
```

## Data Model

### Nodes (PERSON)

| Property | Type   | Description                          |
| -------- | ------ | ------------------------------------ |
| id       | String | Unique identifier (e.g., "jon-snow") |
| name     | String | Full name (e.g., "Jon Snow")         |
| house    | String | House affiliation                    |
| titles   | JSON   | Array of titles held                 |
| alias    | String | Common nickname                      |
| is_alive | String | "true" or "false"                    |

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
│   ├── types.rs     # Person, House, RelationType
│   ├── error.rs     # GotError enum
│   ├── loader.rs    # YAML parsing
│   ├── storage.rs   # HelixDB wrapper
│   └── query.rs     # BFS graph traversal
└── data/
    └── westeros.yaml  # Family tree seed data
```

## License

MIT
