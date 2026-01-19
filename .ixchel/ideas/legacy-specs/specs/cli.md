# CLI Design Specification

The `ixchel` command-line interface provides unified access to all entity types
and operations.

Note: this spec was drafted under the working name “Ixchel”. The implemented tool
is **Ixchel** (binary `ixchel`, canonical dir `.ixchel/`). Treat `ixchel` → `ixchel`
and `.ixchel/` → `.ixchel/` when reading. Some commands/flags described here are
planned and not implemented yet; use `ix-cli/` and `ix-*/specs/` as the current
source of truth.

## Design Principles

1. **Consistent Syntax**: Same patterns across all entity types
2. **Sensible Defaults**: Common operations require minimal flags
3. **Progressive Disclosure**: Simple commands, advanced options available
4. **Machine-Readable Output**: `--json` everywhere
5. **Partial ID Matching**: `dec-a1` instead of `dec-a1b2c3`

## Command Structure

```
ixchel <verb> [entity-type] [id] [options]
```

### Verbs

| Verb      | Description                   | Example                                |
| --------- | ----------------------------- | -------------------------------------- |
| `create`  | Create new entity             | `ixchel create decision "Use Redis"`   |
| `show`    | Display single entity         | `ixchel show dec-42`                   |
| `list`    | List entities with filters    | `ixchel list issues --status open`     |
| `update`  | Modify entity                 | `ixchel update iss-17 --status closed` |
| `delete`  | Remove entity                 | `ixchel delete idea-9`                 |
| `search`  | Semantic search               | `ixchel search "caching strategy"`     |
| `graph`   | Traverse relationships        | `ixchel graph dec-42 --depth 3`        |
| `link`    | Add relationship              | `ixchel link dec-42 spawns iss-17`     |
| `unlink`  | Remove relationship           | `ixchel unlink dec-42 spawns iss-17`   |
| `context` | Generate AI context           | `ixchel context iss-17`                |
| `sync`    | Synchronize files ↔ DB        | `ixchel sync`                          |
| `init`    | Initialize .ixchel/           | `ixchel init`                          |
| `check`   | Validate all entities         | `ixchel check`                         |
| `health`  | Knowledge/graph health report | `ixchel health`                        |
| `config`  | Manage configuration          | `ixchel config show`                   |

## Global Options

```
--json              Output as JSON
--quiet, -q         Suppress non-essential output
--verbose, -v       Increase verbosity
--directory, -d     Override .ixchel/ location
--agent <id>        Mark operation as agent-initiated
--session <id>      Group agent operations
--no-sync           Skip database synchronization
--color <when>      Color output: auto, always, never
--help, -h          Show help
--version, -V       Show version
```

---

## Entity Creation

### Generic Create

```bash
ixchel create <entity-type> "<title>" [options]
```

### Decision

```bash
ixchel create decision "Use PostgreSQL for primary storage" \
  --status proposed \
  --deciders kevin,alice \
  --tags database,infrastructure \
  --depends-on dec-8f9e0d \
  --cites src-pg2024 \
  --addresses idea-7x8y9z
```

**Options:**

```
--status <status>       proposed (default), accepted, rejected
--deciders <list>       Comma-separated decision makers
--tags <list>           Comma-separated tags
--depends-on <ids>      Decisions this depends on
--supersedes <ids>      Decisions this replaces
--amends <ids>          Decisions this modifies
--cites <ids>           Sources cited
--addresses <ids>       Issues/ideas this addresses
--body <text>           Body text (or use editor)
--edit, -e              Open in $EDITOR
```

### Issue

```bash
ixchel create issue "Implement connection pooling" \
  --type feature \
  --priority 1 \
  --assignee alice \
  --implements dec-a1b2c3 \
  --depends-on iss-x1y2z3
```

**Options:**

```
--type <type>           bug, feature, task (default), epic, chore
--priority <0-4>        0=critical, 4=backlog, default=2
--assignee <name>       Assigned person
--parent <id>           Parent epic/task
--implements <ids>      Decisions this implements
--spawned-by <id>       Decision that created this
--blocks <ids>          Issues this blocks
--depends-on <ids>      Issues this depends on
--estimate <minutes>    Time estimate
--tags <list>           Comma-separated tags
--edit, -e              Open in $EDITOR
```

