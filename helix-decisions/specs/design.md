# helix-decisions: Design Specification

**Document:** design.md  
**Status:** In Progress (2026-01-06)  
**Author:** Kevin Chen

> **Implementation Status**
>
> | Phase | Status | Description |
> |-------|--------|-------------|
> | **Phase 1-2 (Core)** | ✅ Complete | HelixDB storage, semantic search, git hooks |
> | **Phase 3 (Indexer + Daemon)** | 🚧 Planned | Incremental indexing, background sync |
>
> HelixDB is the only storage backend. Phase 3 adds incremental indexing and a daemon
> for background sync. See [Phase 3 Implementation](#phase-3-indexer-daemon-implementation) below.

## Design Philosophy

Decisions are not isolated documents—they form a **decision graph**:
- Decision 005 **supersedes** Decision 002
- Decision 007 **amends** Decision 003  
- Decision 004 **relates to** Decision 006
- Decision 008 **depends on** Decision 001

This graph structure enables powerful queries beyond simple search:
- "What's the current decision?" → Follow supersedes chain to leaf
- "Why was this changed?" → Traverse back through supersedes history
- "What else is affected?" → Find related and dependent decisions

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                helix-decisions CLI                   │
│  • search <query>     - Semantic vector search       │
│  • chain <id>         - Show supersedes chain        │
│  • related <id>       - Find related decisions       │
│  • check              - Validate decision files      │
└──────────────────────────┬──────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────┐
│                  DecisionSearcher                    │
│  • sync()    - Delta index decisions + relationships │
│  • search()  - Vector similarity + graph context     │
│  • chain()   - Traverse supersedes edges             │
│  • related() - Find connected decisions              │
└──────────────────────────┬──────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
┌────────▼─────┐   ┌───────▼───────┐   ┌─────▼──────┐
│   Loader     │   │   Embedder    │   │  Storage   │
│  (YAML/MD)   │   │  (fastembed)  │   │ (HelixDB)  │
│              │   │               │   │            │
│ • Parse      │   │ • Local embed │   │ • Nodes    │
│   decisions  │   │ • 384-dim     │   │ • Vectors  │
│ • Extract    │   │ • MiniLM-L6   │   │ • Edges    │
│   relations  │   │               │   │            │
└──────────────┘   └───────────────┘   └────────────┘
```

## Runtime Roles and Consistency

helix-decisions uses a split read/write model to keep queries fast while ensuring a single
writer to LMDB:

- **CLI (read path):** Opens read transactions and serves queries from the current index.
- **Indexer daemon (write path):** Owns all write transactions, processes a queue of
  "scan + delta sync" requests, and updates LMDB + manifest.
- **Strong consistency:** `--sync` blocks until the daemon finishes the pending sync. If the
  daemon is not running, the CLI can take the writer lock and perform a direct sync.
- **Shared infrastructure:** The daemon + IPC layer should be reusable across helix-tools
  CLIs (shared crate with a stable local protocol).

```
CLI (read) -> LMDB (read txn) -> results
CLI -> daemon queue -> daemon (single writer) -> LMDB (write txn)
```

## IPC Transport

- **Local sockets:** Use Unix domain sockets on macOS/Linux and named pipes on Windows.
- **Why:** Low-latency request/response for `--sync`, no open TCP ports, easy per-user
  permissions via filesystem ACLs.
- **Socket path (Unix):** `~/.helix/run/helixd.sock`

## IPC Protocol (v1)

helix-decisions uses the shared helixd protocol defined in
`shared/helix-daemon/specs/design.md`.

## Daemon Scope (Decision)

- **Global per-user daemon:** One helix-tools daemon per user, multiplexing requests across
  repos and tools. All requests are namespaced by `{repo_root, tool}` to keep data and locks
  scoped correctly (decisions, issues, docs, maps).
- **Why:** Fewer background processes, shared backpressure, and consistent IPC for all tools.
- **Implications:** The daemon must enforce per-repo writer locks and fair scheduling.

## Daemon Lifecycle

Lifecycle behavior is defined by helixd. See `shared/helix-daemon/specs/design.md`.

## Graph Schema

### Node: Decision

Decisions are stored as graph nodes with properties and vector embeddings.

```
┌─────────────────────────────────────────┐
│ Node: Decision                          │
├─────────────────────────────────────────┤
│ id: u128 (HelixDB internal)             │
│ label: "decision"                       │
├─────────────────────────────────────────┤
│ Properties:                             │
│   id: u32              # Local number   │
│   uuid: String         # Global hash ID │
│   title: String                         │
│   status: String       # enum as string │
│   date: String         # ISO 8601       │
│   deciders: [String]                    │
│   tags: [String]                        │
│   file_path: String                     │
│   content_hash: String # for delta      │
│   git_commit: String   # immutability   │
│   body: String         # markdown text  │
├─────────────────────────────────────────┤
│ Vector: 384-dim embedding of body       │
└─────────────────────────────────────────┘
```

## ID Scheme

### Local ID (`id`)
- Sequential integer (1, 2, 3...)
- Human-readable and easy to reference
- Unique within a single repository
- Used in filenames: `003-database-migration.md`

### Global UUID (`uuid`)
- Required hash-based identifier via helix-id
- Format: `hx-xxxxxx` (6 hex chars from Blake3 hash)
- Safe for distributed collaboration across branches
- Generated from decision content or random UUID
- Prevents merge conflicts when multiple developers create decisions
- Required in frontmatter; `helix-decisions check` enforces presence

### Why Both?
- `id`: For humans ("see decision 5")
- `uuid`: For machines and cross-repo references
- Local IDs can conflict across branches; UUIDs cannot

## Immutability Model

### Soft Immutability via Git

Decisions become immutable once accepted:

1. **content_hash**: SHA-256 of decision content at acceptance
2. **git_commit**: Git commit hash when status changed to `accepted`

### Amendment Pattern

Instead of modifying accepted decisions:
- Create new decision with `amends: [original_id]`
- Original remains unchanged for audit trail
- Search returns both, with amendment relationship visible

### Supersession Pattern

When a decision is replaced entirely:
- Create new decision with `supersedes: [old_id]`
- Set old decision status to `superseded`
- Graph traversal shows evolution chain

### Edges: Relationships

```
┌──────────────────────────────────────────────────────────┐
│ Edge Types                                                │
├───────────────┬──────────────────────────────────────────┤
│ SUPERSEDES    │ Decision A supersedes Decision B         │
│               │ Direction: A → B                          │
│               │ Inverse: B.superseded_by = A              │
├───────────────┼──────────────────────────────────────────┤
│ AMENDS        │ Decision A modifies Decision B           │
│               │ Direction: A → B                          │
│               │ (B remains valid with amendments)         │
├───────────────┼──────────────────────────────────────────┤
│ DEPENDS_ON    │ Decision A requires Decision B           │
│               │ Direction: A → B                          │
│               │ (A assumes B is accepted)                 │
├───────────────┼──────────────────────────────────────────┤
│ RELATED_TO    │ Decision A and B are topically related   │
│               │ Direction: bidirectional (A ↔ B)         │
│               │ (informational link only)                 │
└───────────────┴──────────────────────────────────────────┘
```

### Example Graph

```
     001 (Database Choice)
         │
         │ SUPERSEDES
         ▼
     003 (PostgreSQL Selection)
         │
    ┌────┴────┐
    │         │
 AMENDS    RELATED_TO
    │         │
    ▼         ▼
   007       004
(Indexes) (Caching)
              │
          DEPENDS_ON
              │
              ▼
             006
        (Redis Choice)
```

## Module Design

### types.rs
```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Decision status values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Proposed,
    Accepted,
    Superseded,
    Deprecated,
}

