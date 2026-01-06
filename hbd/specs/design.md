# Design

This document defines the technical architecture, data model, and query implementations for `hbd`.

> **Implementation Status (2026-01-06)**
>
> The current implementation uses **file-based storage only** (Markdown files in `.tickets/`).
> HelixDB integration, vector embeddings, and graph queries are **planned but not yet implemented**.
>
> What's working today:
> - File-based CRUD via `.tickets/*.md`
> - In-memory dependency traversal and cycle detection
> - YAML frontmatter parsing and serialization
>
> What's planned:
> - HelixDB embedded in the binary (like SQLite—no server to run) as query cache
> - fastembed for local semantic search (no API calls needed)
> - BM25 + vector hybrid search
> - helixd daemon for file watching and background sync
>
> **Note:** Markdown files remain the source of truth. HelixDB acts as a fast query cache
> that can be rebuilt from `.tickets/` at any time.
>
> **HelixDB API Patterns:** When implementing HelixDB integration, follow the corrected patterns
> documented in `helix-decisions/docs/phase3/PHASE_3_CORRECTIONS.md`. Key requirements:
> - Edges must write to 3 databases (edges_db, out_edges_db, in_edges_db)
> - Nodes must use arena allocation + ImmutablePropertiesMap
> - Vectors are stored separately, linked via vector_id property
> - Use `hash_label()` for adjacency DB keys

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Data Model (HelixQL Schema)](#data-model-helixql-schema)
3. [Core Queries](#core-queries)
4. [Embedding Strategy](#embedding-strategy)
5. [Git Sync Protocol](#git-sync-protocol)
6. [File Formats](#file-formats)
7. [Error Handling](#error-handling)

---

## Architecture Overview

### Target Architecture (Planned)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              CLI Layer (hbd)                             │
│   hbd create, list, search, similar, ready, blocked, dep, graph, ...    │
│   - Rust CLI with clap                                                   │
│   - All commands support --json for AI agents                            │
│   - Tries daemon RPC first, falls back to direct DB access              │
└──────────────────────────────────────┬──────────────────────────────────┘
                                       │
               ┌───────────────────────┼───────────────────┐
               │                       │                   │
               v                       v                   v
┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐
│    Git Layer        │ │      HelixDB        │ │   Embedding Layer   │
│  .tickets/*.md      │ │      (LMDB)         │ │                     │
│                     │ │                     │ │  fastembed (local)  │
│  - Source of truth  │ │  - Fast queries     │ │  BGE-small-en-v1.5  │
│  - YAML frontmatter │ │  - Graph traversal  │ │                     │
│  - Merge-friendly   │ │  - Vector search    │ │  Cloud fallback:    │
│  - Human-readable   │ │  - BM25 index       │ │  OpenAI / Gemini    │
└─────────────────────┘ └─────────────────────┘ └─────────────────────┘
               │                   │                   │
               └───────────────────┼───────────────────┘
                                   │
                             ┌─────v─────┐
                             │  Daemon   │
                             │ (helixd)  │
                             │           │
                             │  - Sync   │
                             │  - Watch  │
                             │  - Embed  │
                             └───────────┘
```

### Daemon Integration (Planned)

hbd uses the global helixd daemon for background sync and embedding. The CLI
enqueues work via IPC and optionally waits with `--sync`. Protocol details live
in `shared/helix-daemon/specs/design.md`.

### Current Architecture (Implemented)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              CLI Layer (hbd)                             │
│   hbd create, list, ready, blocked, dep, explain, stale, stats, ...     │
│   - Rust CLI with clap                                                   │
│   - All commands support --json for AI agents                            │
│   - Direct file access (no daemon)                                       │
└──────────────────────────────────────┬──────────────────────────────────┘
                                       │
                                       v
                         ┌─────────────────────┐
                         │    Git Layer        │
                         │  .tickets/*.md      │
                         │                     │
                         │  - Source of truth  │
                         │  - YAML frontmatter │
                         │  - In-memory graphs │
                         │  - Human-readable   │
                         └─────────────────────┘
```
┌─────────────────────────────────────────────────────────────────────────┐
│                              CLI Layer (hbd)                             │
│   hbd create, list, search, similar, ready, blocked, dep, graph, ...    │
│   - Rust CLI with clap                                                   │
│   - All commands support --json for AI agents                            │
│   - Tries daemon RPC first, falls back to direct DB access              │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   │
               ┌───────────────────┼───────────────────┐
               │                   │                   │
               v                   v                   v
┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐
│    Git Layer        │ │      HelixDB        │ │   Embedding Layer   │
│  .tickets/*.md      │ │      (LMDB)         │ │                     │
│                     │ │                     │ │  fastembed (local)  │
│  - Source of truth  │ │  - Fast queries     │ │  BGE-small-en-v1.5  │
│  - YAML frontmatter │ │  - Graph traversal  │ │                     │
│  - Merge-friendly   │ │  - Vector search    │ │  Cloud fallback:    │
│  - Human-readable   │ │  - BM25 index       │ │  OpenAI / Gemini    │
└─────────────────────┘ └─────────────────────┘ └─────────────────────┘
               │                   │                   │
               └───────────────────┼───────────────────┘
                                   │
                             ┌─────v─────┐
                             │  Daemon   │
                             │ (helixd)  │
                             │           │
                             │  - Sync   │
                             │  - Watch  │
                             │  - Embed  │
                             └───────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| **CLI (hbd)** | User interaction, command parsing, output formatting |
| **Git Layer** | Source of truth, merge-friendly storage, version history |
| **HelixDB** | Fast queries, graph traversal, vector/BM25 search |
| **Embedding Layer** | Text vectorization for semantic search |
| **Daemon (helixd)** | Background sync, file watching, async embedding |

### Data Flow

**Write Path:**
```
hbd create "Title"
    │
    ├──▶ Generate hash-based ID (bd-xxxxxx)
    │
    ├──▶ Write .tickets/bd-xxxxxx.md
    │
    ├──▶ Insert Issue node in HelixDB
    │
    └──▶ Queue embedding generation (async)
         └──▶ Insert IssueEmbedding + HAS_EMBEDDING edge
```

**Read Path (after git pull):**
```
git pull (new .tickets/ files)
    │
    ├──▶ Daemon detects file changes
    │
    ├──▶ For each changed file:
    │    ├── Parse YAML frontmatter
    │    ├── Compute content hash
    │    ├── Compare with HelixDB
    │    └── Upsert if content differs
    │
    └──▶ Re-embed if text changed
```

---

## Data Model (HelixQL Schema)

### Nodes

```hql
// ═══════════════════════════════════════════════════════════════════════
//                              ISSUE
// ═══════════════════════════════════════════════════════════════════════

N::Issue {
    // Identity
    INDEX id: String,                     // Hash-based ID: bd-a1b2c3
    
    // Core fields
    title: String,
    body: String,
    status: String,                       // open | in_progress | blocked | deferred | closed | tombstone
    priority: I32,                        // 0=critical, 1=high, 2=medium, 3=low, 4=backlog
    issue_type: String,                   // bug | feature | task | epic | chore | gate
    
    // Timestamps
    created_at: Date DEFAULT NOW,
    updated_at: Date DEFAULT NOW,
    closed_at: Date,
    
    // Attribution
    created_by: String,                   // Username or agent ID
    created_by_type: String,              // human | agent
    assignee: String,
    
    // Agent tracking
    agent_id: String,                     // e.g., "claude-code", "cursor"
    session_id: String,                   // Groups related agent actions
    
    // Extended fields
    external_ref: String,                 // GitHub/Jira/Linear link
    source_repo: String,                  // For multi-repo support
    estimated_minutes: I32 DEFAULT 60,
    actual_minutes: I32,
    
    // Ephemeral issues (not exported to git)
    ephemeral: Boolean DEFAULT false,
    
    // Gate-specific (type=gate)
    await_condition: String,              // e.g., "gh:pr:123", "timer:1h", "human"
    await_status: String,                 // pending | satisfied | timeout
    
    // Compaction
    compaction_tier: I32 DEFAULT 0,       // 0=full, 1=summarized, 2=archived
    original_size: I32,                   // Pre-compaction size in bytes
    
    // Content hash for sync
    content_hash: String,
}

// ═══════════════════════════════════════════════════════════════════════
//                              COMMENT
// ═══════════════════════════════════════════════════════════════════════

N::Comment {
    INDEX id: String,
    body: String,
    created_at: Date DEFAULT NOW,
    created_by: String,
    created_by_type: String,              // human | agent
}

// ═══════════════════════════════════════════════════════════════════════
//                              USER
// ═══════════════════════════════════════════════════════════════════════

N::User {
    INDEX handle: String,
    name: String,
    email: String,
    kind: String,                         // human | agent
    agent_id: String,                     // If kind=agent
}

// ═══════════════════════════════════════════════════════════════════════
//                              LABEL
// ═══════════════════════════════════════════════════════════════════════

N::Label {
    INDEX name: String,
    color: String,                        // Hex color: #ff0000
    description: String,
}

// ═══════════════════════════════════════════════════════════════════════
//                              PROJECT
// ═══════════════════════════════════════════════════════════════════════

N::Project {
    INDEX name: String,
    description: String,
    prefix: String,                       // Optional issue ID prefix
}

// ═══════════════════════════════════════════════════════════════════════
//                              TEAM
// ═══════════════════════════════════════════════════════════════════════

N::Team {
    INDEX name: String,
    description: String,
}

// ═══════════════════════════════════════════════════════════════════════
//                         VECTOR EMBEDDINGS
// ═══════════════════════════════════════════════════════════════════════

V::IssueEmbedding {
    issue_id: String,
    text_hash: String,                    // Hash of embedded text for change detection
    model: String,                        // e.g., "fastembed:bge-small-en-v1.5"
    // Implicit: 384-dimensional vector for BGE-small
}

V::CommentEmbedding {
    comment_id: String,
    text_hash: String,
    model: String,
}
```

### Edges

```hql
// ═══════════════════════════════════════════════════════════════════════
//                              EDGES
// ═══════════════════════════════════════════════════════════════════════

// User relationships
E::AUTHORED {
    From: User,
    To: Issue,
}

E::ASSIGNED_TO {
    From: Issue,
    To: User,
}

E::COMMENTED {
    From: User,
    To: Comment,
}

// Comment relationships
E::COMMENT_ON {
    From: Comment,
    To: Issue,
}

E::MENTIONS {
    From: Comment,
    To: Issue,
    Properties: {
        mention_text: String,             // The @bd-xxxx text
    }
}

// Issue organization
E::TAGGED {
    From: Issue,
    To: Label,
}

E::IN_PROJECT {
    From: Issue,
    To: Project,
}

// Dependencies (critical for graph features)
E::DEPENDS_ON {
    From: Issue,
    To: Issue,
    Properties: {
        dep_type: String,                 // blocks | related | waits_for | duplicate_of
        weight: F64 DEFAULT 1.0,          // For weighted path algorithms
        created_at: Date DEFAULT NOW,
        created_by: String,
    }
}

E::PARENT_OF {
    From: Issue,
    To: Issue,
    Properties: {
        created_at: Date DEFAULT NOW,
    }
}

// Embedding relationships
E::HAS_EMBEDDING {
    From: Issue,
    To: IssueEmbedding,
}

E::HAS_COMMENT_EMBEDDING {
    From: Comment,
    To: CommentEmbedding,
}

// Team relationships
E::MEMBER_OF {
    From: User,
    To: Team,
}

E::OWNS {
    From: Team,
    To: Project,
}

// Subscriptions
E::SUBSCRIBED_TO {
    From: User,
    To: Issue,
}
```

---

## Core Queries

### Issue CRUD

```hql
// Get issue by ID with all relationships
QUERY getIssue(issue_id: String) =>
    issue <- N<Issue>({id: issue_id})
    
    labels <- issue::Out<TAGGED>::{name, color}
    project <- issue::Out<IN_PROJECT>::{name}
    assignee <- issue::Out<ASSIGNED_TO>::{handle, name}
    author <- issue::In<AUTHORED>::{handle, name}
    
    blockers <- issue::Out<DEPENDS_ON>
        ::WHERE(_::{dep_type}::EQ("blocks"))
        ::{id, title, status}
    
    blocked_by <- issue::In<DEPENDS_ON>
        ::WHERE(_::{dep_type}::EQ("blocks"))
        ::{id, title, status}
    
    comments <- issue::In<COMMENT_ON>
        ::ORDER<Asc>(_::{created_at})
        ::{id, body, created_at, created_by}
    
    RETURN {
        issue: issue,
        labels: labels,
        project: project,
        assignee: assignee,
        author: author,
        blockers: blockers,
        blocked_by: blocked_by,
        comments: comments
    }
```

```hql
// List issues with filters
QUERY listIssues(
    status: String,
    issue_type: String,
    priority: I32,
    project: String,
    label: String,
    assignee: String,
    include_ephemeral: Boolean
) =>
    issues <- N<Issue>
        ::WHERE(OR(status::EQ(""), _::{status}::EQ(status)))
        ::WHERE(OR(issue_type::EQ(""), _::{issue_type}::EQ(issue_type)))
        ::WHERE(OR(priority::EQ(-1), _::{priority}::EQ(priority)))
        ::WHERE(OR(include_ephemeral, NOT(_::{ephemeral})))
    
    // Filter by project if specified
    filtered <- issues::WHERE(
        OR(
            project::EQ(""),
            _::Out<IN_PROJECT>::{name}::EQ(project)
        )
    )
    
    // Filter by label if specified
    filtered2 <- filtered::WHERE(
        OR(
            label::EQ(""),
            _::Out<TAGGED>::{name}::EQ(label)
        )
    )
    
    // Filter by assignee if specified
    filtered3 <- filtered2::WHERE(
        OR(
            assignee::EQ(""),
            _::Out<ASSIGNED_TO>::{handle}::EQ(assignee)
        )
    )
    
    // Sort by priority (asc) then created_at (desc)
    sorted <- filtered3
        ::ORDER<Asc>(_::{priority})
        ::ORDER<Desc>(_::{created_at})
    
    RETURN sorted::{id, title, status, priority, issue_type, created_at}
```

### Search

```hql
// BM25 text search
QUERY searchBM25(query: String, limit: I32) =>
    results <- SearchBM25<Issue>(query, limit)
    RETURN results::{id, title, status, priority, score: _::score}
```

```hql
// Semantic similarity search
#[model("local:bge-small-en-v1.5")]
QUERY searchSemantic(query: String, limit: I32) =>
    candidates <- SearchV<IssueEmbedding>(Embed(query), MUL(limit, 2))
    
    issues <- candidates::In<HAS_EMBEDDING>
        ::WHERE(_::{status}::NEQ("tombstone"))
        ::RerankMMR(lambda: 0.7)
        ::RANGE(0, limit)
    
    RETURN issues::{id, title, status, priority, similarity: _::score}
```

```hql
// Hybrid search with RRF fusion
#[model("local:bge-small-en-v1.5")]
QUERY searchHybrid(query: String, status: String, project: String, limit: I32) =>
    // BM25 results
    bm25 <- SearchBM25<Issue>(query, MUL(limit, 2))
    
    // Vector results
    vector <- SearchV<IssueEmbedding>(Embed(query), MUL(limit, 2))
        ::In<HAS_EMBEDDING>
    
    // Fuse with RRF
    fused <- bm25::RerankRRF(k: 60)
    
    // Apply filters
    filtered <- fused
        ::WHERE(OR(status::EQ(""), _::{status}::EQ(status)))
        ::WHERE(OR(
            project::EQ(""),
            _::Out<IN_PROJECT>::{name}::EQ(project)
        ))
    
    // Diversity reranking
    diverse <- filtered::RerankMMR(lambda: 0.7)::RANGE(0, limit)
    
    RETURN diverse::{id, title, status, priority, score: _::score}
```

```hql
// Find similar issues (for duplicate detection)
#[model("local:bge-small-en-v1.5")]
QUERY findSimilar(issue_id: String, limit: I32) =>
    // Get the issue's embedding
    source <- N<Issue>({id: issue_id})::Out<HAS_EMBEDDING>
    
    // Find similar embeddings
    candidates <- SearchV<IssueEmbedding>(source, MUL(limit, 2))
    
    // Traverse to issues, exclude self
    similar <- candidates::In<HAS_EMBEDDING>
        ::WHERE(_::{id}::NEQ(issue_id))
        ::WHERE(_::{status}::NEQ("tombstone"))
        ::RerankMMR(lambda: 0.7)
        ::RANGE(0, limit)
    
    RETURN similar::{id, title, status, similarity: _::score}
```

### Labels

```hql
// Add label to issue
QUERY addLabel(issue_id: String, label_name: String) =>
    issue <- N<Issue>({id: issue_id})
    
    // Create label if it doesn't exist
    label <- UPSERT N<Label>({name: label_name})
    
    // Create edge if not exists
    edge <- UPSERT E<TAGGED>(issue, label)
    
    RETURN {
        issue_id: issue_id,
        label: label_name,
        created: edge::is_new
    }
```

```hql
// Remove label from issue
QUERY removeLabel(issue_id: String, label_name: String) =>
    issue <- N<Issue>({id: issue_id})
    label <- N<Label>({name: label_name})
    
    // Delete edge
    deleted <- DELETE E<TAGGED>(issue, label)
    
    RETURN {
        issue_id: issue_id,
        label: label_name,
        removed: EXISTS(deleted)
    }
```

```hql
// List all labels on an issue
QUERY listLabels(issue_id: String) =>
    issue <- N<Issue>({id: issue_id})
    labels <- issue::Out<TAGGED>::{name, color, description}
    
    RETURN labels
```

```hql
// List all labels in project with usage counts
QUERY listAllLabels() =>
    labels <- N<Label>
    
    FOR label IN labels {
        count <- label::In<TAGGED>::COUNT
    }
    
    RETURN labels::{name, color, description, usage_count: count}
        ::ORDER<Desc>(_::{usage_count})
```

### Comments

```hql
// Add comment to issue
QUERY addComment(issue_id: String, body: String, created_by: String, created_by_type: String) =>
    issue <- N<Issue>({id: issue_id})
    
    comment <- INSERT N<Comment>({
        id: GENERATE_ID(),
        body: body,
        created_by: created_by,
        created_by_type: created_by_type
    })
    
    edge <- INSERT E<COMMENT_ON>(comment, issue)
    
    // Update issue timestamp
    UPDATE issue SET updated_at = NOW()
    
    RETURN comment::{id, body, created_at, created_by}
```

```hql
// List comments for an issue
QUERY listComments(issue_id: String) =>
    issue <- N<Issue>({id: issue_id})
    
    comments <- issue::In<COMMENT_ON>
        ::ORDER<Asc>(_::{created_at})
        ::{id, body, created_at, created_by, created_by_type}
    
    RETURN comments
```

### Stale Issues

```hql
// Find stale issues (not updated recently)
QUERY staleIssues(days: I32, status: String, limit: I32) =>
    threshold <- SUB(NOW(), MUL(days, 86400))  // days to seconds
    
    issues <- N<Issue>
        ::WHERE(_::{updated_at}::LT(threshold))
        ::WHERE(
            OR(
                status::EQ(""),
                _::{status}::EQ(status)
            )
        )
        ::WHERE(
            OR(
                _::{status}::EQ("open"),
                _::{status}::EQ("in_progress")
            )
        )
        ::WHERE(NOT(_::{ephemeral}))
        ::ORDER<Asc>(_::{updated_at})
        ::RANGE(0, limit)
    
    RETURN issues::{id, title, status, priority, updated_at, assignee}
```

### Dependencies & Graph

```hql
// Find all cycles in the dependency graph
QUERY findAllCycles() =>
    // Get all issues with blocking dependencies
    issues <- N<Issue>
        ::WHERE(EXISTS(_::Out<DEPENDS_ON>::WHERE(_::{dep_type}::EQ("blocks"))))
    
    // For each issue, check if it's part of a cycle
    FOR issue IN issues {
        cycle <- issue
            ::Out<DEPENDS_ON>*
            ::WHERE(_::{dep_type}::EQ("blocks"))
            ::WHERE(_::{id}::EQ(issue::{id}))
    }
    
    // Collect unique cycles
    cycles <- cycle::DISTINCT
    
    RETURN cycles::{path: _::{id}::COLLECT}
```

```hql
// Detect if adding a dependency would create a cycle
QUERY detectCycle(from_id: String, to_id: String) =>
    // Check if 'to' can already reach 'from' via DEPENDS_ON
    path <- N<Issue>({id: to_id})
        ::Out<DEPENDS_ON>*                // Transitive closure
        ::WHERE(_::{id}::EQ(from_id))
    
    cycle_exists <- EXISTS(path)
    
    // If cycle exists, get the path for error message
    cycle_path <- N<Issue>({id: to_id})
        ::ShortestPathBFS<DEPENDS_ON>
        ::To(N<Issue>({id: from_id}))
    
    RETURN {
        would_cycle: cycle_exists,
        cycle_path: cycle_path::{id}
    }
```

```hql
// Get ready (unblocked) issues
QUERY readyIssues(project: String) =>
    issues <- N<Issue>
        ::WHERE(_::{status}::EQ("open"))
        ::WHERE(NOT(_::{ephemeral}))
        // No open blockers
        ::WHERE(
            NOT(EXISTS(
                _::Out<DEPENDS_ON>
                    ::WHERE(_::{dep_type}::EQ("blocks"))
                    ::WHERE(_::{status}::NEQ("closed"))
            ))
        )
    
    // Filter by project if specified
    filtered <- issues::WHERE(
        OR(project::EQ(""), _::Out<IN_PROJECT>::{name}::EQ(project))
    )
    
    // Sort by priority then age
    sorted <- filtered
        ::ORDER<Asc>(_::{priority})
        ::ORDER<Desc>(_::{created_at})
    
    RETURN sorted::{id, title, priority, created_at, assignee}
```

```hql
// Get blocked issues with blocker details
QUERY blockedIssues(project: String) =>
    issues <- N<Issue>
        ::WHERE(
            OR(
                _::{status}::EQ("blocked"),
                EXISTS(
                    _::Out<DEPENDS_ON>
                        ::WHERE(_::{dep_type}::EQ("blocks"))
                        ::WHERE(_::{status}::NEQ("closed"))
                )
            )
        )
        ::WHERE(NOT(_::{ephemeral}))
    
    // Filter by project
    filtered <- issues::WHERE(
        OR(project::EQ(""), _::Out<IN_PROJECT>::{name}::EQ(project))
    )
    
    FOR issue IN filtered {
        blockers <- issue::Out<DEPENDS_ON>
            ::WHERE(_::{dep_type}::EQ("blocks"))
            ::WHERE(_::{status}::NEQ("closed"))
            ::{id, title, status, assignee}
    }
    
    RETURN {
        issue: filtered::{id, title, priority},
        blockers: blockers
    }
```

```hql
// Critical path analysis - find longest blocking chain to epic
QUERY criticalPath(epic_id: String) =>
    epic <- N<Issue>({id: epic_id})
    
    // Get all child issues
    children <- epic::Out<PARENT_OF>
        ::WHERE(_::{status}::NEQ("closed"))
    
    // For each child, find its dependency chain weighted by priority * estimate
    FOR child IN children {
        path <- child
            ::ShortestPathDijkstras<DEPENDS_ON>(
                MUL(
                    SUB(5, _::{priority}),          // Higher priority = higher weight
                    DIV(_::{estimated_minutes}, 60) // Hours
                )
            )
            ::WHERE(_::{dep_type}::EQ("blocks"))
    }
    
    // Find the maximum weighted path
    critical <- path::ORDER<Desc>(_::total_weight)::FIRST
    
    RETURN {
        epic: epic::{id, title},
        critical_path: critical::{id, title, priority, estimated_minutes},
        total_weight: critical::total_weight
    }
```

```hql
// Generate dependency graph for visualization (DOT format data)
QUERY dependencyGraph(issue_id: String, max_depth: I32) =>
    root <- N<Issue>({id: issue_id})
    
    // Get all connected issues within depth
    connected <- root
        ::Out<DEPENDS_ON>{0, max_depth}
        ::{id, title, status, priority}
    
    // Get all edges between connected issues
    edges <- connected::Out<DEPENDS_ON>
        ::WHERE(_::IN(connected))
        ::{from: _::source::{id}, to: _::target::{id}, dep_type: _::{dep_type}}
    
    RETURN {
        nodes: connected,
        edges: edges,
        root_id: issue_id
    }
```

```hql
// Explain blocker chain for an issue
QUERY explainBlockers(issue_id: String) =>
    issue <- N<Issue>({id: issue_id})
    
    // Get all transitive blockers
    blockers <- issue
        ::Out<DEPENDS_ON>*
        ::WHERE(_::{dep_type}::EQ("blocks"))
        ::{id, title, status, priority, assignee}
    
    // Build tree structure
    tree <- issue::Out<DEPENDS_ON>
        ::WHERE(_::{dep_type}::EQ("blocks"))
    
    FOR blocker IN tree {
        children <- blocker::Out<DEPENDS_ON>
            ::WHERE(_::{dep_type}::EQ("blocks"))
    }
    
    RETURN {
        issue: issue::{id, title},
        blocker_tree: tree,
        total_blockers: blockers::COUNT
    }
```

### Health & Analytics

```hql
// Project health metrics
QUERY projectHealth(project: String) =>
    all_issues <- N<Issue>
        ::WHERE(NOT(_::{ephemeral}))
        ::WHERE(OR(project::EQ(""), _::Out<IN_PROJECT>::{name}::EQ(project)))
    
    open <- all_issues::WHERE(_::{status}::EQ("open"))
    in_progress <- all_issues::WHERE(_::{status}::EQ("in_progress"))
    blocked <- all_issues::WHERE(_::{status}::EQ("blocked"))
    
    // Stale = open/in_progress with no update in 14 days
    stale <- all_issues
        ::WHERE(
            OR(_::{status}::EQ("open"), _::{status}::EQ("in_progress"))
        )
        ::WHERE(_::{updated_at}::LT(SUB(NOW(), 1209600)))  // 14 days in seconds
    
    // Velocity = closed in last 7 days
    velocity <- all_issues
        ::WHERE(_::{status}::EQ("closed"))
        ::WHERE(_::{closed_at}::GTE(SUB(NOW(), 604800)))   // 7 days
        ::COUNT
    
    RETURN {
        total_open: open::COUNT,
        total_in_progress: in_progress::COUNT,
        total_blocked: blocked::COUNT,
        total_stale: stale::COUNT,
        weekly_velocity: velocity
    }
```

```hql
// Label health metrics
QUERY labelHealth(label_name: String) =>
    label <- N<Label>({name: label_name})
    issues <- label::In<TAGGED>
    
    active <- issues::WHERE(
        AND(
            _::{status}::NEQ("closed"),
            _::{status}::NEQ("tombstone")
        )
    )
    
    blocked <- active::WHERE(_::{status}::EQ("blocked"))
    
    // Average age in days
    avg_age <- AVG(DIV(SUB(NOW(), active::{created_at}), 86400))
    
    // 30-day velocity
    velocity_30d <- issues
        ::WHERE(_::{closed_at}::GTE(SUB(NOW(), 2592000)))
        ::COUNT
    
    RETURN {
        label: label_name,
        total_active: active::COUNT,
        blocked_count: blocked::COUNT,
        avg_age_days: avg_age,
        velocity_30d: velocity_30d
    }
```

```hql
// Statistics aggregation
QUERY issueStats(project: String) =>
    issues <- N<Issue>
        ::WHERE(NOT(_::{ephemeral}))
        ::WHERE(OR(project::EQ(""), _::Out<IN_PROJECT>::{name}::EQ(project)))
    
    by_status <- issues::GROUP_BY(status)::COUNT
    by_type <- issues::GROUP_BY(issue_type)::COUNT
    by_priority <- issues::GROUP_BY(priority)::COUNT
    
    // This week's activity
    created_this_week <- issues
        ::WHERE(_::{created_at}::GTE(SUB(NOW(), 604800)))
        ::COUNT
    
    closed_this_week <- issues
        ::WHERE(_::{closed_at}::GTE(SUB(NOW(), 604800)))
        ::COUNT
    
    RETURN {
        by_status: by_status,
        by_type: by_type,
        by_priority: by_priority,
        created_this_week: created_this_week,
        closed_this_week: closed_this_week,
        net_change: SUB(created_this_week, closed_this_week)
    }
```

```hql
// Simple issue count with optional filters
QUERY countIssues(status: String, issue_type: String) =>
    issues <- N<Issue>
        ::WHERE(NOT(_::{ephemeral}))
        ::WHERE(OR(status::EQ(""), _::{status}::EQ(status)))
        ::WHERE(OR(issue_type::EQ(""), _::{issue_type}::EQ(issue_type)))
    
    RETURN {
        count: issues::COUNT
    }
```

### Maintenance & Cleanup

```hql
// Find issues eligible for cleanup
QUERY cleanupCandidates(older_than_days: I32) =>
    threshold <- SUB(NOW(), MUL(older_than_days, 86400))
    
    issues <- N<Issue>
        ::WHERE(_::{status}::EQ("closed"))
        ::WHERE(_::{closed_at}::LT(threshold))
        ::WHERE(NOT(_::{ephemeral}))
        // Exclude issues with open dependents
        ::WHERE(
            NOT(EXISTS(
                _::In<DEPENDS_ON>
                    ::WHERE(_::{status}::NEQ("closed"))
            ))
        )
    
    RETURN issues::{id, title, closed_at}
```

```hql
// Delete issue and cascade (for cleanup)
QUERY deleteIssueCascade(issue_id: String) =>
    issue <- N<Issue>({id: issue_id})
    
    // Delete all comments
    comments <- issue::In<COMMENT_ON>
    DELETE comments
    
    // Delete all dependency edges (both directions)
    out_deps <- issue::Out<DEPENDS_ON>
    in_deps <- issue::In<DEPENDS_ON>
    DELETE out_deps
    DELETE in_deps
    
    // Delete embeddings
    embedding <- issue::Out<HAS_EMBEDDING>
    DELETE embedding
    
    // Delete labels (edges only)
    label_edges <- issue::Out<TAGGED>
    DELETE label_edges
    
    // Delete the issue
    DELETE issue
    
    RETURN {deleted: issue_id}
```

```hql
// Merge issues: transfer relationships from sources to target
QUERY mergeIssues(source_ids: [String], target_id: String) =>
    target <- N<Issue>({id: target_id})
    
    FOR source_id IN source_ids {
        source <- N<Issue>({id: source_id})
        
        // Move comments
        comments <- source::In<COMMENT_ON>
        FOR comment IN comments {
            DELETE E<COMMENT_ON>(comment, source)
            INSERT E<COMMENT_ON>(comment, target)
        }
        
        // Transfer incoming dependencies (X blocks source → X blocks target)
        in_deps <- source::In<DEPENDS_ON>
        FOR dep IN in_deps {
            // Skip if would create self-reference
            IF dep::source::{id}::NEQ(target_id) THEN
                INSERT E<DEPENDS_ON>(dep::source, target, dep::properties)
            DELETE dep
        }
        
        // Transfer outgoing dependencies (source blocks Y → target blocks Y)
        out_deps <- source::Out<DEPENDS_ON>
        FOR dep IN out_deps {
            IF dep::target::{id}::NEQ(target_id) THEN
                INSERT E<DEPENDS_ON>(target, dep::target, dep::properties)
            DELETE dep
        }
        
        // Mark source as tombstone
        UPDATE source SET 
            status = "tombstone",
            body = CONCAT("Merged into ", target_id)
    }
    
    RETURN {
        target: target_id,
        merged: source_ids
    }
```

### System Info

```hql
// Get system statistics for info command
QUERY systemInfo() =>
    total_issues <- N<Issue>::COUNT
    open_issues <- N<Issue>::WHERE(_::{status}::EQ("open"))::COUNT
    embedded_issues <- N<Issue>::WHERE(EXISTS(_::Out<HAS_EMBEDDING>))::COUNT
    total_labels <- N<Label>::COUNT
    total_comments <- N<Comment>::COUNT
    
    // Get most recent sync (from metadata)
    // Note: This would typically be stored in a config node
    
    RETURN {
        total_issues: total_issues,
        open_issues: open_issues,
        embedded_issues: embedded_issues,
        total_labels: total_labels,
        total_comments: total_comments
    }
```

---

## Embedding Strategy

### Model Configuration

```toml
# .helix/config.toml
[embeddings]
# Backend: fastembed (local) | ollama | openai | gemini
backend = "fastembed"

# Model selection
model = "BGESmallENV15"  # 384 dimensions, ~130MB

# Cache directory for model weights
cache_dir = ".helix/models"

# Offline mode: never attempt cloud APIs
offline_only = true

# Fallback chain when primary fails
fallback = ["model2vec", "bm25_only"]

[embeddings.cloud]
# Only used if offline_only = false and local fails
provider = "openai"
model = "text-embedding-3-small"
# API key from environment: HBD_OPENAI_API_KEY
```

### Rust Integration

```rust
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use std::path::Path;

pub struct Embedder {
    model: TextEmbedding,
    model_name: String,
}

impl Embedder {
    pub fn new(cache_dir: &Path) -> Result<Self, EmbedError> {
        let model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::BGESmallENV15,
            cache_dir: Some(cache_dir.to_path_buf()),
            show_download_progress: true,
            ..Default::default()
        })?;
        
        Ok(Self {
            model,
            model_name: "fastembed:bge-small-en-v1.5".to_string(),
        })
    }
    
    pub fn embed_issue(&self, title: &str, body: &str) -> Result<Vec<f32>, EmbedError> {
        let text = format!("{}\n\n{}", title, body);
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap())
    }
    
    pub fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, EmbedError> {
        self.model.embed(texts, None)
    }
    
    pub fn model_name(&self) -> &str {
        &self.model_name
    }
    
    pub fn dimensions(&self) -> usize {
        384  // BGE-small-en-v1.5
    }
}
```

### Embedding Lifecycle

```
Issue Created
    │
    ├──▶ Compute text hash: blake3(title + body)
    │
    ├──▶ Check if embedding exists with same text_hash
    │    ├── Yes → Skip (already embedded)
    │    └── No → Continue
    │
    ├──▶ Generate embedding via fastembed
    │
    ├──▶ Store IssueEmbedding node with:
    │    - issue_id
    │    - text_hash
    │    - model name
    │    - 384-dim vector
    │
    └──▶ Create HAS_EMBEDDING edge
```

### Fallback Chain

```
1. Try fastembed (local, no network)
   ├── Model cached → Load and use
   ├── Model not cached → Download once (requires network)
   └── OOM or error → Continue to fallback

2. Try model2vec (ultra-lightweight fallback)
   ├── 30MB model, 256 dimensions
   └── Slightly lower quality but very fast

3. Try cloud provider (if API key set and offline_only=false)
   ├── OpenAI text-embedding-3-small
   └── Gemini gemini-embedding-001

4. BM25 only mode
   ├── Log warning
   ├── Skip all vector operations
   └── Queue for re-embedding later
```

---

## Git Sync Protocol

### File Layout

```
your-project/
├── .tickets/
│   ├── bd-a1b2c3.md          # Issue files
│   ├── bd-d4e5f6.md
│   └── .labels.yaml          # Label definitions
│
├── .helix/
│   ├── config.toml           # hbd configuration
│   ├── helix.db/             # HelixDB data (gitignored)
│   └── models/               # Embedding models (gitignored)
│
├── helix.toml                # HelixDB schema
└── .gitignore                # Includes .helix/helix.db/, .helix/models/
```

### Sync Algorithm

```rust
pub fn sync() -> Result<SyncResult, SyncError> {
    let mut result = SyncResult::default();
    
    // 1. Export: HelixDB → .tickets/
    for issue in db.query::<Issue>("dirty = true")? {
        let path = tickets_dir.join(format!("{}.md", issue.id));
        let content = issue.to_markdown();
        
        if path.exists() {
            let existing = fs::read_to_string(&path)?;
            let existing_hash = blake3::hash(existing.as_bytes());
            let new_hash = blake3::hash(content.as_bytes());
            
            if existing_hash != new_hash {
                fs::write(&path, &content)?;
                result.exported += 1;
            }
        } else {
            fs::write(&path, &content)?;
            result.created += 1;
        }
        
        db.mark_clean(&issue.id)?;
    }
    
    // 2. Import: .tickets/ → HelixDB
    for entry in fs::read_dir(tickets_dir)? {
        let path = entry?.path();
        if path.extension() != Some("md") { continue; }
        
        let content = fs::read_to_string(&path)?;
        let issue = Issue::from_markdown(&content)?;
        
        let content_hash = blake3::hash(content.as_bytes());
        
        match db.get_issue(&issue.id)? {
            Some(existing) if existing.content_hash == content_hash => {
                // No change, skip
            }
            Some(existing) if existing.updated_at > issue.updated_at => {
                // DB is newer, skip (will be exported on next sync)
            }
            _ => {
                // File is newer or new, import
                db.upsert_issue(&issue)?;
                result.imported += 1;
                
                // Queue for re-embedding if text changed
                if text_changed(&issue) {
                    embedder.queue_embed(&issue.id)?;
                }
            }
        }
    }
    
    Ok(result)
}
```

### Conflict Resolution

| Scenario | Resolution |
|----------|------------|
| Same ID, same content hash | Skip (already synced) |
| Same ID, DB newer | Export DB version to file |
| Same ID, file newer | Import file to DB |
| Git merge conflict | Manual resolution required |
| Concurrent creates | Hash IDs prevent collisions |

### Hash-Based IDs

```rust
pub fn generate_issue_id() -> String {
    let uuid = uuid::Uuid::new_v4();
    let hash = blake3::hash(uuid.as_bytes());
    let hex = hex::encode(&hash.as_bytes()[..3]);  // 6 hex chars
    format!("bd-{}", hex)
}
```

**Why this works:**
- UUIDs ensure uniqueness across machines
- Short hashes (6 chars) are human-friendly
- No coordination needed between branches
- Birthday paradox: ~0.1% collision chance at 1000 issues

---

## File Formats

### Issue Markdown Format

```markdown
---
id: bd-a1b2c3
title: Fix memory leak in parser
status: open
priority: 1
type: bug
created_at: 2026-01-03T10:30:00Z
updated_at: 2026-01-03T14:22:00Z
created_by: kevin
created_by_type: human
assignee: kevin
labels:
  - performance
  - parser
depends_on:
  - id: bd-x7y8z9
    type: blocks
project: helix-tools
estimated_minutes: 120
---

## Description

The parser leaks memory when processing files larger than 10MB. 
Memory usage grows linearly with each parsed file and is never freed.

## Steps to Reproduce

1. Create a 15MB test file
2. Run `hbd parse large-file.txt` in a loop
3. Observe memory growth in `htop`

## Acceptance Criteria

- [ ] No memory growth over 1000 file parses
- [ ] Add regression test with 100MB file
- [ ] Document memory limits in README
```

### Label Definitions

```yaml
# .tickets/.labels.yaml
labels:
  - name: bug
    color: "#d73a4a"
    description: Something isn't working
  
  - name: feature
    color: "#0075ca"
    description: New feature or request
  
  - name: performance
    color: "#fbca04"
    description: Performance improvement
  
  - name: documentation
    color: "#0052cc"
    description: Improvements to documentation
```

### Configuration

```toml
# .helix/config.toml

[project]
name = "helix-tools"
default_assignee = ""

[sync]
auto_sync = true
interval_seconds = 5
auto_commit = true
commit_message_template = "chore(issues): sync {action} {count} issue(s)"

[embeddings]
backend = "fastembed"
model = "BGESmallENV15"
cache_dir = ".helix/models"
offline_only = true

[search]
bm25_title_weight = 2.0
bm25_body_weight = 1.0
hybrid_rrf_k = 60
mmr_lambda = 0.7
default_limit = 20

[health]
stale_days = 14
velocity_window_days = 7

[compaction]
tier1_age_days = 30
tier2_age_days = 90
max_summary_ratio = 0.25
```

---

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum HbdError {
    #[error("Issue not found: {0}")]
    IssueNotFound(String),
    
    #[error("Cycle detected: {}", .0.join(" -> "))]
    CycleDetected(Vec<String>),
    
    #[error("Invalid issue format: {0}")]
    InvalidFormat(String),
    
    #[error("Sync conflict: {0}")]
    SyncConflict(String),
    
    #[error("Embedding failed: {0}")]
    EmbeddingError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] helix_db::GraphError),
    
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Issue not found |
| 4 | Cycle detected |
| 5 | Sync conflict |
| 10 | Embedding unavailable (warning, operation continued) |

### Error Messages

All errors follow this format:
```
error: <short description>

<detailed explanation>

hint: <suggested fix>
```

Example:
```
error: Cycle detected in dependencies

Adding bd-abc123 -> bd-xyz789 would create a cycle:
  bd-xyz789 -> bd-def456 -> bd-abc123

hint: Remove one of the existing dependencies first:
  hbd dep remove bd-xyz789 blocks bd-def456
```
