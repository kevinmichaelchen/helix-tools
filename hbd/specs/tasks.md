# Implementation Tasks

This document breaks down the `hbd` implementation into phases with specific, trackable tasks.

## Timeline Overview

| Phase | Focus                            | Duration  | Dependencies |
| ----- | -------------------------------- | --------- | ------------ |
| 1     | Core Infrastructure + Labels     | 2.5 weeks | None         |
| 2     | Dependencies & Graph             | 1.5 weeks | Phase 1      |
| 3     | Search & Vectors                 | 2 weeks   | Phase 1      |
| 4     | Sync & Daemon                    | 1 week    | Phases 1-3   |
| 5     | AI Agent Features                | 2 weeks   | Phase 4      |
| 6     | Health & Analytics               | 1.5 weeks | Phase 1      |
| 7     | Polish, Maintenance & CLI Parity | 1.5 weeks | Phases 1-6   |

**Total: ~12 weeks**

### New Features Summary (vs Beads Compatibility)

| Feature             | Priority | Phase | Notes                            |
| ------------------- | -------- | ----- | -------------------------------- |
| Label management    | High     | 1     | `label add/remove/list/list-all` |
| Cycle detection     | High     | 2     | `dep cycles` command             |
| Stale detection     | High     | 6     | `stale --days N`                 |
| System info         | High     | 7     | `info` command                   |
| Comment commands    | High     | 1     | `comment`, `comments`            |
| Graph visualization | Medium   | 2     | `graph <id>` DOT output          |
| Issue count         | Medium   | 6     | `count` with filters             |
| Merge duplicates    | Medium   | 7     | `merge --into`                   |
| Restore compacted   | Medium   | 7     | `restore <id>`                   |
| Admin cleanup       | Medium   | 7     | `admin cleanup --older-than`     |

---

## Implementation Status

> **Last Updated:** 2026-01-03
>
> This document tracks planned implementation phases. Checkboxes reflect actual completion status.
>
> **Legend:** ✅ = Complete | 🚧 = In Progress | ⏳ = Blocked | ❌ = Cut from scope

---

## Phase 1: Core Infrastructure (Weeks 1-2)

### 1.1 Project Setup

- [x] **T1.1.1** Create Rust workspace structure
  ```
  hbd/
  ├── Cargo.toml
  └── src/
      ├── main.rs
      ├── lib.rs
      ├── cli/           # Command definitions
      ├── db/            # HelixDB interactions
      ├── sync/          # Git sync logic
      ├── embed/         # Embedding integration
      └── types/         # Shared types
  ```

- [x] **T1.1.2** Configure Cargo.toml with workspace dependencies
  - clap for CLI
  - tokio for async
  - serde for serialization
  - gix for git operations (switched from git2)
  - fastembed for embeddings

- [x] **T1.1.3** Set up error handling with thiserror
  - Define HbdError enum
  - Implement exit codes
  - Create error formatting helpers

- [x] **T1.1.4** Set up logging with tracing
  - Console output for CLI
  - File output for daemon
  - Structured JSON logging option

### 1.2 Schema Implementation

- [x] **T1.2.1** Write `helix.toml` schema file
  - Issue node with all fields
  - Comment node
  - User, Label, Project, Team nodes
  - IssueEmbedding vector node
  - All edge definitions

- [x] **T1.2.2** Create Rust types matching schema
  ```rust
  pub struct Issue { ... }
  pub struct Comment { ... }
  pub enum Status { Open, InProgress, Blocked, Closed }  // Note: simplified from spec
  pub enum IssueType { Bug, Feature, Task, Epic, Chore }  // Note: Gate deferred
  ```

- [x] **T1.2.3** Implement serialization
  - ~~HelixDB protocol serialization~~ (deferred - using file-based storage)
  - YAML frontmatter serialization
  - JSON output serialization

### 1.3 Git-First File Storage

