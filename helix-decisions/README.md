# helix-decisions

Decision graph infrastructure with semantic search and persistent indexing.

**Status:** Core Complete  
**Created:** 2026-01-05

## Why helix-decisions?

Decisions are the backbone of software architecture. Unlike code (which shows *what*), decisions capture *why*:
- "What have we decided about caching?"
- "Why did we choose X over Y?"
- "Are we already committed to this direction?"

But `.decisions/` directories are unindexed markdown. helix-decisions makes them **searchable in < 100ms** via semantic indexing and tracks relationships between decisions.

## Directory Scope

helix-decisions is **repo-scoped**, not global. Each repository has one `.decisions/` directory at its root. Monorepos have one `.decisions/` directory per repo.

**Discovery:** `helix-decisions` automatically finds `.decisions/` by walking up from your current directory to the git root.

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Create decisions directory
mkdir .decisions

# Create your first decision
cat > .decisions/001-initial-architecture.md << 'EOF'
---
id: 1
title: Initial Architecture
status: proposed
date: 2026-01-06
deciders:
  - Alice
tags:
  - architecture
---

# Context
...

# Decision
...
EOF

# Search decisions
helix-decisions search "architecture"

# Follow supersedes chains
helix-decisions chain 1

# Find related decisions
helix-decisions related 1

# Validate decision files
helix-decisions check
```

## Usage

```bash
# Search decisions (auto-discovers .decisions/)
helix-decisions search <QUERY> [OPTIONS]

# Options:
--directory <PATH>         # Override decision directory
--sync                     # Block until index is up to date
--limit <N>                # Results limit (default: 10)
--status <STATUS>          # Filter by status (proposed|accepted|superseded|deprecated)
--tags <TAGS>              # Filter by tags (comma-separated)
--json                     # JSON output

# Follow supersedes chain
helix-decisions chain <ID>

# Find related decisions
helix-decisions related <ID>

# Validate decision files (frontmatter + required fields)
helix-decisions check [--json]

# Install git hooks (manual, opt-in)
helix-decisions init-hooks

# Remove git hooks
helix-decisions remove-hooks
```

## Enforcing Immutability

Accepted decisions (status: accepted) should not be modified—instead, create new decisions with `amends: [id]` to reference the original.

To enforce this with git hooks:

```bash
helix-decisions init-hooks
```

This will install a pre-commit hook that blocks commits modifying accepted decisions.

**Bypass options:**
- `git commit --no-verify` — Skip hook for a single commit
- `HELIX_DECISIONS_SKIP_HOOKS=1` — Environment variable bypass
- Delete `.git/hooks/pre-commit` — Remove hook entirely

## Configuration

Global config at `~/.helix/config/helix-decisions.toml`:

```toml
strict = true  # Block modifications to accepted decisions (default: true)
```

Per-repo config at `.helix/helix-decisions.toml` overrides global.

## Decision Format

Decisions are markdown files with YAML frontmatter in `.decisions/`:

```yaml
---
id: 3
uuid: hx-a1b2c3  # Required: hash-based UUID for rename safety
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
amends: [2]              # Optional: decisions this amends
depends_on: [2, 4]       # Optional: prerequisite decisions
related_to: 5            # Optional: related decisions
---

# Context and Problem Statement
...

# Decision
...
```

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | integer | Local sequential ID (1, 2, 3...) |
| `uuid` | string | Hash-based UUID (hx-xxxxxx) for rename safety |
| `title` | string | Human-readable title |
| `status` | string | proposed, accepted, superseded, deprecated |
| `date` | date | ISO 8601 date |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `deciders` | list | People who made the decision |
| `tags` | list | Categorization tags |
| `content_hash` | string | SHA256 hash for immutability |
| `git_commit` | string | Git commit when accepted |
| `supersedes` | int/list | Decision(s) this replaces |
| `amends` | int/list | Decision(s) this amends |
| `depends_on` | int/list | Prerequisite decisions |
| `related_to` | int/list | Related decisions |

## Validation

`helix-decisions check` validates all `.md` files in `.decisions/` and fails if frontmatter is
missing or invalid, or if required fields like `uuid` are missing.

## ID Scheme

- **`id`**: Local sequential integer (1, 2, 3...) for human readability
- **`uuid`**: Required hash-based UUID (via helix-id) for distributed safety across branches

## How It Works

1. **First invocation:** Scan `.decisions/`, embed with fastembed, build LMDB index (sync, ~2-5s)
2. **Subsequent invocations:** Query existing LMDB immediately, then queue a background sync
3. **Strong consistency (optional):** `--sync` blocks until the index is up to date

Index is stored at `.helix/data/decisions/` within your repository (LMDB `data.mdb`,
`lock.mdb`).

## Consistency and Indexing

- **Single writer:** The global helixd daemon owns all write transactions to LMDB.
- **Background updates:** CLI enqueues a "scan + delta sync" request and returns results from
  the current index immediately.
- **`--sync` flag:** Blocks until the daemon finishes the pending sync (or runs a direct sync
  if no daemon is running).
- **Rename/delete detection:** The daemon updates the manifest and removes or renames entries
  without forcing a full re-embed.

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

## Architecture

```
User/Agent
    │
    ↓
┌─────────────────────────────────┐
│   helix-decisions CLI           │
│   • search, chain, related      │
│   • init-hooks, remove-hooks    │
└──────────┬──────────────────────┘
           │
    ┌──────▼──────────────────────┐
    │ Shared Infrastructure        │
    │ • helix-embeddings (search)  │
    │ • helix-db (LMDB)            │
    │ • helix-discovery (find)     │
    │ • helix-config (settings)    │
    └──────────────────────────────┘
```

## Specs

See `specs/` directory:
- [requirements.md](specs/requirements.md) - User stories, acceptance criteria
- [design.md](specs/design.md) - Architecture, data model
- [tasks.md](specs/tasks.md) - Implementation phases

## License

MIT