/// Relationship types between decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// This decision replaces another (makes it obsolete)
    Supersedes,
    /// This decision modifies another without replacing it
    Amends,
    /// This decision requires another's decision to be in effect
    DependsOn,
    /// This decision is topically related to another
    RelatedTo,
}

impl RelationType {
    /// Edge label for HelixDB storage
    pub fn as_edge_label(&self) -> &'static str {
        match self {
            Self::Supersedes => "SUPERSEDES",
            Self::Amends => "AMENDS",
            Self::DependsOn => "DEPENDS_ON",
            Self::RelatedTo => "RELATED_TO",
        }
    }
}

/// A relationship from this decision to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relation_type: RelationType,
    pub target_id: u32,
}

/// Decision metadata from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMetadata {
    pub id: u32,
    #[serde(default)]
    pub uuid: Option<String>,  // Global hash-based ID
    pub title: String,
    pub status: Status,
    pub date: NaiveDate,
    #[serde(default)]
    pub deciders: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content_hash: Option<String>,  // For immutability proof
    #[serde(default)]
    pub git_commit: Option<String>,    // Commit when accepted
    
    // Relationship fields (all optional, can be single ID or array)
    #[serde(default)]
    pub supersedes: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub superseded_by: Option<u32>,  // Inverse, usually auto-set
    #[serde(default)]
    pub amends: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub depends_on: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub related_to: Option<OneOrMany<u32>>,
}

