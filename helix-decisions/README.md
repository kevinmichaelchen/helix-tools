# helix-decisions

Decision graph infrastructure with semantic search, backed by embedded HelixDB.

**Status:** Scaffolded  
**Created:** 2026-01-05

## Why helix-decisions?

Decisions are the backbone of software architecture. Unlike code (which shows *what*), decisions capture *why*:
- "What have we decided about caching?"
- "Why did we choose X over Y?"
- "Are we already committed to this direction?"

But `.decisions/` directories are unindexed markdown. helix-decisions makes them **searchable in < 100ms** via semantic indexing and tracks relationships between decisions.

## Core Idea

```bash
# First invocation: index all decisions into HelixDB
helix-decisions search "database migration" 

# Result: Ranked decisions with scores and metadata
# [
#   { id: 3, title: "...", status: "accepted", score: 0.87, ... },
#   { id: 1, title: "...", status: "proposed", score: 0.72, ... }
# ]

# Second invocation: fast delta + search
helix-decisions search "caching strategy"  # < 100ms (mostly search, minimal indexing)

# Follow supersedes chains
helix-decisions chain 3  # Shows decision lineage

# Find related decisions
helix-decisions related 3  # Shows all connected decisions
```

**Key:** Persistent HelixDB index, delta indexing on each call.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Search decisions (default: .decisions/)
helix-decisions search <QUERY> [OPTIONS]

# Options:
--directory <PATH>         # Decision directory (default: .decisions/)
--limit <N>                # Results limit (default: 10)
--status <STATUS>          # Filter by status (proposed|accepted|superseded|deprecated)
--tags <TAGS>              # Filter by tags (comma-separated)
--json                     # JSON output (default for piping)

# Follow supersedes chain
helix-decisions chain <ID>

# Find related decisions
helix-decisions related <ID>

# Examples:
helix-decisions search "caching"
helix-decisions --directory ./architecture search "database" --status accepted
helix-decisions search "performance" --limit 3 --json
helix-decisions chain 5
```

## Output

### Pretty (human-readable)
```
[1] 003: Database Migration Strategy
    Status: accepted
    Score: 0.89
    Tags: database, migration, infrastructure

[2] 001: Schema Versioning Approach
    Status: proposed
    Score: 0.71
    Tags: database, schema, testing
```

### JSON (machine-readable)
```json
{
  "query": "database migration",
  "results": [
    {
      "id": 3,
      "uuid": "hx-a1b2c3",
      "title": "Database Migration Strategy",
      "status": "accepted",
      "score": 0.89,
      "tags": ["database", "migration"],
      "file_path": ".decisions/003-database-migration-strategy.md"
    }
  ]
}
```

## Decision Format

Decisions are markdown files with YAML frontmatter in `.decisions/`:

```yaml
---
id: 3
uuid: hx-a1b2c3  # Optional: hash-based UUID for distributed safety
title: Database Migration Strategy
status: accepted
date: 2026-01-04
deciders:
  - Alice
  - Bob
tags:
  - database
  - migration
content_hash: abc123...  # Optional: for immutability proof
git_commit: def456...    # Optional: commit when accepted
supersedes: 1            # Optional: decision this replaces
depends_on: [2, 4]       # Optional: prerequisite decisions
related_to: 5            # Optional: related decisions
---

# Context and Problem Statement
...

# Decision
...
```

## ID Scheme

- **`id`**: Local sequential integer (1, 2, 3...) for human readability
- **`uuid`**: Optional hash-based UUID (via helix-id) for distributed safety across branches

## How It Works

1. **First invocation:** Scan `.decisions/`, embed with fastembed, store in HelixDB (~2-5s)
2. **Subsequent invocations:** Delta check (file hashes), re-index only changed decisions, search (~100ms)

## Architecture

```
User/Agent
    │
    ↓
┌─────────────────────────────────┐
│   helix-decisions CLI           │
└──────────┬──────────────────────┘
           │
    ┌──────▼──────────────────────┐
    │ Embedded HelixDB             │
    │ • Vector index               │
    │ • Graph relationships        │
    │ • Persistent (~/.helix/)     │
    └──────────────────────────────┘
```

## Specs

See `specs/` directory:
- [requirements.md](specs/requirements.md) - User stories, acceptance criteria
- [design.md](specs/design.md) - Architecture, data model
- [tasks.md](specs/tasks.md) - Implementation phases

## License

MIT