- [x] **T1.3.1** Implement Markdown parser
  - Parse YAML frontmatter with gray_matter
  - Extract body content
  - Handle malformed files gracefully

- [x] **T1.3.2** Implement Markdown writer
  - Serialize Issue to YAML frontmatter
  - Format body with proper Markdown
  - Preserve custom sections

- [x] **T1.3.3** Implement hash-based ID generation
  ```rust
  fn generate_id() -> String {
      let uuid = Uuid::new_v4();
      let hash = blake3::hash(uuid.as_bytes());
      format!("bd-{}", &hex::encode(&hash.as_bytes()[..3]))
  }
  ```

- [x] **T1.3.4** Implement content hashing for sync
  - Hash title + body for change detection
  - Store hash in Issue struct
  - Compare on sync

### 1.4 Basic CRUD Commands

- [x] **T1.4.1** Implement `hbd init`
  - Create `.ixchel/issues/` directory
  - Create `.ixchel/config.toml` with defaults
  - ~~Create `helix.toml` with schema~~ (helix.toml exists but not auto-created by init)
  - Add `.ixchel/data/` to `.gitignore`
  - Abort if already initialized

- [x] **T1.4.2** Implement `hbd create`
  - Parse arguments: title, --description, --type, --priority, --labels, --assignee
  - Generate hash-based ID
  - Write `.ixchel/issues/bd-xxxx.md`
  - ~~Insert Issue node in HelixDB~~ (deferred - file-only for now)
  - Support --json output
  - Support --agent flag

- [x] **T1.4.3** Implement `hbd show`
  - Load issue by ID
  - Display formatted output
  - Include labels, comments, dependencies
  - Support --json output

- [x] **T1.4.4** Implement `hbd list`
  - Support filters: --status, --type, --priority, --label, --assignee
  - ~~--project filter~~ (removed - will re-add when projects implemented)
  - ~~--include-ephemeral~~ (removed - ephemeral requires HelixDB)
  - Default: exclude closed issues unless --status explicitly provided (AC-003.1)
  - Sort by priority then created_at
  - Table output with columns: ID, Title, Status, Priority, Assignee
  - Support --json output

- [x] **T1.4.5** Implement `hbd update`
  - Update specified fields
  - Set updated_at timestamp
  - Write to file ~~and DB~~
  - ~~Queue re-embedding if title/body changed~~ (deferred)

- [x] **T1.4.6** Implement `hbd close`
  - Set status to closed
  - Set closed_at timestamp
  - Add closing comment with reason
  - Warn if open children exist, require --force to proceed (AC-005.3)

- [x] **T1.4.7** Implement `hbd reopen`
  - Set status to open
  - Clear closed_at
  - ~~Add reopening comment~~ (status change only)

### 1.5 Comment Support

- [x] **T1.5.1** Implement `hbd comment <id> "message"`
  - ~~Add Comment node~~ (stored in issue file, not separate node)
  - ~~Create COMMENT_ON edge~~ (deferred - file-only)
  - Update issue's updated_at
  - Append to Markdown file
  - Support --agent flag for agent comments

- [x] **T1.5.2** Implement `hbd comments <id>`
  - List all comments for an issue
  - Show author and timestamp
  - Support --json output

### 1.6 Label Management

- [x] **T1.6.1** Implement `hbd label add <id> <label>`
  - ~~Create Label node if not exists~~ (labels stored inline in issue)
  - ~~Create TAGGED edge~~ (deferred - file-only)
  - Support comma-separated labels: `label1,label2,label3` (AC-005B.4)
  - Idempotent: silently succeeds if label already exists (AC-005B.3)
  - Update Markdown frontmatter
  - Support --json output