/// Helper for YAML fields that can be single value or array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T: Clone> OneOrMany<T> {
    pub fn to_vec(&self) -> Vec<T> {
        match self {
            Self::One(v) => vec![v.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

impl DecisionMetadata {
    /// Extract all outgoing relationships from metadata
    pub fn relationships(&self) -> Vec<Relationship> {
        let mut rels = Vec::new();
        
        if let Some(ref ids) = self.supersedes {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::Supersedes,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.amends {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::Amends,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.depends_on {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::DependsOn,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.related_to {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::RelatedTo,
                    target_id: id,
                });
            }
        }
        
        rels
    }
}

/// Full decision with body and computed fields
#[derive(Debug, Clone)]
pub struct Decision {
    pub metadata: DecisionMetadata,
    pub body: String,
    pub file_path: PathBuf,
    pub content_hash: String,
    pub embedding: Option<Vec<f32>>,
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub id: u32,
    pub uuid: Option<String>,
    pub title: String,
    pub status: Status,
    pub score: f32,
    pub tags: Vec<String>,
    pub date: NaiveDate,
    pub deciders: Vec<String>,
    pub file_path: PathBuf,
    /// Related decisions found via graph traversal
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedDecision>,
}

/// Minimal info for related decision references
#[derive(Debug, Clone, Serialize)]
pub struct RelatedDecision {
    pub id: u32,
    pub title: String,
    pub relation: RelationType,
}

/// Search response envelope
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub count: usize,
    pub results: Vec<SearchResult>,
}

/// Chain response for supersedes traversal
#[derive(Debug, Serialize)]
pub struct ChainResponse {
    pub root_id: u32,
    pub chain: Vec<ChainNode>,
}

#[derive(Debug, Serialize)]
pub struct ChainNode {
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub date: NaiveDate,
    pub is_current: bool,  // true for leaf (not superseded)
}
```

### loader.rs
```rust
use crate::types::{Decision, DecisionMetadata};
use anyhow::Result;
use gray_matter::{engine::YAML, Matter};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Load all decisions from a directory
pub fn load_decisions(dir: &Path) -> Result<Vec<Decision>> {
    let mut decisions = Vec::new();
    let matter = Matter::<YAML>::new();
    
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "md") {
            match load_decision(&path, &matter) {
                Ok(decision) => decisions.push(decision),
                Err(e) => eprintln!("Warning: Skipping {}: {}", path.display(), e),
            }
        }
    }
    
    Ok(decisions)
}

