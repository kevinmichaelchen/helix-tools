# Requirements

This document defines user stories and acceptance criteria for `hbd` using [EARS notation](https://www.iaria.org/conferences2015/filesICCGI15/EARS.pdf) (Easy Approach to Requirements Syntax).

> **Implementation Notes (2026-01-03)**
>
> The current implementation has made the following simplifications from this spec:
>
> | Spec | Implementation |
> |------|----------------|
> | 6 statuses (open, in_progress, blocked, deferred, closed, tombstone) | 4 statuses (open, in_progress, blocked, closed) |
> | 6 issue types (bug, feature, task, epic, chore, gate) | 5 types (bug, feature, task, epic, chore) |
> | 4 dependency types (blocks, related, waits_for, duplicate_of) | 3 types (blocks, related, waits_for) |
> | HelixDB for storage and queries | File-based storage only |
> | Async embedding generation | Not implemented |
> | helixd daemon with file watching | Not implemented |
>
> These requirements remain the target specification. Implementation will align as features are added.

## EARS Notation Reference

| Pattern | Template |
|---------|----------|
| Ubiquitous | THE SYSTEM SHALL `<action>` |
| Event-driven | WHEN `<trigger>` THE SYSTEM SHALL `<action>` |
| State-driven | WHILE `<state>` THE SYSTEM SHALL `<action>` |
| Optional | WHERE `<feature>` THE SYSTEM SHALL `<action>` |
| Complex | IF `<condition>` THEN THE SYSTEM SHALL `<action>` |

---

## 1. Core Issue Management

### US-001: Create Issue

**As a** developer  
**I want to** create an issue with title, description, and metadata  
**So that** I can track work items in my project

| ID | Acceptance Criterion |
|----|---------------------|
| AC-001.1 | WHEN a user runs `hbd create "Title" --description "..." --type bug` THE SYSTEM SHALL create a Markdown file in `.tickets/` with YAML frontmatter containing all specified fields |
| AC-001.2 | WHEN the issue is created THE SYSTEM SHALL generate a unique hash-based ID (e.g., `bd-a1b2c3`) using the first 6 characters of a UUID v4 hash |
| AC-001.3 | WHEN the issue is created THE SYSTEM SHALL insert an Issue node into HelixDB with all properties indexed |
| AC-001.4 | WHEN the issue is created THE SYSTEM SHALL asynchronously generate an embedding vector and store it as an IssueEmbedding node |
| AC-001.5 | WHEN `--json` flag is provided THE SYSTEM SHALL output the created issue as JSON to stdout |

---

### US-002: View Issue

**As a** developer  
**I want to** view the full details of an issue  
**So that** I can understand the work required

| ID | Acceptance Criterion |
|----|---------------------|
| AC-002.1 | WHEN a user runs `hbd show <id>` THE SYSTEM SHALL display the issue title, description, metadata, comments, and dependency information |
| AC-002.2 | WHEN the issue has dependencies THE SYSTEM SHALL display blocking issues with their status |
| AC-002.3 | WHEN `--json` flag is provided THE SYSTEM SHALL output the issue as JSON including all related entities |
| AC-002.4 | IF the issue ID does not exist THEN THE SYSTEM SHALL display an error message and exit with code 1 |

---

### US-003: List Issues

**As a** developer  
**I want to** list issues with various filters  
**So that** I can find relevant work items

| ID | Acceptance Criterion |
|----|---------------------|
| AC-003.1 | WHEN a user runs `hbd list` THE SYSTEM SHALL display all non-closed issues sorted by priority then creation date |
| AC-003.2 | WHEN `--status <status>` is provided THE SYSTEM SHALL filter issues to only those matching the status |
| AC-003.3 | WHEN `--type <type>` is provided THE SYSTEM SHALL filter issues to only those matching the type |
| AC-003.4 | WHEN `--label <label>` is provided THE SYSTEM SHALL filter issues to only those tagged with the label |
| AC-003.5 | WHEN `--assignee <user>` is provided THE SYSTEM SHALL filter issues to only those assigned to the user |
| AC-003.6 | WHEN `--project <name>` is provided THE SYSTEM SHALL filter issues to only those in the project |
| AC-003.7 | THE SYSTEM SHALL support combining multiple filters with AND semantics |

---

### US-004: Update Issue

**As a** developer  
**I want to** modify issue properties  
**So that** I can keep issues accurate as work progresses

| ID | Acceptance Criterion |
|----|---------------------|
| AC-004.1 | WHEN a user runs `hbd update <id> --status in_progress` THE SYSTEM SHALL update the status field in both the Markdown file and HelixDB |
| AC-004.2 | WHEN any field is updated THE SYSTEM SHALL set `updated_at` to the current timestamp |
| AC-004.3 | WHEN the title or description is updated THE SYSTEM SHALL regenerate the embedding vector |
| AC-004.4 | THE SYSTEM SHALL support updating: status, priority, type, title, description, assignee, labels, estimated_minutes |

---

### US-005: Close Issue

**As a** developer  
**I want to** close an issue with a reason  
**So that** completed or abandoned work is tracked

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005.1 | WHEN a user runs `hbd close <id> --reason "Done"` THE SYSTEM SHALL set status to "closed" and record the reason as a comment |
| AC-005.2 | WHEN the issue is closed THE SYSTEM SHALL set `closed_at` to the current timestamp |
| AC-005.3 | WHEN the issue has open child issues (via PARENT_OF) THE SYSTEM SHALL warn the user and require `--force` to proceed |

---

## 1B. Label Management

### US-005B: Add Label

**As a** developer  
**I want to** add labels to issues  
**So that** I can categorize and filter work items

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005B.1 | WHEN a user runs `hbd label add <id> <label>` THE SYSTEM SHALL create a TAGGED edge from the issue to the label |
| AC-005B.2 | WHEN the label does not exist THE SYSTEM SHALL create it with default color |
| AC-005B.3 | WHEN the issue already has the label THE SYSTEM SHALL do nothing (idempotent) |
| AC-005B.4 | THE SYSTEM SHALL support adding multiple labels at once: `hbd label add <id> label1,label2` |
| AC-005B.5 | THE SYSTEM SHALL update the issue's Markdown frontmatter |

---

### US-005C: Remove Label

**As a** developer  
**I want to** remove labels from issues  
**So that** I can correct miscategorization

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005C.1 | WHEN a user runs `hbd label remove <id> <label>` THE SYSTEM SHALL remove the TAGGED edge |
| AC-005C.2 | WHEN the label is not on the issue THE SYSTEM SHALL display a warning and exit successfully |
| AC-005C.3 | THE SYSTEM SHALL update the issue's Markdown frontmatter |

---

### US-005D: List Labels

**As a** developer  
**I want to** see all labels on an issue or all labels in the project  
**So that** I can understand categorization

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005D.1 | WHEN a user runs `hbd label list <id>` THE SYSTEM SHALL display all labels on that issue |
| AC-005D.2 | WHEN a user runs `hbd label list-all` THE SYSTEM SHALL display all labels in the project with usage counts |
| AC-005D.3 | WHEN `--json` flag is provided THE SYSTEM SHALL output labels as JSON |

---

## 1C. Comment Management

### US-005E: Add Comment

**As a** developer  
**I want to** add comments to issues  
**So that** I can provide updates and context

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005E.1 | WHEN a user runs `hbd comment <id> "message"` THE SYSTEM SHALL create a Comment node linked to the issue |
| AC-005E.2 | THE SYSTEM SHALL set created_by to the current user or agent |
| AC-005E.3 | THE SYSTEM SHALL append the comment to the issue's Markdown file |
| AC-005E.4 | THE SYSTEM SHALL update the issue's `updated_at` timestamp |
| AC-005E.5 | WHEN `--json` flag is provided THE SYSTEM SHALL output the created comment as JSON |

---

### US-005F: List Comments

**As a** developer  
**I want to** view all comments on an issue  
**So that** I can see the discussion history

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005F.1 | WHEN a user runs `hbd comments <id>` THE SYSTEM SHALL display all comments in chronological order |
| AC-005F.2 | THE SYSTEM SHALL show author, timestamp, and content for each comment |
| AC-005F.3 | WHEN `--json` flag is provided THE SYSTEM SHALL output comments as JSON array |

---

## 2. Search & Discovery

### US-006: Text Search (BM25)

**As a** developer  
**I want to** search issues by keywords  
**So that** I can find issues mentioning specific terms

| ID | Acceptance Criterion |
|----|---------------------|
| AC-006.1 | WHEN a user runs `hbd search "memory leak"` THE SYSTEM SHALL return issues ranked by BM25 relevance across title and body fields |
| AC-006.2 | THE SYSTEM SHALL weight title matches 2x higher than body matches |
| AC-006.3 | WHEN `--limit N` is provided THE SYSTEM SHALL return at most N results (default: 20) |
| AC-006.4 | THE SYSTEM SHALL display relevance scores alongside results |

---

### US-007: Semantic Search (Vector)

**As a** developer  
**I want to** find semantically similar issues  
**So that** I can discover related work even with different wording

| ID | Acceptance Criterion |
|----|---------------------|
| AC-007.1 | WHEN a user runs `hbd similar <id>` THE SYSTEM SHALL return up to 10 issues ranked by vector similarity to the specified issue |
| AC-007.2 | WHEN a user runs `hbd search "query" --semantic` THE SYSTEM SHALL embed the query and search by vector similarity |
| AC-007.3 | THE SYSTEM SHALL use MMR reranking (lambda=0.7) to ensure diversity in results |
| AC-007.4 | THE SYSTEM SHALL display similarity scores (0.0-1.0) alongside results |

---

### US-008: Hybrid Search

**As a** developer  
**I want to** combine keyword and semantic search  
**So that** I get the best of both approaches

| ID | Acceptance Criterion |
|----|---------------------|
| AC-008.1 | WHEN a user runs `hbd search "query" --hybrid` THE SYSTEM SHALL execute both BM25 and vector searches |
| AC-008.2 | THE SYSTEM SHALL fuse results using Reciprocal Rank Fusion (RRF) with k=60 |
| AC-008.3 | THE SYSTEM SHALL apply MMR reranking (lambda=0.7) for diversity after fusion |
| AC-008.4 | WHEN search filters are provided THE SYSTEM SHALL apply them as post-filters to the fused results |

---

### US-009: Duplicate Detection

**As a** developer  
**I want** potential duplicates flagged when creating issues  
**So that** I avoid creating redundant work items

| ID | Acceptance Criterion |
|----|---------------------|
| AC-009.1 | WHEN an issue is created THE SYSTEM SHALL search for existing issues with similarity > 0.85 |
| AC-009.2 | WHEN potential duplicates are found THE SYSTEM SHALL display them with "Possible duplicates:" heading |
| AC-009.3 | THE SYSTEM SHALL NOT block creation, only warn |
| AC-009.4 | WHEN `--no-duplicate-check` is provided THE SYSTEM SHALL skip duplicate detection |

---

### US-009B: Stale Issue Detection

**As a** developer  
**I want to** find issues that haven't been updated recently  
**So that** I can identify forgotten or abandoned work

| ID | Acceptance Criterion |
|----|---------------------|
| AC-009B.1 | WHEN a user runs `hbd stale` THE SYSTEM SHALL return issues not updated in the last 14 days (default) |
| AC-009B.2 | WHEN `--days N` is provided THE SYSTEM SHALL use N days as the threshold |
| AC-009B.3 | THE SYSTEM SHALL only include open or in_progress issues by default |
| AC-009B.4 | WHEN `--status <status>` is provided THE SYSTEM SHALL filter to that status |
| AC-009B.5 | THE SYSTEM SHALL sort by staleness (oldest first) |
| AC-009B.6 | WHEN `--limit N` is provided THE SYSTEM SHALL return at most N results |

---

## 3. Dependencies & Graph

### US-010: Add Dependency

**As a** developer  
**I want to** declare dependencies between issues  
**So that** work order is clear

| ID | Acceptance Criterion |
|----|---------------------|
| AC-010.1 | WHEN a user runs `hbd dep add <from> blocks <to>` THE SYSTEM SHALL create a DEPENDS_ON edge with dep_type="blocks" |
| AC-010.2 | THE SYSTEM SHALL support dependency types: `blocks`, `related`, `waits_for`, `duplicate_of` |
| AC-010.3 | WHEN a dependency is added THE SYSTEM SHALL update both issue Markdown files |
| AC-010.4 | THE SYSTEM SHALL prevent self-referential dependencies |

---

### US-010B: Show Cycles

**As a** developer  
**I want to** see all circular dependencies in the project  
**So that** I can identify and resolve problematic dependency chains

| ID | Acceptance Criterion |
|----|---------------------|
| AC-010B.1 | WHEN a user runs `hbd dep cycles` THE SYSTEM SHALL find all circular dependency chains |
| AC-010B.2 | THE SYSTEM SHALL display each cycle with the full path (e.g., "A -> B -> C -> A") |
| AC-010B.3 | WHEN no cycles exist THE SYSTEM SHALL display "No cycles detected" |
| AC-010B.4 | WHEN `--json` flag is provided THE SYSTEM SHALL output cycles as a JSON array |

---

### US-011: Cycle Detection

**As a** developer  
**I want** the system to prevent circular dependencies  
**So that** I don't create unsolvable blockers

| ID | Acceptance Criterion |
|----|---------------------|
| AC-011.1 | WHEN a user runs `hbd dep add <from> blocks <to>` AND this would create a cycle THE SYSTEM SHALL reject the operation with exit code 1 |
| AC-011.2 | WHEN rejecting a cycle THE SYSTEM SHALL display the full cycle path (e.g., "Cycle detected: A -> B -> C -> A") |
| AC-011.3 | THE SYSTEM SHALL use BFS shortest path to detect cycles in O(V+E) time |

---

### US-012: Ready Issues

**As a** developer  
**I want to** find issues with no open blockers  
**So that** I can pick up work immediately

| ID | Acceptance Criterion |
|----|---------------------|
| AC-012.1 | WHEN a user runs `hbd ready` THE SYSTEM SHALL return all open issues that have no DEPENDS_ON edges to non-closed issues |
| AC-012.2 | THE SYSTEM SHALL sort results by priority (ascending) then age (descending) |
| AC-012.3 | WHEN `--project <name>` is provided THE SYSTEM SHALL filter to that project |

---

### US-013: Blocked Issues

**As a** developer  
**I want to** see which issues are blocked and why  
**So that** I can understand project bottlenecks

| ID | Acceptance Criterion |
|----|---------------------|
| AC-013.1 | WHEN a user runs `hbd blocked` THE SYSTEM SHALL return all issues with status="blocked" or with open blockers |
| AC-013.2 | THE SYSTEM SHALL display each blocking issue with its status and assignee |
| AC-013.3 | WHEN a user runs `hbd explain <id>` THE SYSTEM SHALL display the full dependency tree |

---

### US-013B: Dependency Graph Visualization

**As a** developer  
**I want to** visualize the dependency graph for an issue  
**So that** I can understand complex dependency relationships

| ID | Acceptance Criterion |
|----|---------------------|
| AC-013B.1 | WHEN a user runs `hbd graph <id>` THE SYSTEM SHALL generate a DOT format representation of the dependency graph |
| AC-013B.2 | THE SYSTEM SHALL include all transitive dependencies up to 5 levels deep (configurable) |
| AC-013B.3 | THE SYSTEM SHALL color nodes by status (green=closed, yellow=in_progress, red=blocked, gray=open) |
| AC-013B.4 | WHEN `--output <file>` is provided THE SYSTEM SHALL write the DOT output to the specified file |
| AC-013B.5 | WHEN `--json` flag is provided THE SYSTEM SHALL output the graph as JSON with nodes and edges arrays |

---

### US-014: Critical Path Analysis

**As a** project lead  
**I want to** find the longest dependency chain blocking an epic  
**So that** I can prioritize the most impactful work

| ID | Acceptance Criterion |
|----|---------------------|
| AC-014.1 | WHEN a user runs `hbd critical-path <epic-id>` THE SYSTEM SHALL return the longest weighted path from any leaf issue to the epic |
| AC-014.2 | THE SYSTEM SHALL calculate edge weights as: `(5 - priority) * estimated_minutes` (default estimated_minutes=60) |
| AC-014.3 | THE SYSTEM SHALL use Dijkstra's algorithm for path finding |
| AC-014.4 | THE SYSTEM SHALL display estimated time to completion based on summed estimates along the critical path |

---

## 4. Offline & Sync

### US-015: Offline Operation

**As a** developer working without internet  
**I want** full tracker functionality offline  
**So that** I can work from anywhere

| ID | Acceptance Criterion |
|----|---------------------|
| AC-015.1 | WHEN the system starts THE SYSTEM SHALL attempt to load `fastembed` with BGE-small-en-v1.5 model |
| AC-015.2 | WHEN the model is not cached THE SYSTEM SHALL download it on first use (requires one-time network) |
| AC-015.3 | WHEN fastembed is unavailable THE SYSTEM SHALL fall back to BM25-only search with a warning |
| AC-015.4 | WHEN cloud API keys are not set THE SYSTEM SHALL NOT attempt any network calls |

---

### US-016: Git Sync

**As a** developer  
**I want** issues to sync automatically via git  
**So that** team members see the same issues

| ID | Acceptance Criterion |
|----|---------------------|
| AC-016.1 | WHEN a user runs `hbd sync` THE SYSTEM SHALL export any dirty HelixDB changes to `.tickets/` Markdown files |
| AC-016.2 | WHEN a user runs `hbd sync` THE SYSTEM SHALL import any `.tickets/` changes not in HelixDB |
| AC-016.3 | THE SYSTEM SHALL use content hashing to detect changes (not timestamps) |
| AC-016.4 | WHEN helixd is running THE SYSTEM SHALL auto-sync every 5 seconds (debounced) |

---

### US-017: Conflict Resolution

**As a** developer  
**I want** predictable conflict handling  
**So that** git merges don't corrupt data

| ID | Acceptance Criterion |
|----|---------------------|
| AC-017.1 | THE SYSTEM SHALL use hash-based IDs (e.g., `bd-a1b2c3`) to prevent ID collisions across branches |
| AC-017.2 | WHEN the same issue is modified on two branches THE SYSTEM SHALL keep the version with the later `updated_at` |
| AC-017.3 | WHEN git merge conflicts occur in `.tickets/` files THE SYSTEM SHALL require manual resolution |
| AC-017.4 | WHEN importing after manual conflict resolution THE SYSTEM SHALL validate YAML frontmatter syntax |

---

### US-018: Multi-Repository Support

**As a** developer working across repos  
**I want** to track cross-repo dependencies  
**So that** I can manage monorepo-like workflows

| ID | Acceptance Criterion |
|----|---------------------|
| AC-018.1 | THE SYSTEM SHALL store `source_repo` on each issue (default: current repo) |
| AC-018.2 | WHEN displaying dependencies THE SYSTEM SHALL show repo prefix for external issues (e.g., `other-repo:bd-xyz`) |
| AC-018.3 | THE SYSTEM SHALL support `hbd dep add <local-id> blocks <repo>:<remote-id>` for cross-repo deps |
| AC-018.4 | THE SYSTEM SHALL NOT validate existence of external issue IDs (trust user input) |

---

## 5. AI Agent Features

### US-019: Agent Tracking

**As an** AI coding assistant  
**I want** my issue activity tracked  
**So that** humans can audit agent work

| ID | Acceptance Criterion |
|----|---------------------|
| AC-019.1 | THE SYSTEM SHALL support `--agent <agent-id>` flag on all mutating commands |
| AC-019.2 | WHEN `--agent` is provided THE SYSTEM SHALL set `created_by_type="agent"` and `agent_id` fields |
| AC-019.3 | THE SYSTEM SHALL support `--session <session-id>` to group related agent actions |
| AC-019.4 | WHEN a user runs `hbd list --created-by-agent` THE SYSTEM SHALL filter to agent-created issues |

---

### US-020: Ephemeral Issues

**As an** AI agent  
**I want** to create temporary issues for my workspace  
**So that** I can track intermediate work without polluting the project

| ID | Acceptance Criterion |
|----|---------------------|
| AC-020.1 | WHEN a user runs `hbd create --ephemeral "Temp task"` THE SYSTEM SHALL set `ephemeral=true` |
| AC-020.2 | THE SYSTEM SHALL store ephemeral issues in HelixDB but NOT export them to `.tickets/` |
| AC-020.3 | THE SYSTEM SHALL exclude ephemeral issues from `hbd list` unless `--include-ephemeral` is provided |
| AC-020.4 | WHEN `hbd sync` runs THE SYSTEM SHALL delete closed ephemeral issues older than 24 hours |

---

### US-021: Gate Coordination

**As an** AI agent  
**I want** to create gates that wait for external conditions  
**So that** I can coordinate multi-step workflows

| ID | Acceptance Criterion |
|----|---------------------|
| AC-021.1 | WHEN a user runs `hbd create --type gate --await "gh:pr:123"` THE SYSTEM SHALL create a gate issue with `await_condition` field |
| AC-021.2 | THE SYSTEM SHALL support await types: `gh:pr:<number>`, `gh:run:<id>`, `timer:<duration>`, `human`, `issue:<id>:closed` |
| AC-021.3 | WHEN a user runs `hbd wait <gate-id>` THE SYSTEM SHALL poll the await condition and block until satisfied or timeout |
| AC-021.4 | WHEN the condition is satisfied THE SYSTEM SHALL set `await_status="satisfied"` and close the gate |
| AC-021.5 | WHEN `--timeout <duration>` is exceeded THE SYSTEM SHALL set `await_status="timeout"` and exit with code 1 |

---

### US-022: Context Compaction

**As an** AI agent  
**I want** old closed issues summarized to reduce context  
**So that** I can maintain long-term project memory efficiently

| ID | Acceptance Criterion |
|----|---------------------|
| AC-022.1 | WHEN an issue has been closed for >30 days THE SYSTEM SHALL mark it as eligible for Tier 1 compaction |
| AC-022.2 | WHEN a user runs `hbd compact` THE SYSTEM SHALL summarize eligible issues to <25% original size using an LLM |
| AC-022.3 | THE SYSTEM SHALL preserve the original embedding vector (do not re-embed the summary) |
| AC-022.4 | THE SYSTEM SHALL set `compaction_tier=1` and store `original_size` for metrics |
| AC-022.5 | WHEN summary would be larger than original THE SYSTEM SHALL skip compaction for that issue |
| AC-022.6 | WHEN `--dry-run` is provided THE SYSTEM SHALL report what would be compacted without making changes |

---

### US-023: Context Retrieval

**As an** AI agent  
**I want** to get relevant project context quickly  
**So that** I can understand the codebase state

| ID | Acceptance Criterion |
|----|---------------------|
| AC-023.1 | WHEN a user runs `hbd context` THE SYSTEM SHALL return a curated summary of: open issues by priority, recent activity, blocked items |
| AC-023.2 | WHEN `--query "topic"` is provided THE SYSTEM SHALL use hybrid search to find relevant issues |
| AC-023.3 | WHEN `--limit <tokens>` is provided THE SYSTEM SHALL truncate output to fit within token budget |
| AC-023.4 | THE SYSTEM SHALL format output as Markdown suitable for LLM context windows |

---

## 6. Analytics & Health

### US-024: Project Health

**As a** project lead  
**I want** to see project health metrics  
**So that** I can identify issues early

| ID | Acceptance Criterion |
|----|---------------------|
| AC-024.1 | WHEN a user runs `hbd health` THE SYSTEM SHALL display: total open, blocked count, stale count (>14 days no update), velocity (closed/week) |
| AC-024.2 | THE SYSTEM SHALL assign a health grade: A (healthy), B (minor issues), C (needs attention), D (critical) |
| AC-024.3 | THE SYSTEM SHALL provide specific recommendations for improving health |

---

### US-025: Label Health

**As a** project lead  
**I want** to see health metrics per label  
**So that** I can identify problem areas

| ID | Acceptance Criterion |
|----|---------------------|
| AC-025.1 | WHEN a user runs `hbd health --label <name>` THE SYSTEM SHALL display metrics scoped to that label |
| AC-025.2 | THE SYSTEM SHALL show: total active, blocked, avg age, 30-day velocity |
| AC-025.3 | THE SYSTEM SHALL compare to project-wide averages |

---

### US-026: Statistics

**As a** developer  
**I want** to see issue statistics  
**So that** I can understand project state

| ID | Acceptance Criterion |
|----|---------------------|
| AC-026.1 | WHEN a user runs `hbd stats` THE SYSTEM SHALL display counts by status, type, priority, and assignee |
| AC-026.2 | THE SYSTEM SHALL show trends: created this week, closed this week, net change |
| AC-026.3 | WHEN `--json` is provided THE SYSTEM SHALL output statistics as JSON |

---

### US-026B: Issue Count

**As a** developer  
**I want to** quickly see the total number of issues  
**So that** I can understand project size at a glance

| ID | Acceptance Criterion |
|----|---------------------|
| AC-026B.1 | WHEN a user runs `hbd count` THE SYSTEM SHALL display the total number of issues |
| AC-026B.2 | WHEN `--status <status>` is provided THE SYSTEM SHALL count only issues with that status |
| AC-026B.3 | WHEN `--type <type>` is provided THE SYSTEM SHALL count only issues of that type |
| AC-026B.4 | THE SYSTEM SHALL support combining filters |
| AC-026B.5 | WHEN `--json` flag is provided THE SYSTEM SHALL output count as JSON |

---

## 6B. Maintenance & Cleanup

### US-026C: Merge Duplicates

**As a** developer  
**I want to** merge duplicate issues into one  
**So that** I can consolidate discussions and reduce clutter

| ID | Acceptance Criterion |
|----|---------------------|
| AC-026C.1 | WHEN a user runs `hbd merge <source-ids...> --into <target-id>` THE SYSTEM SHALL consolidate all sources into the target |
| AC-026C.2 | THE SYSTEM SHALL move all comments from source issues to the target |
| AC-026C.3 | THE SYSTEM SHALL transfer all dependencies (incoming and outgoing) to the target |
| AC-026C.4 | THE SYSTEM SHALL set source issues to status "tombstone" with a note referencing the target |
| AC-026C.5 | WHEN `--dry-run` is provided THE SYSTEM SHALL show what would be merged without making changes |
| AC-026C.6 | THE SYSTEM SHALL NOT allow merging closed issues into open issues |

---

### US-026D: Restore Compacted Issue

**As a** developer  
**I want to** view the full content of a compacted issue  
**So that** I can see original details when needed

| ID | Acceptance Criterion |
|----|---------------------|
| AC-026D.1 | WHEN a user runs `hbd restore <id>` THE SYSTEM SHALL display the pre-compaction content from git history |
| AC-026D.2 | WHEN the issue has not been compacted THE SYSTEM SHALL display a message indicating no compaction history |
| AC-026D.3 | THE SYSTEM SHALL show: original title, description, and compaction date |
| AC-026D.4 | WHEN `--json` flag is provided THE SYSTEM SHALL output the restored content as JSON |

---

### US-026E: Admin Cleanup

**As a** project maintainer  
**I want to** bulk delete old closed issues  
**So that** I can reduce database size and clutter

| ID | Acceptance Criterion |
|----|---------------------|
| AC-026E.1 | WHEN a user runs `hbd admin cleanup` THE SYSTEM SHALL identify closed issues eligible for deletion |
| AC-026E.2 | WHEN `--older-than N` is provided THE SYSTEM SHALL only include issues closed more than N days ago (default: 90) |
| AC-026E.3 | WHEN `--dry-run` is provided THE SYSTEM SHALL show what would be deleted without making changes |
| AC-026E.4 | WHEN `--force` is provided THE SYSTEM SHALL proceed with deletion |
| AC-026E.5 | THE SYSTEM SHALL require `--force` flag for actual deletion (safety) |
| AC-026E.6 | WHEN `--cascade` is provided THE SYSTEM SHALL also delete orphaned dependencies and comments |
| AC-026E.7 | THE SYSTEM SHALL NOT delete issues with open children or dependents unless `--cascade` |

---

## 7. Configuration

### US-026F: System Info

**As a** developer  
**I want to** see system status and configuration  
**So that** I can troubleshoot issues and verify setup

| ID | Acceptance Criterion |
|----|---------------------|
| AC-026F.1 | WHEN a user runs `hbd info` THE SYSTEM SHALL display: database path, issue prefix, helixd status |
| AC-026F.2 | THE SYSTEM SHALL show: total issue count, embedding model status, last sync time |
| AC-026F.3 | THE SYSTEM SHALL indicate if the project is initialized |
| AC-026F.4 | WHEN `--json` flag is provided THE SYSTEM SHALL output all info as JSON |

---

### US-027: Project Initialization

**As a** developer  
**I want** to initialize hbd in my project  
**So that** I can start tracking issues

| ID | Acceptance Criterion |
|----|---------------------|
| AC-027.1 | WHEN a user runs `hbd init` THE SYSTEM SHALL create `.tickets/` directory |
| AC-027.2 | THE SYSTEM SHALL create `.helix/config.toml` with default settings |
| AC-027.3 | THE SYSTEM SHALL create `helix.toml` with the issue tracker schema |
| AC-027.4 | THE SYSTEM SHALL add `.helix/helix.db/` to `.gitignore` |
| AC-027.5 | WHEN `hbd init` is run in an existing hbd project THE SYSTEM SHALL abort with an error |

---

### US-028: Configuration

**As a** developer  
**I want** to configure hbd behavior  
**So that** I can customize it for my workflow

| ID | Acceptance Criterion |
|----|---------------------|
| AC-028.1 | THE SYSTEM SHALL read configuration from `.helix/config.toml` |
| AC-028.2 | THE SYSTEM SHALL support configuring: embedding backend, cloud API keys, sync interval, default project |
| AC-028.3 | THE SYSTEM SHALL support environment variable overrides with `HBD_` prefix |
| AC-028.4 | WHEN a user runs `hbd config show` THE SYSTEM SHALL display current configuration |
| AC-028.5 | WHEN a user runs `hbd config set <key> <value>` THE SYSTEM SHALL update the config file |

---

## Non-Functional Requirements

### NFR-001: Performance

| ID | Requirement |
|----|-------------|
| NFR-001.1 | THE SYSTEM SHALL complete `hbd list` in <100ms for projects with <1000 issues |
| NFR-001.2 | THE SYSTEM SHALL complete `hbd search` in <500ms including embedding generation |
| NFR-001.3 | THE SYSTEM SHALL complete `hbd create` in <200ms (excluding embedding, which is async) |
| NFR-001.4 | THE SYSTEM SHALL use <500MB RAM for projects with <10000 issues |

---

### NFR-002: Reliability

| ID | Requirement |
|----|-------------|
| NFR-002.1 | THE SYSTEM SHALL NOT corrupt data on unexpected termination (use atomic writes) |
| NFR-002.2 | THE SYSTEM SHALL maintain consistency between `.tickets/` and HelixDB after `hbd sync` |
| NFR-002.3 | THE SYSTEM SHALL log all errors with sufficient context for debugging |

---

### NFR-003: Usability

| ID | Requirement |
|----|-------------|
| NFR-003.1 | THE SYSTEM SHALL provide helpful error messages with suggested fixes |
| NFR-003.2 | THE SYSTEM SHALL support `--help` on all commands with examples |
| NFR-003.3 | THE SYSTEM SHALL support shell completion for bash, zsh, and fish |
| NFR-003.4 | THE SYSTEM SHALL use consistent output formatting across commands |