### Idea

```bash
ixchel create idea "What if we used WebAssembly for plugins?" \
  --champion bob \
  --effort high \
  --impact high \
  --inspired-by src-wasm
```

**Options:**

```
--status <status>       draft (default), proposed, parked
--champion <name>       Advocate for this idea
--effort <level>        low, medium, high, unknown
--impact <level>        low, medium, high, unknown
--inspired-by <ids>     Sources/ideas that sparked this
--tags <list>           Comma-separated tags
--edit, -e              Open in $EDITOR
```

### Report

```bash
ixchel create report "Q4 Performance Retrospective" \
  --type retrospective \
  --period-start 2025-10-01 \
  --period-end 2025-12-31 \
  --summarizes iss-a1,iss-a2,dec-42
```

**Options:**

```
--type <type>           postmortem, rfc, retrospective, analysis, research
--status <status>       draft (default), published
--period-start <date>   Start date (for retrospectives)
--period-end <date>     End date
--incident-date <date>  Incident timestamp (for postmortems)
--summarizes <ids>      Entities this summarizes
--cites <ids>           Sources cited
--recommends <ids>      Decisions/ideas proposed
--edit, -e              Open in $EDITOR
```

### Source

```bash
ixchel create source "Redis: An In-Memory Data Structure Store" \
  --type paper \
  --url https://example.com/redis.pdf \
  --authors "Salvatore Sanfilippo" \
  --published 2019-06-30 \
  --publisher "SIGMOD 2019"
```

**Options:**

```
--type <type>           paper, article, documentation, book, talk, video, repository
--url <url>             Original location
--authors <list>        Comma-separated authors
--published <date>      Publication date
--publisher <name>      Journal, blog, conference
--doi <doi>             DOI identifier
--isbn <isbn>           ISBN for books
--archived <url>        Archive.org URL
--local <path>          Local file path
--edit, -e              Open in $EDITOR
```

### Citation

```bash
ixchel create citation "Redis single-threaded design rationale" \
  --from src-e5f6g7 \
  --page "Section 3.2" \
  --quote "The single-threaded nature of Redis is not a limitation..." \
  --supports dec-42
```

**Options:**

```
--from <id>             Source being cited (required)
--quote <text>          The quoted text
--page <ref>            Page number or section
--timestamp <time>      For videos (e.g., "14:32")
--paraphrase            Mark as paraphrased
--supports <ids>        Decisions this supports
--contradicts <ids>     Decisions this contradicts
--edit, -e              Open in $EDITOR
```

---

## Entity Display

### Show Single Entity

```bash
ixchel show <id>
ixchel show dec-a1b2c3
ixchel show dec-a1       # Partial ID matching
```

**Options:**

```
--format <fmt>          full (default), summary, yaml, json
--relationships         Include relationship details
--no-body               Omit body text
```

**Output (full):**

```
┌─────────────────────────────────────────────────────────────┐
│ dec-a1b2c3: Use PostgreSQL for Primary Storage              │
├─────────────────────────────────────────────────────────────┤
│ Status: accepted                  Priority: -               │
│ Type: decision                    Date: 2026-01-15          │
│ Created: 2026-01-10 14:30         By: kevin (human)         │
│ Updated: 2026-01-15 09:00                                   │
│ Tags: database, infrastructure                              │
│ Deciders: kevin, alice, bob                                 │
├─────────────────────────────────────────────────────────────┤
│ Relationships:                                              │
│   depends_on: dec-8f9e0d (Use managed infrastructure)       │
│   cites: src-pg2024 (PostgreSQL 16 Documentation)           │
│   addresses: idea-7x8y9z (Should we use a SQL database?)    │
│   spawns: iss-b2c3d4 (Implement connection pooling)         │
├─────────────────────────────────────────────────────────────┤
│ ## Context                                                  │
│                                                             │
│ We need a primary database for the application...           │
│                                                             │
│ ## Decision                                                 │
│                                                             │
│ We will use PostgreSQL 16 with...                           │
└─────────────────────────────────────────────────────────────┘
```

