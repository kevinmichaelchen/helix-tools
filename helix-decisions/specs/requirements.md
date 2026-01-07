# helix-decisions: Requirements Specification

**Document:** requirements.md  
**Status:** In Progress (2026-01-06)  
**Author:** Kevin Chen

> **Implementation Status:**  
> - **Phase 1-2:** Complete — HelixDB storage with fastembed  
> - **Phase 3 (Planned):** Incremental indexing + daemonized sync for sub-50ms delta sync

## Vision

helix-decisions is **general decision graph infrastructure** with semantic search. Unlike ADR-specific tools, it treats decisions as first-class graph nodes with relationships, enabling powerful queries about architectural evolution.

## User Stories

### For Agents
```
As an AI agent building context for a task,
I want to search for relevant decisions quickly,
So that I can understand the design constraints and avoid conflicts.
```

**Acceptance Criteria:**
- `helix-decisions search "database migration" --json` returns ranked results in < 100ms
- Output includes: id, uuid, title, status, score, tags, date, deciders
- Semantic search (not just keyword matching)
- Works offline (no external API calls)

### For Developers
```
As a developer,
I want to quickly find past architectural decisions from the terminal,
So that I can reference the reasoning behind design choices.
```

**Acceptance Criteria:**
- `helix-decisions search "caching"` returns human-readable results
- Can filter by status: `helix-decisions search "caching" --status accepted`
- First run indexes all decisions (~2-5s), subsequent runs are fast
- Works in any repo with `.decisions/` directory

### For Decision Lineage
```
As an architect,
I want to trace how decisions evolved over time,
So that I can understand why current decisions exist.
```

**Acceptance Criteria:**
- `helix-decisions chain 5` shows supersedes chain from decision 5
- `helix-decisions related 3` shows all connected decisions
- Graph traversal is fast (< 50ms)

### For CI/Automation
```
As a CI pipeline,
I want to verify that relevant decisions exist before deployment,
So that deployments align with architectural decisions.
```

**Acceptance Criteria:**
- `helix-decisions search "deployment strategy" --limit 1` exits 0 if found, 1 if not
- Works with shell pipelines

## Functional Requirements

### FR-1: Load Decisions from Filesystem
- **Input:** Directory path (default: `.decisions/`)
- **Output:** Parsed Decision objects with metadata
- **Requirements:**
  - Support markdown files: `NNN-title-kebab-case.md`
  - Parse YAML frontmatter (id, uuid, title, status, date, deciders, tags, relationships)
  - Extract body text for embedding
  - Handle missing/malformed files gracefully

### FR-2: Index into HelixDB
- **Input:** Parsed decisions
- **Output:** Persistent vector index in project-local `.helix/data/decisions/`
- **Requirements:**
  - Embed each decision body using fastembed (CPU)
  - Store embeddings + metadata in HelixDB
  - Create graph edges for relationships
  - Create index on first run (~2-5s for 100 decisions)
  - HelixDB persists across invocations
  - Reuse persisted embeddings when content hash and model are unchanged

### FR-3: Delta Indexing
- **Input:** Current decisions directory
- **Output:** Updated HelixDB index
- **Requirements:**
  - Track file hashes to detect changes
  - Re-index only changed/new decisions
  - Remove deleted decisions from index
  - Detect renames via content hash + uuid and update path without re-embed
  - If uuid is missing, treat rename as delete + add
  - Execute in < 100ms for typical case (no changes)

### FR-4: Semantic Search
- **Input:** Query string
- **Output:** Ranked decisions with scores (0.0-1.0)
- **Requirements:**
  - Embed query using same model as decisions
  - Search HelixDB for nearest neighbors
  - Return top-K results sorted by score
  - Execute in < 50ms

### FR-5: Metadata Filtering
- **Input:** Status and/or tags
- **Output:** Filtered ranked results
- **Requirements:**
  - Filter by status: `--status accepted`
  - Filter by tags: `--tags database,api`
  - Combine filters with AND logic

### FR-6: Graph Traversal
- **Input:** Decision ID
- **Output:** Related decisions via graph edges
- **Requirements:**
  - `chain <id>`: Follow supersedes edges to find evolution
  - `related <id>`: Find all connected decisions (1-hop)
  - Return relationship type with each result

### FR-7: Output Formatting
- **Input:** Search/query results
- **Output:** Formatted text (pretty or JSON)
- **Requirements:**
  - Pretty: Human-readable with colors
  - JSON: Machine-readable with full metadata
  - Default: Pretty if terminal, JSON if piped

### FR-8: CLI Interface
- **Requirements:**
  - Commands: `search`, `chain`, `related`
  - Global options: `--directory`, `--json`
  - Search options: `--limit`, `--status`, `--tags`
  - Help: `helix-decisions --help`

### FR-11: Validation (Lint/Check)
- **Input:** Decisions directory
- **Output:** Validation report + exit code
- **Requirements:**
  - `helix-decisions check` SHALL fail when frontmatter is missing or invalid
  - `helix-decisions check` SHALL fail when `uuid` is missing
  - Git hooks MAY invoke `helix-decisions check` to enforce uuid presence

### FR-9: Incremental Indexing (Phase 3)
- **Input:** Git working tree state + manifest of indexed decisions
- **Output:** Minimal set of re-indexing operations
- **Requirements:**
  - **Stage 1 - Git Detection:** Use `gix` to detect changed files since last indexed commit
  - **Stage 2 - Manifest Comparison:** Compare file content hashes against stored manifest
  - **Stage 3 - Vector Sync:** Re-embed only changed content, tombstone deleted decisions
  - Skip re-embedding when only metadata changed (frontmatter without body change)
  - Track `git_commit` of last full index for baseline comparison
  - Handle force-pushes and rebases gracefully (fall back to full manifest scan)