fn load_decision(path: &Path, matter: &Matter<YAML>) -> Result<Decision> {
    let content = std::fs::read_to_string(path)?;
    let parsed = matter.parse(&content);
    
    let metadata: DecisionMetadata = parsed
        .data
        .ok_or_else(|| anyhow::anyhow!("Missing frontmatter"))?
        .deserialize()?;
    
    let body = parsed.content;
    let content_hash = hash_content(&content);
    
    Ok(Decision {
        metadata,
        body,
        file_path: path.to_path_buf(),
        content_hash,
        embedding: None,
    })
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### embeddings.rs
```rust
use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2))?;
        Ok(Self { model })
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap())
    }
    
    pub fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        self.model.embed(texts, None).map_err(Into::into)
    }
}
```

### storage.rs (HelixDB Implementation)

```rust
use crate::helix_backend::HelixDecisionBackend;
use crate::types::{ChainNode, Decision, RelatedDecision};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub trait DecisionStorage: Send + Sync {
    fn index(&mut self, decisions: Vec<Decision>) -> Result<()>;
    fn remove(&mut self, paths: Vec<String>) -> Result<()>;
    fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Decision, f32)>>;
    fn get_hashes(&self) -> Result<HashMap<String, String>>;
    fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>>;
    fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>>;
}

pub struct HelixDecisionStorage {
    backend: HelixDecisionBackend,
}

impl HelixDecisionStorage {
    pub fn open(repo_root: &Path) -> Result<Self> {
        Ok(Self {
            backend: HelixDecisionBackend::new(repo_root)?,
        })
    }
}
```

### delta.rs
```rust
use crate::types::Decision;
use std::collections::HashMap;

/// Delta detection result
pub struct DeltaResult {
    pub to_add: Vec<Decision>,
    pub to_remove: Vec<String>,
}

/// Compute delta between filesystem and indexed decisions
pub fn compute_delta(
    current_decisions: Vec<Decision>,
    stored_hashes: HashMap<String, String>,
) -> DeltaResult {
    let mut to_add = Vec::new();
    let mut to_remove = Vec::new();
    
    // Track which stored paths we've seen
    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    for decision in current_decisions {
        let path = decision.file_path.to_string_lossy().to_string();
        seen_paths.insert(path.clone());
        
        match stored_hashes.get(&path) {
            Some(stored_hash) if stored_hash == &decision.content_hash => {
                // No change, skip
            }
            _ => {
                // New or changed, need to re-index
                to_add.push(decision);
            }
        }
    }
    
    // Find deleted decisions
    for path in stored_hashes.keys() {
        if !seen_paths.contains(path) {
            to_remove.push(path.clone());
        }
    }
    
    DeltaResult { to_add, to_remove }
}
```

### searcher.rs
```rust
use crate::delta::compute_delta;
use crate::embeddings::Embedder;
use crate::loader::load_decisions;
use crate::storage::{DecisionStorage, HelixDBStorage};
use crate::types::{SearchResponse, SearchResult, Status};
use anyhow::Result;
use std::path::Path;

pub struct DecisionSearcher {
    storage: Box<dyn DecisionStorage>,
    embedder: Embedder,
}

impl DecisionSearcher {
    pub fn new() -> Result<Self> {
        let storage = Box::new(HelixDBStorage::open()?);
        let embedder = Embedder::new()?;
        Ok(Self { storage, embedder })
    }
    
    /// Load and sync decisions from directory
    pub fn sync(&mut self, dir: &Path) -> Result<()> {
        // Load current decisions
        let decisions = load_decisions(dir)?;
        
        // Get stored hashes
        let stored_hashes = self.storage.get_hashes()?;
        
        // Compute delta
        let delta = compute_delta(decisions, stored_hashes);
        
        // Remove deleted decisions
        if !delta.to_remove.is_empty() {
            self.storage.remove(delta.to_remove)?;
        }
        
        // Embed and index new/changed decisions
        if !delta.to_add.is_empty() {
            let mut decisions_with_embeddings = Vec::new();
            for mut decision in delta.to_add {
                let embedding = self.embedder.embed(&decision.body)?;
                decision.embedding = Some(embedding);
                decisions_with_embeddings.push(decision);
            }
            self.storage.index(decisions_with_embeddings)?;
        }
        
        Ok(())
    }
    
    /// Search for decisions matching query
    pub fn search(
        &self,
        query: &str,
        limit: usize,
        status_filter: Option<Status>,
        tags_filter: Option<Vec<String>>,
    ) -> Result<SearchResponse> {
        // Embed query
        let query_embedding = self.embedder.embed(query)?;
        
        // Search storage
        let results = self.storage.search(query_embedding, limit * 2)?;  // Over-fetch for filtering
        
        // Filter and convert
        let mut search_results: Vec<SearchResult> = results
            .into_iter()
            .filter(|(decision, _)| {
                // Status filter
                if let Some(ref status) = status_filter {
                    if &decision.metadata.status != status {
                        return false;
                    }
                }
                // Tags filter
                if let Some(ref tags) = tags_filter {
                    if !tags.iter().all(|t| decision.metadata.tags.contains(t)) {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .map(|(decision, score)| SearchResult {
                id: decision.metadata.id,
                uuid: decision.metadata.uuid,
                title: decision.metadata.title,
                status: decision.metadata.status,
                score,
                tags: decision.metadata.tags,
                date: decision.metadata.date,
                deciders: decision.metadata.deciders,
                file_path: decision.file_path,
            })
            .collect();
        
        search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        Ok(SearchResponse {
            query: query.to_string(),
            count: search_results.len(),
            results: search_results,
        })
    }
}
```

## Data Flow

### First Invocation (Cold Start)
```
1. CLI parses args (query, options)
2. DecisionSearcher::new() opens HelixDB (creates if needed)
3. CLI acquires writer lock (or waits for daemon) and runs a full sync
4. Delta check finds all decisions are new
5. Embed all decisions with fastembed (~2-5s)
6. Store decisions as vectors in HelixDB (with properties)
7. Create relationship edges from frontmatter
8. DecisionSearcher::search(query) embeds query
9. HelixDB vector search returns ranked results
10. Optionally enrich with graph context
11. CLI formats and outputs results
```

### Subsequent Invocations (Warm)
```
1. CLI parses args
2. DecisionSearcher::new() opens existing HelixDB
3. CLI serves query from existing index immediately
4. CLI enqueues "scan + delta sync" request to daemon
5. Daemon performs delta sync (re-embed changed, update edges, handle deletes/renames)
6. Search proceeds as normal (~100ms total)
```

### Graph Traversal (chain/related commands)
```
1. CLI parses args with decision ID
2. DecisionSearcher::new() opens existing HelixDB
3. No sync needed for read-only graph queries
4. Traverse edges from specified decision
5. Return connected decisions with relationship info
```

## Query Examples

### Semantic Search
```bash
# Find decisions about caching
helix-decisions search "caching strategy"

# With graph context (show related decisions)
helix-decisions search "caching strategy" --with-related

# Filter by status
helix-decisions search "database" --status accepted
```

### Graph Queries
```bash
# Show the evolution of a decision (supersedes chain)
helix-decisions chain 2
# Output: 002 → 005 → 008 (current)

# Find all decisions related to a specific one
helix-decisions related 5
# Output: 
#   supersedes: 002
#   amended_by: 007
#   related_to: 004, 006

# Find the current decision (follow supersedes to leaf)
helix-decisions chain 2
# Output: 008 (the leaf of the chain starting at 2)
```

### JSON Output (for agents)
```bash
helix-decisions search "authentication" --json
```
```json
{
  "query": "authentication",
  "count": 2,
  "results": [
    {
      "id": 4,
      "title": "JWT Authentication",
      "status": "accepted",
      "score": 0.89,
      "tags": ["auth", "security"],
      "date": "2026-01-03",
      "related": [
        {"id": 1, "title": "API Design", "relation": "depends_on"}
      ]
    }
  ]
}
```

## Storage Schema

### Storage (HelixDB)

See [Phase 3: Indexer + Daemon Implementation](#phase-3-indexer-daemon-implementation) for the corrected
graph schema with proper arena allocation, 3-DB edge writes, and vector mapping.

```
.helix/data/decisions/
├── data.mdb         # LMDB data file (nodes, edges, vectors, metadata)
└── lock.mdb         # LMDB lock file
```

### Edge Types

| Edge Label | Direction | Meaning |
|------------|-----------|---------|
| `SUPERSEDES` | A → B | A replaces B (B is obsolete) |
| `AMENDS` | A → B | A modifies B (B still valid) |
| `DEPENDS_ON` | A → B | A requires B to be accepted |
| `RELATED_TO` | A ↔ B | Bidirectional topic relationship |

### Decision Frontmatter Format

```yaml
---
id: 5
uuid: hx-a1b2c3             # Required: hash-based UUID for rename safety
title: PostgreSQL for Primary Database
status: accepted
date: 2026-01-04
deciders:
  - Alice
  - Bob
tags:
  - database
  - infrastructure
content_hash: abc123...     # Optional: for immutability proof
git_commit: def456...       # Optional: commit when accepted

# Relationships (all optional, can be single ID or array)
supersedes: 2               # This decision replaces decision 2
amends: [3, 4]              # This decision modifies decisions 3 and 4
depends_on: 1               # This decision assumes decision 1 is accepted
related_to: [6, 7]          # Related but not dependent
---

# Context and Problem Statement
...
```

## Embedding Model

Using `fastembed` with `AllMiniLML6V2`:
- 384 dimensions
- ~50ms per embedding (CPU)
- Good semantic understanding
- Small model size (~90MB)

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| First sync | 2-5s | Embedding 100 decisions |
| Delta sync | < 50ms | Hash comparison |
| Query embed | 50-100ms | Single text |
| Vector search | < 50ms | HelixDB |
| Graph traversal | < 50ms | Chain/related |
| Total search | < 100ms | After first run |

## Error Handling

| Error | Behavior |
|-------|----------|
| Missing directory | Exit 2 with message |
| Malformed YAML | Warn, skip file |
| HelixDB error | Exit 2 with message |
| Embedding error | Exit 2 with message |
| No results | Exit 1, show "No results" |

---

## Phase 3: Indexer + Daemon Implementation

> **Status:** Planned  
> **Documents:** See `docs/phase3/PHASE_3_PLAN.md` and `docs/phase3/PHASE_3_CORRECTIONS.md`

Phase 3 focuses on incremental indexing and background sync on top of HelixDB:
- **Incremental indexing** via 3-stage change detection
- **Native graph traversal** for chain/related queries
- **Daemonized sync** for single-writer guarantees and low-latency queries

### Architecture (Phase 3)

```
┌─────────────────────────────────────────────────────────────────┐
│                    helix-decisions CLI                           │
│  • search <query>     - Semantic vector search                   │
│  • chain <id>         - Show supersedes chain                    │
│  • related <id>       - Find related decisions                   │
│  • check              - Validate decision files                  │
└──────────────────────────────┬──────────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────────┐
│                     DecisionSearcher                             │
│  • sync()    - 3-stage incremental indexing                      │
│  • search()  - Vector similarity via HNSW                        │
│  • chain()   - Traverse out_edges_db for SUPERSEDES              │
│  • related() - Query all edge types (both directions)            │
└──────────────────────────────┬──────────────────────────────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         │                     │                     │
┌────────▼─────────┐   ┌───────▼───────┐   ┌────────▼────────┐
│   git_utils.rs   │   │  Embedder     │   │ HelixDB Backend │
│                  │   │ (fastembed)   │   │                 │
│ • git ls-files   │   │               │   │ • nodes_db      │
│ • Respects       │   │ • 384-dim     │   │ • edges_db      │
│   .gitignore     │   │ • f32 → f64   │   │ • out_edges_db  │
│                  │   │               │   │ • in_edges_db   │
└──────────────────┘   └───────────────┘   │ • vectors (HNSW)│
                                           │ • metadata_db   │
                                           └─────────────────┘
```

### Graph Schema (Phase 3 - Corrected)

**Node Label:** `DECISION`

Nodes are stored using arena-allocated labels and `ImmutablePropertiesMap`:

```rust
Node<'arena> {
    id: u128,                    // HelixDB node ID (UUID v4)
    label: &'arena str,          // "DECISION" (arena-allocated)
    version: u8,                 // Schema version
    properties: Option<ImmutablePropertiesMap<'arena>>,
}

// Properties stored in node:
{
    "id": i64,                   // Local sequential ID (1, 2, 3...)
    "title": String,
    "status": String,            // "proposed"|"accepted"|"superseded"|"deprecated"
    "date": String,              // ISO 8601
    "file_path": String,
    "content_hash": String,      // SHA256 of file content
    "tags": String,              // JSON array as string
    "deciders": String,          // JSON array as string
    "vector_id": String,         // UUID of associated vector (for search mapping)
}
```

**Edge Storage (3 Databases per Edge):**

Edges MUST be written to THREE databases for traversal to work:

```rust
// For edge: A --[SUPERSEDES]--> B

// 1. Edge data
edges_db.put(edge_key(&edge_id), edge.to_bincode_bytes()?)

// 2. Outgoing adjacency (for traversal FROM node A)
let label_hash = hash_label("SUPERSEDES", None);
out_edges_db.put(
    out_edge_key(&node_a_id, &label_hash),
    pack_edge_data(edge_id, node_b_id)
)

// 3. Incoming adjacency (for traversal TO node A)
in_edges_db.put(
    in_edge_key(&node_b_id, &label_hash),
    pack_edge_data(edge_id, node_a_id)
)
```

**Vector Storage:**

Vectors are stored separately from nodes. The `vector_id` property links them:

```rust
// 1. Insert vector (generates its own UUID)
let vector_id = Uuid::new_v4().as_u128();
let embedding_f64: Vec<f64> = embedding.iter().map(|&x| x as f64).collect();
vectors.insert(&mut wtxn, vector_id, &embedding_f64)?;

// 2. Store vector_id in node properties for mapping
properties.push(("vector_id", Value::String(arena.alloc(vector_id.to_string()))));

// 3. Create secondary index on vector_id for reverse lookup
storage.create_secondary_index("vector_id")?;
```

**Manifest (in metadata_db):**

```rust
const MANIFEST_KEY: &str = "manifest:helix-decisions:v1";

#[derive(Serialize, Deserialize)]
pub struct ManifestEntry {
    pub file_path: String,
    pub mtime: u64,
    pub size: u64,
    pub content_hash: String,
    pub node_id: u128,           // HelixDB node ID
    pub vector_id: u128,         // HNSW vector ID
    pub embedding_model: String,
    pub indexer_version: String,
}

pub struct IndexManifest {
    pub entries: HashMap<String, ManifestEntry>,
}
```

### Rename/Delete Detection and Embedding Reuse

- **Rename detection (uuid-only):** If a stored entry disappears but a new file appears with
  the same frontmatter `uuid` and identical `content_hash`, treat it as a rename. Update only
  the `file_path` property and manifest entry (no re-embedding). If `uuid` is missing, treat
  the change as delete + add.
- **Embedding reuse:** Persist `vector_id` and `embedding_model` in the manifest. If the
  `content_hash` is unchanged and the model matches, reuse the existing vector.
- **Deletion:** If no rename match exists, tombstone the node and vector and remove the
  manifest entry.

### Path Normalization and Identity Rules

- **Path normalization:** Store repo-root-relative paths using `/` separators and no `.`/`..`
  segments. Normalize incoming paths before manifest lookup.
- **Identity:** `uuid` is the stable identity and is required for rename optimization.
- **Conflicts:** If two files share the same `uuid` but different `content_hash`, abort the
  sync with a hard error. If multiple files share the same `uuid` and hash, also abort
  (ambiguous).

### 3-Stage Incremental Indexing

```
sync() {
    1. Load manifest from metadata_db
    2. Get file list via git ls-files
    3. For each file:
    
       ╔══ STAGE 1: Stat Check (FAST) ════════════════════════════╗
       ║ if file.mtime == manifest.mtime && file.size == manifest.size:
       ║     SKIP (no change)
       ╚══════════════════════════════════════════════════════════╝
                          ↓
       ╔══ STAGE 2: Content Hash (SLOWER) ════════════════════════╗
       ║ content_hash = sha256(file_content)
       ║ if content_hash == manifest.content_hash:
       ║     UPDATE mtime+size in manifest
       ║     SKIP embedding (content unchanged)
       ╚══════════════════════════════════════════════════════════╝
                          ↓
       ╔══ STAGE 3: Full Re-index (THOROUGH) ═════════════════════╗
       ║ Parse YAML frontmatter
       ║ Generate embedding (384-dim, f32 → f64)
       ║ Upsert decision node (arena + ImmutablePropertiesMap)
       ║ Replace vector in place using existing vector_id when possible
       ║ Create relationship edges (3 DBs per edge)
       ╚══════════════════════════════════════════════════════════╝
    
    4. Attempt rename match for removed files before deletion
    5. Delete nodes + vectors for removed files (no rename match)
    6. Save manifest back to metadata_db
}
```

### Batching and Transactions

- A sync is **not** globally atomic. It runs in phases with batched write transactions.
- **Phase 1 (nodes + vectors):** Upsert nodes/vectors in batches of size `N` (configurable).
- **Phase 2 (edges):** After **all** node/vector batches complete, update edges in batches.
- Each batch is a single LMDB write transaction. On batch failure, abort that batch and
  return an error; previous committed batches remain valid.
- Manifest entries are updated only for successfully committed batches. Unfinished files
  remain pending for the next sync.

**Batch size default:** `N = 100` unless overridden by config or environment.

### Edge Update Semantics

- For each changed decision, remove all outgoing edges for that node, then re-create edges
  from current frontmatter in the same batch transaction.
- Edge updates are performed after all node upserts to avoid edges pointing at missing nodes.
- **Edge removal mechanism:** Use `out_edges_db` to scan and delete all outgoing edges for
  the node across all relation labels, then delete the corresponding edge records from
  `edges_db` and `in_edges_db`.

### Vector Update Semantics

- On content change, update the vector **in place** using the existing `vector_id` when the
  backend supports it.
- **Fallback (HelixDB current API):** `drop_vector(old_id)` → `insert(new_vector)` which
  generates a new UUID. Update the node `vector_id` property and manifest entry to the new
  ID within the same batch transaction.

### Sync Stats

```
SyncStats {
  scanned: u32,
  added: u32,
  modified: u32,
  deleted: u32,
  renamed: u32,
  unchanged: u32,
  errors: u32,
  duration_ms: u64
}
```

### Module: helix_backend.rs (Phase 3)

```rust
use bumpalo::Bump;
use helix_db::{
    helix_engine::{
        storage_core::HelixGraphStorage,
        traversal_core::{HelixGraphEngine, HelixGraphEngineOpts, config::Config},
    },
    utils::{items::{Node, Edge}, properties::ImmutablePropertiesMap, label_hash::hash_label},
    protocol::value::Value,
};
use uuid::Uuid;

pub struct HelixDecisionBackend {
    engine: HelixGraphEngine,
    manifest: IndexManifest,
    embedding_model: String,
}

impl HelixDecisionBackend {
    pub fn new(repo_root: &Path) -> Result<Self> {
        // Determine DB path (respect HELIX_DB_PATH env var)
        let db_path = std::env::var("HELIX_DB_PATH")
            .unwrap_or_else(|_| {
                repo_root
                    .join(".helix")
                    .join("data")
                    .join("decisions")
                    .to_string_lossy()
                    .to_string()
            });
        
        // Create engine with path passed through opts
        let opts = HelixGraphEngineOpts {
            path: db_path,
            config: Config {
                vector_config: Some(VectorConfig {
                    m: Some(16),
                    ef_construction: Some(128),
                    ef_search: Some(64),
                }),
                graph_config: Some(GraphConfig {
                    secondary_indices: Some(vec![
                        "id".to_string(),
                        "vector_id".to_string(),
                    ]),
                }),
                db_max_size_gb: Some(1),
                ..Default::default()
            },
            version_info: VersionInfo::default(),
        };
        
        let engine = HelixGraphEngine::new(opts)?;
        let manifest = IndexManifest::load(&engine)?;
        
        Ok(Self {
            engine,
            manifest,
            embedding_model: "BAAI/bge-small-en-v1.5".to_string(),
        })
    }
    
    /// Upsert a decision node with proper arena allocation
    fn upsert_decision_node(
        &mut self,
        decision: &Decision,
        embedding: &[f32],
    ) -> Result<(u128, u128)> {
        let arena = Bump::new();
        let mut wtxn = self.engine.storage.graph_env.write_txn()?;
        
        // 1. Insert vector first (get vector_id)
        let vector_id = Uuid::new_v4().as_u128();
        let embedding_f64: Vec<f64> = embedding.iter().map(|&x| x as f64).collect();
        self.engine.storage.vectors.insert(&mut wtxn, vector_id, &embedding_f64)?;
        
        // 2. Build properties in arena
        let mut props = Vec::new();
        props.push(("id", Value::I64(decision.metadata.id as i64)));
        props.push(("title", Value::String(arena.alloc_str(&decision.metadata.title))));
        props.push(("status", Value::String(arena.alloc_str(&decision.metadata.status.to_string()))));
        props.push(("date", Value::String(arena.alloc_str(&decision.metadata.date.to_string()))));
        props.push(("file_path", Value::String(arena.alloc_str(&decision.file_path.to_string_lossy()))));
        props.push(("content_hash", Value::String(arena.alloc_str(&decision.content_hash))));
        props.push(("vector_id", Value::String(arena.alloc_str(&vector_id.to_string()))));
        // ... tags, deciders as JSON strings ...
        
        let properties = ImmutablePropertiesMap::from_items(props, &arena)?;
        
        // 3. Create node with arena-allocated label
        let node_id = Uuid::new_v4().as_u128();
        let label = arena.alloc_str("DECISION");
        let node = Node {
            id: node_id,
            label,
            version: 1,
            properties: Some(properties),
        };
        
        // 4. Store node using key helper
        let key = HelixGraphStorage::node_key(&node_id);
        self.engine.storage.nodes_db.put(&mut wtxn, &key, &node.to_bincode_bytes()?)?;
        
        wtxn.commit()?;
        Ok((node_id, vector_id))
    }
    
    /// Create relationship edges (writes to 3 databases)
    fn create_relationship_edges(
        &mut self,
        from_node_id: u128,
        metadata: &DecisionMetadata,
    ) -> Result<()> {
        let arena = Bump::new();
        let mut wtxn = self.engine.storage.graph_env.write_txn()?;
        
        for rel in metadata.relationships() {
            // Look up target node_id from manifest
            if let Some(target_entry) = self.find_node_by_id(rel.target_id) {
                let to_node_id = target_entry.node_id;
                let edge_id = Uuid::new_v4().as_u128();
                
                // Create edge struct
                let edge_label = arena.alloc_str(rel.relation_type.as_edge_label());
                let edge = Edge {
                    id: edge_id,
                    label: edge_label,
                    version: 1,
                    from_node: from_node_id,
                    to_node: to_node_id,
                    properties: None,
                };
                
                // 1. Write edge data
                let edge_key = HelixGraphStorage::edge_key(&edge_id);
                self.engine.storage.edges_db.put(&mut wtxn, &edge_key, &edge.to_bincode_bytes()?)?;
                
                // 2. Write outgoing adjacency
                let label_hash = hash_label(edge_label, None);
                let out_key = HelixGraphStorage::out_edge_key(&from_node_id, &label_hash);
                let out_val = pack_edge_data(edge_id, to_node_id);
                self.engine.storage.out_edges_db.put(&mut wtxn, &out_key, &out_val)?;
                
                // 3. Write incoming adjacency
                let in_key = HelixGraphStorage::in_edge_key(&to_node_id, &label_hash);
                let in_val = pack_edge_data(edge_id, from_node_id);
                self.engine.storage.in_edges_db.put(&mut wtxn, &in_key, &in_val)?;
            }
        }
        
        wtxn.commit()?;
        Ok(())
    }
    
    /// Delete a decision node and its vector
    fn delete_decision_node(&mut self, node_id: u128, vector_id: u128) -> Result<()> {
        let mut wtxn = self.engine.storage.graph_env.write_txn()?;
        
        // 1. Delete node (drops edges + indices)
        self.engine.storage.drop_node(&mut wtxn, &node_id)?;
        
        // 2. Tombstone vector
        self.engine.storage.drop_vector(&mut wtxn, &vector_id)?;
        
        wtxn.commit()?;
        Ok(())
    }
    
    /// Search vectors and map back to decisions
    pub fn search(&self, embedding: &[f32], limit: usize) -> Result<Vec<(Decision, f32)>> {
        let arena = Bump::new();
        let rtxn = self.engine.storage.graph_env.read_txn()?;
        
        // Convert f32 → f64 for HNSW
        let query_f64: Vec<f64> = embedding.iter().map(|&x| x as f64).collect();
        
        // Search vectors
        let vector_results = self.engine.storage.vectors.search(&rtxn, &query_f64, limit)?;
        
        // Map vector_id → node → Decision
        let mut results = Vec::new();
        for result in vector_results {
            // Lookup node by vector_id secondary index
            if let Some(node) = self.lookup_node_by_vector_id(&rtxn, result.id, &arena)? {
                let decision = self.node_to_decision(&node)?;
                results.push((decision, result.distance as f32));
            }
        }
        
        Ok(results)
    }
}
```

### Performance Targets (Phase 3)

| Operation | Phase 1-2 | Phase 3 | Notes |
|-----------|-----|---------|-------|
| First sync | 2-5s | 2-5s | Embedding dominates |
| Delta sync (no changes) | ~100ms | <50ms | 3-stage skip |
| Delta sync (1 file changed) | ~500ms | <100ms | Single re-embed |
| Query embedding | 50-100ms | 50-100ms | fastembed unchanged |
| Vector search | <100ms | <50ms | HNSW optimized |
| Graph traversal | N/A (in-memory) | <50ms | Native LMDB |
| Total search | <200ms | <100ms | After first run |

### Index Location

```
your-repo/
├── .decisions/          # Source of truth (Markdown files)
│   ├── 001-arch.md
│   └── 002-db.md
└── .helix/
    └── data/decisions/  # HelixDB storage
        ├── data.mdb     # LMDB data file
        └── lock.mdb     # LMDB lock file
```

### Storage Initialization

```rust
pub fn open_storage() -> Result<Box<dyn DecisionStorage>> {
    let repo_root = find_git_root()?;
    Ok(Box::new(HelixDecisionStorage::open(&repo_root)?))
}
```

### Key Corrections from HelixDB API Review

See `docs/phase3/PHASE_3_CORRECTIONS.md` for full details. Key issues fixed:

| Issue | Correction |
|-------|-----------|
| Edge insertion | Must write to 3 DBs (edges_db, out_edges_db, in_edges_db) |
| Node construction | Must use arena allocation + ImmutablePropertiesMap |
| Vector insertion | HNSW generates UUID; store vector_id in node properties |
| Vector deletion | Must tombstone both node and vector |
| Label hashing | Must hash labels for adjacency DB keys |
| Config path | Must plumb through HelixGraphEngineOpts.path |
| Secondary indices | Must create explicitly for id, vector_id |
| Metadata namespace | Use "manifest:helix-decisions:v1" to avoid collisions |