---

## Entity Listing

### List All of a Type

```bash
ixchel list decisions
ixchel list issues
ixchel list ideas
ixchel list reports
ixchel list sources
ixchel list citations
ixchel list                # All types
```

### Common Filters

```
--status <list>         Filter by status
--tags <list>           Filter by tags (all must match)
--created-by <name>     Filter by creator
--created-after <date>  After date
--created-before <date> Before date
--limit <n>             Max results (default: 50)
--offset <n>            Skip first n results
--sort <field>          Sort by: created, updated, title, priority
--order <dir>           asc or desc
```

### Type-Specific Filters

**Issues:**

```
--type <list>           bug, feature, task, epic, chore
--priority <list>       0, 1, 2, 3, 4
--assignee <name>       Assigned to
--open                  Shortcut for --status open,in_progress,blocked
--closed                Shortcut for --status closed
--blocked               Only blocked issues
--ready                 Only ready (unblocked open) issues
```

**Decisions:**

```
--proposed              Shortcut for --status proposed
--accepted              Shortcut for --status accepted
--superseded            Shortcut for --status superseded
```

### Output Formats

```bash
ixchel list issues                      # Table (default)
ixchel list issues --json               # JSON array
ixchel list issues --format oneline     # One line per entity
ixchel list issues --format ids         # IDs only
```

**Table Output:**

```
┌────────────┬──────────────────────────────┬────────┬──────────┬─────────┐
│ ID         │ Title                        │ Status │ Priority │ Updated │
├────────────┼──────────────────────────────┼────────┼──────────┼─────────┤
│ iss-b2c3d4 │ Implement connection pooling │ open   │ 1 (high) │ 2h ago  │
│ iss-e5f6g7 │ Fix memory leak in parser    │ open   │ 0 (crit) │ 1d ago  │
│ iss-h8i9j0 │ Update documentation         │ closed │ 3 (low)  │ 3d ago  │
└────────────┴──────────────────────────────┴────────┴──────────┴─────────┘
```

---

## Semantic Search

### Basic Search

```bash
ixchel search "database performance optimization"
```

### Filtered Search

```bash
ixchel search "caching" \
  --types decision,issue \
  --status accepted,open \
  --tags infrastructure \
  --limit 20
```

**Options:**

```
--types <list>          Entity types to search (default: all)
--status <list>         Filter by status
--tags <list>           Filter by tags
--created-by <name>     Filter by creator
--created-after <date>  After date
--limit <n>             Max results (default: 10)
--threshold <0-1>       Minimum similarity score
--explain               Show why results matched
```

**Output:**

```
Search: "database performance optimization"
Found 5 results (0.12s)

┌───────┬────────────┬────────────────────────────────────┬─────────┐
│ Score │ ID         │ Title                              │ Type    │
├───────┼────────────┼────────────────────────────────────┼─────────┤
│ 0.89  │ dec-a1b2c3 │ Use PostgreSQL for Primary Storage │ decision│
│ 0.82  │ iss-b2c3d4 │ Implement connection pooling       │ issue   │
│ 0.78  │ src-pg2024 │ PostgreSQL 16 Performance Guide    │ source  │
│ 0.71  │ rpt-d4e5f6 │ Q4 Database Performance Retro      │ report  │
│ 0.65  │ idea-x1y2  │ Use read replicas for analytics    │ idea    │
└───────┴────────────┴────────────────────────────────────┴─────────┘
```

---

## Graph Traversal

### View Relationships

```bash
ixchel graph dec-42
ixchel graph dec-42 --depth 3
ixchel graph dec-42 --direction incoming
ixchel graph dec-42 --types spawns,implements
```

**Options:**

```
--depth <n>             Max traversal depth (default: 2)
--direction <dir>       outgoing (default), incoming, both
--types <list>          Relationship types to follow
--format <fmt>          tree (default), dot, json
--entity-types <list>   Only include these entity types
```

**Tree Output:**