### FR-10: Indexer Daemon and Consistency
- **Input:** CLI invocation + repo path
- **Output:** Immediate query results + queued background sync
- **Requirements:**
  - A global per-user daemon owns all HelixDB write transactions (single writer)
  - Requests are namespaced by `{repo_root, tool}` to keep data scoped per repo
  - CLI enqueues a "scan + delta sync" request on each invocation
  - CLI uses existing index immediately (eventual consistency)
  - IPC uses local sockets (Unix domain sockets + Windows named pipes)
  - Unix socket path: `~/.helix/run/helixd.sock`
  - Protocol v1 is defined in `shared/helix-daemon/specs/design.md`
  - Provide `--sync` to block until the pending sync completes
  - If the daemon is not running, `--sync` runs a direct sync with a writer lock

## Non-Functional Requirements

### Performance

#### Phase 1-2 — HelixDB Storage
| Operation | Target | Notes |
|-----------|--------|-------|
| First search | 2-5s | Full indexing |
| Subsequent search | < 200ms | File hash comparison + search |
| Delta sync (no changes) | ~100ms | Full file scan for hash comparison |
| Delta sync (1 file changed) | ~500ms | Re-embed + LMDB write |
| Query embedding | 50-100ms | fastembed |
| Graph traversal | < 50ms | In-memory |

#### Phase 3 — Native HelixDB
| Operation | Target | Improvement | Notes |
|-----------|--------|-------------|-------|
| First search | 2-5s | Same | Embedding is bottleneck |
| Subsequent search | < 100ms | 2x faster | Native HNSW |
| Delta sync (no changes) | < 50ms | 2x faster | Git-based detection |
| Delta sync (1 file changed) | < 100ms | 5x faster | Incremental graph update |
| Query embedding | 50-100ms | Same | fastembed |
| HelixDB search | < 20ms | 2.5x faster | Native vector index |
| Graph traversal | < 20ms | 2.5x faster | Native edge traversal |

**Phase 3 Key Improvements:**
- **Git-first detection:** Use `gix` to skip unchanged files before hashing
- **Incremental updates:** Update only changed nodes/edges, not full rewrite
- **Native HNSW:** HelixDB's built-in vector index

### Reliability
- Handle missing `.decisions/` gracefully
- Handle malformed YAML (skip with warning)
- Handle deleted decisions (remove from index)
- Prevent concurrent writers across multiple CLI invocations

### Compatibility
- Linux, macOS, Windows
- Works offline
- Embedded HelixDB (no separate setup)

## Data Model

### DecisionMetadata (Required)
```yaml
id: integer           # Local sequential (1, 2, 3...)
uuid: string          # Required for rename optimization; enforced by check/hook
title: string
status: enum          # proposed | accepted | superseded | deprecated
date: date            # YYYY-MM-DD
deciders: [string]
tags: [string]
content_hash: string  # Optional: for immutability proof
git_commit: string    # Optional: commit when accepted
```

### Relationships (Optional)
```yaml
supersedes: integer | [integer]     # Decisions this replaces
amends: integer | [integer]         # Decisions this modifies
depends_on: integer | [integer]     # Prerequisite decisions
related_to: integer | [integer]     # Loosely related decisions
```

### SearchResult
```json
{
  "id": 3,
  "uuid": "hx-a1b2c3",
  "title": "Database Migration Strategy",
  "status": "accepted",
  "score": 0.89,
  "tags": ["database", "migration"],
  "date": "2026-01-04",
  "deciders": ["Alice", "Bob"],
  "file_path": ".decisions/003-database-migration-strategy.md"
}
```

## ID Scheme

### Local ID (`id`)
- Sequential integer (1, 2, 3...)
- Human-readable and easy to reference
- Unique within a single repository

### Global UUID (`uuid`)
- Hash-based identifier via helix-id
- Format: `hx-xxxxxx` (6 hex chars from Blake3 hash)
- Safe for distributed collaboration across branches
- Generated from decision content or UUID
- Required for rename optimization; lint/check should enforce presence

## Immutability Model

Decisions become immutable once accepted:
- `content_hash`: SHA-256 of decision content at acceptance
- `git_commit`: Git commit hash when status changed to accepted
- Amendments create new decisions with `amends` relationship
- Superseding creates new decisions with `supersedes` relationship

## CLI Specification

```
helix-decisions <COMMAND> [OPTIONS]

COMMANDS:
  search <QUERY>    Search decisions semantically
  chain <ID>        Show supersedes chain
  related <ID>      Find related decisions
  check             Validate decision frontmatter

GLOBAL OPTIONS:
  -d, --directory <PATH>   Decision directory (default: .decisions/)
  -j, --json               JSON output
  -h, --help               Show help
  -V, --version            Show version

SEARCH OPTIONS:
  -l, --limit <N>          Max results (default: 10)
  --status <STATUS>        Filter by status
  --tags <TAGS>            Filter by tags (comma-separated)
```

### Exit Codes
- 0: Success (results found)
- 1: No results found
- 2: Error

## Out of Scope

- Decision creation (use templates + $EDITOR)
- Decision editing (use $EDITOR)
- Format validation (CI concern)
- Git operations (users commit)
- Approval workflows (separate tool)
- Auto-generation of UUIDs (manual or via init command)