- [x] **T1.6.2** Implement `hbd label remove <id> <label>`
  - ~~Remove TAGGED edge~~ (file-only)
  - Update Markdown frontmatter
  - Warn (don't error) if label not present (AC-005C.2)

- [x] **T1.6.3** Implement `hbd label list <id>`
  - Show all labels on issue
  - Support --json output

- [x] **T1.6.4** Implement `hbd label list-all`
  - Show all labels in project
  - Include usage count per label
  - Support --json output

---

## Phase 2: Dependencies & Graph (Week 3)

### 2.1 Dependency Management

- [x] **T2.1.1** Implement `hbd dep add <from> <type> <to>`
  - Types: blocks, related, waits_for ~~, duplicate_of~~ (cut)
  - ~~Create DEPENDS_ON edge with properties~~ (stored in issue file)
  - Update ~~both issue~~ blocked issue Markdown file
  - Prevent self-referential deps

- [x] **T2.1.2** Implement `hbd dep remove <from> <type> <to>`
  - ~~Remove DEPENDS_ON edge~~ (file-only)
  - Update Markdown files

- [x] **T2.1.3** Implement `hbd dep list <id>`
  - Show outgoing dependencies (this blocks X)
  - Show incoming dependencies (blocked by Y)
  - ~~Group by dependency type~~ (flat list)

### 2.2 Cycle Detection

- [x] **T2.2.1** Implement cycle detection algorithm
  - BFS from target to source
  - If path exists, adding edge would create cycle
  - Return cycle path for error message

- [x] **T2.2.2** Integrate cycle check into `hbd dep add`
  - Check before creating edge
  - Reject with error and cycle path display
  - ~~Exit code 4~~ (uses general error exit code)

- [x] **T2.2.3** Implement `hbd dep cycles`
  - Find all cycles in the dependency graph
  - Display each cycle path
  - Support --json output

### 2.3 Ready/Blocked Queries

- [x] **T2.3.1** Implement `hbd ready`
  - Find issues with no open blockers
  - Sort by priority then age
  - ~~--project filter~~ (removed - will re-add when projects implemented)

- [x] **T2.3.2** Implement `hbd blocked`
  - Find issues with open blockers
  - Show each blocker with status/assignee
  - ~~--project filter~~ (removed - will re-add when projects implemented)

- [x] **T2.3.3** Implement `hbd explain <id>`
  - Display full dependency tree
  - Show transitive blockers
  - ~~Highlight critical path~~ (shows tree only)

### 2.4 Path Algorithms

- [ ] **T2.4.1** Implement `hbd critical-path <epic-id>`
  - Use Dijkstra with priority*estimate weights
  - Find longest path to any leaf
  - Display path with time estimates

- [ ] **T2.4.2** Implement `hbd graph <id>`
  - Generate DOT format dependency graph
  - Support --output for file export
  - Color nodes by status (green=closed, yellow=in_progress, red=blocked, gray=open)
  - Support --depth to limit traversal (default: 5)
  - Support --json for machine-readable graph data

---

## Phase 3: Search & Vectors (Weeks 4-5)

> **Status:** Not started. Requires HelixDB integration.

### 3.1 BM25 Text Search

- [ ] **T3.1.1** Configure BM25 index on Issue
  - Index title with weight 2.0
  - Index body with weight 1.0

- [ ] **T3.1.2** Implement `hbd search <query>`
  - Execute BM25 search
  - Display results with scores
  - Support --limit

### 3.2 Embedding Setup

- [ ] **T3.2.1** Create embedder module
  - Wrap fastembed initialization
  - Handle model download on first use
  - Support model selection via config

- [ ] **T3.2.2** Implement embedding caching
  - Store text_hash with embedding
  - Skip re-embedding if hash unchanged

- [ ] **T3.2.3** Implement fallback chain
  - fastembed → model2vec → BM25-only
  - Log warnings on fallback

### 3.3 Vector Search

- [ ] **T3.3.1** Implement embedding on issue create/update
  - Async embedding in background
  - Create IssueEmbedding node
  - Create HAS_EMBEDDING edge

- [ ] **T3.3.2** Implement `hbd similar <id>`
  - Find similar by vector distance
  - Apply MMR for diversity
  - Exclude source issue

- [ ] **T3.3.3** Implement `hbd search <query> --semantic`
  - Embed query text
  - Search by vector similarity
  - Display similarity scores

### 3.4 Hybrid Search

- [ ] **T3.4.1** Implement RRF fusion
  - Execute BM25 and vector in parallel
  - Fuse with RRF (k=60)

- [ ] **T3.4.2** Implement `hbd search <query> --hybrid`
  - Use fused results
  - Apply MMR diversity reranking
  - Support filter flags

### 3.5 Duplicate Detection

- [ ] **T3.5.1** Implement duplicate check on create
  - Find issues with similarity > 0.85
  - Display as warnings
  - Don't block creation

- [ ] **T3.5.2** Add `--no-duplicate-check` flag
  - Skip similarity search
  - Faster creation for bulk imports

---

## Phase 4: Sync & Daemon (Week 6)

> **Status:** Not started. CLI commands defined but not implemented.

### 4.1 File Watcher

- [ ] **T4.1.1** Implement file watcher with notify
  - Watch `.ixchel/issues/` directory
  - Debounce events (5 seconds)
  - Queue sync on change

### 4.2 Sync Command

- [ ] **T4.2.1** Implement `hbd sync`
  - Export dirty DB changes to files
  - Import new/changed files to DB
  - Use content hash for change detection
  - Queue re-embedding for text changes

- [ ] **T4.2.2** Implement `hbd sync --import-only`
  - Only import from files
  - Don't export DB changes

- [ ] **T4.2.3** Implement `hbd sync --export-only`
  - Only export to files
  - Don't import file changes

### 4.3 Auto-Commit

- [ ] **T4.3.1** Implement auto-commit on sync
  - Check config.sync.auto_commit
  - Stage `.ixchel/issues/` changes
  - Commit with template message

### 4.4 Daemon Mode

- [ ] **T4.4.1** Implement `hbd daemon start`
  - Start background process
  - Create PID file
  - Initialize file watcher
  - Start sync loop

- [ ] **T4.4.2** Implement `hbd daemon stop`
  - Send SIGTERM to daemon
  - Clean up PID file

- [ ] **T4.4.3** Implement `hbd daemon status`
  - Check if daemon running
  - Display uptime and sync stats

- [ ] **T4.4.4** Implement RPC socket for CLI→daemon
  - Unix socket at `.ixchel/hbd.sock`
  - Commands route through daemon if available
  - Fall back to direct DB if daemon unavailable

### 4.5 Multi-Repo Support

- [ ] **T4.5.1** Store source_repo on issues
  - Default to current git remote
  - Allow override on create

- [ ] **T4.5.2** Support cross-repo dependency syntax
  - `hbd dep add bd-xxx blocks other-repo:bd-yyy`
  - Display with repo prefix

---

## Phase 5: AI Agent Features (Weeks 7-8)

> **Status:** Basic agent tracking implemented. Advanced features not started.

### 5.1 Agent Tracking

- [x] **T5.1.1** Implement --agent and --session flags
  - Set created_by_type="agent"
  - Set agent_id and session_id fields

- [ ] **T5.1.2** Implement `hbd list --created-by-agent`
  - Filter to agent-created issues

- [ ] **T5.1.3** Implement `hbd audit <id>`
  - Show all changes with actor info
  - Include agent/session info

### 5.2 Ephemeral Issues

> **Note:** Ephemeral issues require HelixDB for non-file storage. CLI flags removed until HelixDB integration is complete.

- [ ] **T5.2.1** Implement `hbd create --ephemeral`
  - Set ephemeral=true
  - Store in HelixDB only (not exported to .ixchel/issues/)
  - Requires: HelixDB integration

- [ ] **T5.2.2** Exclude ephemeral from list by default
  - Add --include-ephemeral flag
  - Requires: HelixDB integration

- [ ] **T5.2.3** Clean up old closed ephemeral on sync
  - Delete if closed > 24 hours
  - Configurable retention
  - Requires: HelixDB integration

### 5.3 Gate Coordination

- [ ] **T5.3.1** Implement gate issue type
  - Store await_condition field
  - Store await_status field

- [ ] **T5.3.2** Parse await condition types
  - `gh:pr:<number>` - GitHub PR
  - `gh:run:<id>` - GitHub Actions
  - `timer:<duration>` - Time delay
  - `human` - Manual approval
  - `issue:<id>:closed` - Issue dependency

- [ ] **T5.3.3** Implement `hbd wait <gate-id>`
  - Poll await condition
  - Block until satisfied or timeout
  - Update await_status on completion

- [ ] **T5.3.4** Implement condition checkers
  - GitHub API for pr/run (optional, requires token)
  - Timer with sleep
  - Human waits for `hbd approve <id>`
  - Issue checks status in DB

### 5.4 Context Compaction

- [ ] **T5.4.1** Implement compaction eligibility check
  - Closed > 30 days
  - Not already compacted
  - Has substantial content

- [ ] **T5.4.2** Implement `hbd compact`
  - Find eligible issues
  - Summarize with LLM (Claude/GPT)
  - Replace description with summary
  - Preserve original embedding
  - Set compaction_tier=1

- [ ] **T5.4.3** Implement `hbd compact --dry-run`
  - Report what would be compacted
  - Show size reduction estimates

### 5.5 Context Retrieval

- [ ] **T5.5.1** Implement `hbd context`
  - Curated project summary
  - Open issues by priority
  - Recent activity
  - Blocked items

- [ ] **T5.5.2** Implement `hbd context --query "topic"`
  - Hybrid search for relevant issues
  - Format for LLM context

- [ ] **T5.5.3** Implement `hbd context --limit <tokens>`
  - Estimate token count
  - Truncate to fit budget

---

## Phase 6: Health & Analytics (Week 9)

### 6.0 Stale Issue Detection

- [x] **T6.0.1** Implement `hbd stale`
  - Find issues not updated in N days (default: 14)
  - Support --days flag for custom threshold
  - Support --status filter
  - Support --limit flag
  - Sort by staleness (oldest first)
  - Support --json output

### 6.1 Health Metrics

- [ ] **T6.1.1** Implement `hbd health`
  - Total open/blocked/stale counts
  - Weekly velocity (closed per week)
  - Health grade calculation
  - Recommendations

- [ ] **T6.1.2** Implement `hbd health --label <name>`
  - Scoped metrics for label
  - Compare to project averages

### 6.2 Statistics

- [x] **T6.2.1** Implement `hbd stats`
  - Counts by status, type, priority
  - Weekly trends: created, closed, net change (AC-026.2)

- [x] **T6.2.2** Implement `hbd stats --json`
  - Machine-readable output
  - Include all aggregations

### 6.3 Quick Count

- [ ] **T6.3.1** Implement `hbd count`
  - Display total issue count
  - Support --status filter
  - Support --type filter
  - Support combining filters
  - Support --json output

---

## Phase 7: Polish & CLI Parity (Week 10)

### 7.0 Maintenance Commands

- [x] **T7.0.1** Implement `hbd info`
  - Display ~~database path~~ tickets directory
  - ~~Display issue prefix~~ (not yet)
  - ~~Display daemon status (running/stopped)~~ (not yet)
  - Display total issue count
  - ~~Display embedding model status~~ (not yet)
  - ~~Display last sync time~~ (not yet)
  - Support --json output

- [ ] **T7.0.2** Implement `hbd merge <source-ids...> --into <target-id>`
  - Transfer comments from sources to target
  - Transfer dependencies (incoming and outgoing)
  - Mark source issues as tombstone
  - Prevent merging closed into open
  - Support --dry-run flag
  - Support --json output

- [ ] **T7.0.3** Implement `hbd restore <id>`
  - Retrieve pre-compaction content from git history
  - Display original title, description, compaction date
  - Handle non-compacted issues gracefully
  - Support --json output

- [ ] **T7.0.4** Implement `hbd admin cleanup`
  - Find closed issues older than N days (default: 90)
  - Support --older-than flag
  - Support --dry-run flag
  - Require --force for actual deletion
  - Support --cascade to delete orphaned deps/comments
  - Prevent deleting issues with open dependents unless --cascade
  - Support --json output

### 7.1 Beads CLI Compatibility

- [ ] **T7.1.1** Ensure command name parity where sensible
  - `create`, `show`, `list`, `update`, `close`
  - `ready`, `blocked`, `dep`
  - `sync`, `search`

- [ ] **T7.1.2** Ensure --json flag on all commands
  - Consistent JSON schema
  - Include exit code in JSON

### 7.2 Shell Completion

- [ ] **T7.2.1** Generate shell completions
  - Bash
  - Zsh
  - Fish

- [ ] **T7.2.2** Implement `hbd completion <shell>`
  - Output completion script

### 7.3 Help & Documentation

- [ ] **T7.3.1** Write detailed --help for all commands
  - Examples for each command
  - Common flag documentation

- [ ] **T7.3.2** Write man pages (optional)
  - Generate from clap

### 7.4 Configuration

- [ ] **T7.4.1** Implement `hbd config show`
  - Display current configuration
  - Show defaults and overrides

- [ ] **T7.4.2** Implement `hbd config set <key> <value>`
  - Update config file
  - Validate values

- [ ] **T7.4.3** Support environment variable overrides
  - HBD_EMBEDDING_BACKEND
  - HBD_OPENAI_API_KEY
  - HBD_SYNC_INTERVAL

### 7.5 Testing

- [ ] **T7.5.1** Unit tests for core types
  - Issue serialization
  - ID generation
  - Content hashing

- [ ] **T7.5.2** Integration tests for commands
  - Create/read/update/delete flow
  - Dependency operations
  - Search operations

- [ ] **T7.5.3** HQL test cases
  - Add to hql-tests/ in helix-db repo
  - Cover all query patterns

### 7.6 Documentation

- [ ] **T7.6.1** Write README.md
  - Installation instructions
  - Quick start guide
  - Feature overview

- [ ] **T7.6.2** Write CLI reference
  - All commands with examples
  - Configuration options

- [ ] **T7.6.3** Write query cookbook
  - Common query patterns
  - HelixQL examples

---

## Definition of Done

Each task is complete when:

1. **Code written** - Implementation complete
2. **Tests pass** - Unit and integration tests
3. **Docs updated** - README, help text, comments
4. **Reviewed** - Self-review for quality
5. **Works offline** - No network dependencies (except first model download)
6. **--json works** - Machine-readable output supported

---

## Risk Mitigation

| Risk                           | Mitigation                               |
| ------------------------------ | ---------------------------------------- |
| fastembed model download fails | Fallback to model2vec, then BM25-only    |
| HelixDB API changes            | Pin to specific version, test on CI      |
| Large repos slow               | Incremental sync, batch operations       |
| Git merge conflicts            | Hash-based IDs, content-based resolution |
| Memory usage high              | Lazy loading, stream processing          |

---

## Success Metrics

| Metric               | Target                         |
| -------------------- | ------------------------------ |
| `hbd list` latency   | <100ms for 1000 issues         |
| `hbd search` latency | <500ms including embedding     |
| `hbd create` latency | <200ms (excluding async embed) |
| Memory usage         | <500MB for 10000 issues        |
| Test coverage        | >80% for core modules          |
| Offline capability   | 100% (after initial setup)     |