```
dec-a1b2c3: Use PostgreSQL for Primary Storage
├── spawns
│   ├── iss-b2c3d4: Implement connection pooling
│   │   └── depends_on
│   │       └── iss-x1y2z3: Set up dev database
│   └── iss-c4d5e6: Configure backup strategy
├── depends_on
│   └── dec-8f9e0d: Use managed infrastructure
└── cites
    └── src-pg2024: PostgreSQL 16 Documentation
```

**DOT Output (for Graphviz):**

```bash
ixchel graph dec-42 --format dot | dot -Tpng -o graph.png
```

---

## Relationship Management

### Add Relationship

```bash
ixchel link <from-id> <relationship> <to-id>
ixchel link dec-42 spawns iss-17
ixchel link iss-17 implements dec-42
ixchel link rpt-99 summarizes iss-1,iss-2,iss-3
```

### Remove Relationship

```bash
ixchel unlink <from-id> <relationship> <to-id>
ixchel unlink dec-42 spawns iss-17
```

### Valid Relationships by Entity Type

| From     | Relationship                   | To               |
| -------- | ------------------------------ | ---------------- |
| Decision | supersedes, amends, depends_on | Decision         |
| Decision | spawns                         | Issue            |
| Decision | addresses                      | Issue, Idea      |
| Decision | cites                          | Source           |
| Issue    | blocks, depends_on             | Issue            |
| Issue    | implements                     | Decision         |
| Issue    | spawned_by                     | Decision         |
| Idea     | evolves_into                   | Decision, Issue  |
| Idea     | inspired_by                    | Source, Idea     |
| Report   | summarizes                     | Issue, Decision  |
| Report   | cites                          | Source           |
| Report   | recommends                     | Decision, Idea   |
| Citation | from_source                    | Source           |
| Citation | supports, contradicts          | Decision         |
| Citation | used_in                        | Report, Decision |
| Any      | relates_to                     | Any              |

---

## Entity Updates

### Update Properties

```bash
ixchel update <id> [options]
ixchel update iss-17 --status closed --closed-reason "Completed"
ixchel update dec-42 --status accepted
ixchel update idea-9 --status evolved --evolves-into dec-99
```

**Common Options:**

```
--title <text>          Change title
--status <status>       Change status
--tags <list>           Replace tags
--add-tags <list>       Add tags
--remove-tags <list>    Remove tags
--edit, -e              Open in $EDITOR
```

**Issue-Specific:**

```
--assignee <name>       Change assignee
--priority <0-4>        Change priority
--type <type>           Change type
--closed-reason <text>  Reason for closing
```

**Decision-Specific:**

```
--deciders <list>       Change deciders
```

### Immutability Enforcement

```bash
$ ixchel update dec-42 --title "New title"
Error: Cannot modify 'title' of accepted decision dec-42.
Hint: Accepted decisions are immutable. Create a new decision that supersedes this one.
```

---

## AI Context Generation

### Generate Context for AI Assistant

```bash
ixchel context <id>
ixchel context iss-17 --depth 2 --max-tokens 8000
```

**Options:**

```
--depth <n>             Relationship depth to include (default: 2)
--max-tokens <n>        Maximum tokens (default: from config)
--format <fmt>          markdown (default), xml, json
--include <types>       Entity types to include
--exclude <types>       Entity types to exclude
--no-relationships      Exclude related entities
```

**Output:**

```markdown
# Context for iss-b2c3d4: Implement connection pooling

## This Issue

**Status:** open | **Priority:** high | **Type:** feature
**Assignee:** alice | **Created:** 2026-01-16 by kevin

### Description

Implement PgBouncer-style connection pooling for the database layer...

## Implements Decision

### dec-a1b2c3: Use PostgreSQL for Primary Storage (accepted)

We decided to use PostgreSQL 16 for primary storage because...

## Blocking Issues

### iss-x1y2z3: Set up dev database (open)

Development database environment needs to be configured first...

## Relevant Sources

### src-pg2024: PostgreSQL 16 Documentation

Official documentation on connection management...
```

---

## Maintenance Commands

### Initialize

```bash
ixchel init
ixchel init --with-hooks      # Also install git hooks
```

Creates:

- `.ixchel/config.toml`
- `.ixchel/decisions/`
- `.ixchel/issues/`
- `.ixchel/ideas/`
- `.ixchel/reports/`
- `.ixchel/sources/`
- `.ixchel/citations/`
- Updates `.gitignore` with `.ixchel/data/`

### Validate

```bash
ixchel check
ixchel check --fix            # Auto-fix where possible
ixchel check decisions        # Only decisions
```

Checks:

- YAML frontmatter validity
- Required fields present
- Relationships point to existing entities
- No broken references
- ID format correctness
- Cycle detection

### Sync

```bash
ixchel sync
ixchel sync --full            # Rebuild entire index
ixchel sync --dry-run         # Show what would change
```

### Health Report

```bash
ixchel health
```

**Output:**

```
Knowledge Health Report
═══════════════════════════════════════════════════════════════

Entity Counts
─────────────
  Decisions: 42 (38 accepted, 3 proposed, 1 superseded)
  Issues: 127 (89 open, 38 closed)
  Ideas: 15 (8 draft, 4 proposed, 3 evolved)
  Reports: 8 (7 published, 1 draft)
  Sources: 34
  Citations: 67

Potential Issues
────────────────
  ⚠ 3 decisions have no supporting sources (dec-12, dec-17, dec-23)
  ⚠ 5 ideas have been in draft > 30 days
  ⚠ 2 issues blocked for > 60 days (iss-45, iss-78)
  ⚠ 12 sources have never been cited
  ⚠ 1 cycle detected in issue dependencies

Stale Entities (no update > 90 days)
────────────────────────────────────
  idea-3: Consider GraphQL for API
  dec-8: Use JWT for authentication
  iss-23: Update onboarding docs

Relationship Statistics
───────────────────────
  Total edges: 342
  Most connected: dec-42 (18 relationships)
  Orphaned entities: 3 (no incoming or outgoing relationships)
```

### Configuration

```bash
ixchel config show
ixchel config get embedding.model
ixchel config set embedding.model "BAAI/bge-base-en-v1.5"
ixchel config edit              # Open in $EDITOR
```

---

## Git Hooks

### Install Hooks

```bash
ixchel hooks install
```

Installs pre-commit hook that:

- Validates modified entity files
- Enforces decision immutability
- Checks for broken relationships

### Remove Hooks

```bash
ixchel hooks remove
```

### Bypass Hooks

```bash
git commit --no-verify
IXCHEL_SKIP_HOOKS=1 git commit
```

---

## Shell Completions

```bash
ixchel completions bash > ~/.local/share/bash-completion/completions/ixchel
ixchel completions zsh > ~/.zfunc/_ixchel
ixchel completions fish > ~/.config/fish/completions/ixchel.fish
```

---

## JSON Output Schema

All commands support `--json` for machine-readable output.

### Entity JSON

```json
{
  "id": "dec-a1b2c3",
  "entity_type": "decision",
  "title": "Use PostgreSQL for Primary Storage",
  "status": "accepted",
  "metadata": {
    "created_at": "2026-01-10T14:30:00Z",
    "updated_at": "2026-01-15T09:00:00Z",
    "created_by": "kevin",
    "created_by_type": "human",
    "tags": ["database", "infrastructure"],
    "uuid": "550e8400-e29b-41d4-a716-446655440000"
  },
  "relationships": [
    {"type": "depends_on", "target_id": "dec-8f9e0d"},
    {"type": "cites", "target_id": "src-pg2024"},
    {"type": "spawns", "target_id": "iss-b2c3d4"}
  ],
  "properties": {
    "date": "2026-01-15",
    "deciders": ["kevin", "alice", "bob"]
  },
  "body": "## Context\n\nWe need a primary database..."
}
```

### Search Result JSON

```json
{
  "query": "database performance",
  "total": 5,
  "elapsed_ms": 120,
  "results": [
    {
      "id": "dec-a1b2c3",
      "entity_type": "decision",
      "title": "Use PostgreSQL for Primary Storage",
      "score": 0.89,
      "snippet": "...database performance is critical..."
    }
  ]
}
```

### Error JSON

```json
{
  "error": {
    "code": "ENTITY_NOT_FOUND",
    "message": "Entity not found: dec-99999",
    "details": {
      "id": "dec-99999",
      "entity_type": "decision"
    }
  }
}
```
