# PHASE 3: HelixDB Integration Plan

**Objective:** Replace `JsonFileBackend` with HelixDB backend using incremental indexing via manifest-in-metadata pattern.

**Timeline Estimate:** 2-3 sessions (~6-8 hours total)

**Key Principle:** HelixDB as query cache + vector store, Markdown files as source of truth.

---

## A. Architecture Overview

### Current State (MVP - Phase 2)
- Storage: `JsonFileBackend` (simple JSON files at `.helix/data/decisions/`)
- Persistence: Decision objects serialized to JSON
- Indexing: Full scan on every `sync()` call
- Performance: ~100ms delta-check (slow for large repos)

### Target State (Phase 3 - HelixDB)
- Storage: `HelixDB` (LMDB-backed graph database at `.helixdb/`)
- Persistence: Decision nodes + relationship edges in HelixDB
- Indexing: Incremental via manifest-in-metadata pattern
- Performance: ~50-100ms total (only changed files re-indexed)

### Graph Schema

**Node Label:** `DECISION` (one node per decision file)
```
Properties:
  id: u32                      # Local sequential ID (1, 2, 3...)
  title: string                # Decision title
  status: string               # proposed|accepted|superseded|deprecated
  date: string                 # ISO 8601 date
  file_path: string            # Relative path from git root (.decisions/001-*.md)
  content_hash: string         # SHA256 hash of file content
  embedding: [f32; 384]        # Vector embedding (fastembed)
  tags: [string]               # Tags (list)
  deciders: [string]           # Deciders (list)
  node_id: string              # HelixDB node ID (uuid format, stored in metadata)
```

**Edge Labels:**
- `SUPERSEDES` - decision A supersedes B
- `AMENDS` - decision A amends B
- `DEPENDS_ON` - decision A depends on B
- `RELATED_TO` - decision A related to B
- `SUPERSEDED_BY` - inverse of SUPERSEDES

**Manifest (stored in metadata):**
```
Key: "manifest:decisions"
Value:
  {
    "entries": [
      {
        "file_path": ".decisions/001-initial-architecture.md",
        "mtime": 1704585600,
        "size": 2048,
        "content_hash": "abc123...",
        "node_id": "550e8400-e29b-41d4-a716-446655440000",
        "embedding_model": "BAAI/bge-small-en-v1.5",
        "indexer_version": "1"
      }
    ]
  }
```

---

## B. Implementation Plan (4 Modules)

### Module 1: `manifest.rs` (250 lines)
Tracks file state for incremental indexing.

**Types:**
```rust
pub struct ManifestEntry {
    pub file_path: String,
    pub mtime: u64,
    pub size: u64,
    pub content_hash: String,
    pub node_id: u128,              // HelixDB node ID
    pub embedding_model: String,
    pub indexer_version: String,
}

pub struct IndexManifest {
    pub entries: HashMap<String, ManifestEntry>,
    pub created_at: i64,
}
```

**Methods:**
```rust
impl IndexManifest {
    pub fn load(engine: &HelixGraphEngine) -> Result<Self>
    pub fn save(&self, engine: &HelixGraphEngine) -> Result<()>
    
    pub fn get(&self, path: &str) -> Option<&ManifestEntry>
    pub fn contains(&self, path: &str) -> bool
    pub fn upsert(&mut self, entry: ManifestEntry)
    pub fn remove(&mut self, path: &str)
    pub fn entries(&self) -> impl Iterator<Item = (&String, &ManifestEntry)>
}
```

**JSON schema stored in metadata:**
```json
{
  "entries": {
    ".decisions/001-architecture.md": {
      "mtime": 1704585600,
      "size": 2048,
      "content_hash": "abc123...",
      "node_id": "550e8400-e29b-41d4-a716-446655440000",
      "embedding_model": "BAAI/bge-small-en-v1.5",
      "indexer_version": "1"
    }
  }
}
```

---

### Module 2: `git_utils.rs` (100 lines)
Fast file enumeration using git.

**Functions:**
```rust
pub fn git_ls_files(
    repo_root: &Path,
    glob: &str,
) -> Result<Vec<PathBuf>>

// Internal helper
fn run_git_command(
    cwd: &Path,
    args: &[&str],
) -> Result<String>
```

**Implementation:**
- Runs `git ls-files '.decisions/**/*.md'` (respects .gitignore)
- Falls back to directory walk if git unavailable
- Returns sorted Vec<PathBuf>

**Usage Example:**
```rust
let files = git_ls_files(&repo_root, ".decisions/**/*.md")?;
// Returns [.decisions/001-arch.md, .decisions/002-db.md, ...]
```

---

### Module 3: `helix_backend.rs` (600 lines)
Core HelixDB integration with incremental indexing.

**Main Struct:**
```rust
pub struct HelixDecisionBackend {
    engine: HelixGraphEngine,
    embedder: Embedder,
    manifest: IndexManifest,
    embedding_model: String,
}
```

**Key Methods:**

1. **Constructor & Loading**
   ```rust
   pub fn new(repo_root: &Path) -> Result<Self>
   pub fn load_manifest() -> Result<IndexManifest>
   ```

2. **Incremental Indexing** (main logic)
   ```rust
   pub fn sync(&mut self, decisions_dir: &Path) -> Result<SyncStats> {
       // 1. Load manifest from metadata
       // 2. Get file list via git ls-files
       // 3. Compute delta (add, modify, delete)
       // 4. For each file:
       //    a. Stat check (mtime+size)
       //    b. Hash check (content_hash)
       //    c. If changed: parse, embed, upsert node+edges
       //    d. If deleted: delete node and edges
       // 5. Save manifest back to metadata
       
       // Return stats for diagnostics:
       struct SyncStats {
           files_scanned: usize,
           files_added: usize,
           files_modified: usize,
           files_deleted: usize,
           duration_ms: u64,
       }
   }
   ```

3. **Node/Edge Operations**
   ```rust
   fn upsert_decision_node(
       &self,
       decision: &Decision,
       content_hash: &str,
   ) -> Result<u128>  // Returns node_id
   
   fn create_relationship_edges(
       &self,
       from_id: u128,
       metadata: &DecisionMetadata,
   ) -> Result<()>
   
   fn delete_decision_node(&self, node_id: u128) -> Result<()>
   ```

4. **Search & Retrieval**
   ```rust
   pub fn search(
       &self,
       embedding: Vec<f32>,
       limit: usize,
   ) -> Result<Vec<(Decision, f32)>>
   
   pub fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>>
   pub fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>>
   ```

**Implementation Details:**

a. **Transaction Pattern:**
```rust
let mut wtxn = self.engine.storage.graph_env.write_txn()?;

// 1. Create/update node
let node_id = generate_node_id();
let node = Node {
    id: node_id,
    label: "DECISION",
    version: 1,
    properties: serialize_decision_properties(decision),
};
self.engine.storage.nodes_db.put(&mut wtxn, &node_id, &node.to_bincode_bytes()?)?;

// 2. Add vector
self.engine.storage.vectors.add_vector(&mut wtxn, node_id, embedding)?;

// 3. Create edges for relationships
for rel in decision.metadata.relationships() {
    let edge_id = generate_edge_id();
    let edge = Edge {
        id: edge_id,
        label: rel.relation_type.as_edge_label(),
        version: 1,
        from_node: node_id,
        to_node: rel.target_id as u128,  // May need mapping from u32 -> u128
        properties: None,
    };
    // Insert edge...
}

wtxn.commit()?;
```

b. **Change Detection (3-stage filter):**
```
Stage 1: Stat check (fast)
  if file.mtime == manifest.entry.mtime && file.size == manifest.entry.size {
    skip to next file
  }

Stage 2: Content hash (slower)
  content_hash = sha256(file_content)
  if content_hash == manifest.entry.content_hash {
    update mtime+size in manifest, skip re-embedding
  }

Stage 3: Full re-index
  parse YAML, embed, upsert node+edges
```

c. **Embedding Model Versioning:**
- Store embedding_model in manifest
- If model changes, re-embed all vectors
- Prevents stale embeddings from old model versions

d. **Error Handling:**
- Partial failures: Continue indexing other files, report summary
- Transaction rollback on critical errors
- Log all changes for audit trail

---

### Module 4: Update `storage.rs` (200 lines)
Wire HelixDB backend into existing DecisionStorage trait.

**Changes:**

1. **New Struct:**
   ```rust
   pub struct HelixDecisionStorage {
       backend: HelixDecisionBackend,
       decision_cache: HashMap<u32, Decision>,
   }
   ```

2. **Implement DecisionStorage Trait:**
   ```rust
   impl DecisionStorage for HelixDecisionStorage {
       fn index(&mut self, decisions: Vec<Decision>) -> Result<()> {
           // Delegate to backend.sync()
       }
       
       fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Decision, f32)>> {
           // Delegate to backend.search()
       }
       
       fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>> {
           // Delegate to backend.get_chain()
       }
       
       fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>> {
           // Delegate to backend.get_related()
       }
   }
   ```

3. **Open Method (replaces PersistentDecisionStorage::open):**
   ```rust
   impl HelixDecisionStorage {
       pub fn open() -> Result<Self> {
           let repo_root = find_git_root()?;
           let backend = HelixDecisionBackend::new(&repo_root)?;
           Ok(Self {
               backend,
               decision_cache: HashMap::new(),
           })
       }
   }
   ```

4. **Deprecation:**
   - Keep `PersistentDecisionStorage` for testing/fallback
   - Mark as deprecated in docs
   - Add feature flag to choose backend (future flexibility)

---

## C. Files to Create/Modify

### Create (New Files)
| File | Lines | Purpose |
|------|-------|---------|
| `helix-decisions/src/manifest.rs` | 250 | Manifest tracking for incremental indexing |
| `helix-decisions/src/git_utils.rs` | 100 | Git-based file enumeration |
| `helix-decisions/src/helix_backend.rs` | 600 | HelixDB storage backend |

### Modify (Existing Files)
| File | Changes |
|------|---------|
| `helix-decisions/src/storage.rs` | Add `HelixDecisionStorage`, implement trait wrapper |
| `helix-decisions/src/lib.rs` | Export new modules: `manifest`, `git_utils`, `helix_backend` |
| `helix-decisions/Cargo.toml` | Add dependencies: `helix-db` (direct), keep `sha2` |
| `helix-decisions/src/main.rs` | Minor: Update error messages if storage changes |

---

## D. Testing Strategy

### Unit Tests (per module)

**manifest.rs:**
- Serialize/deserialize manifest
- Entry operations (get, upsert, remove)
- Manifest persistence in metadata

**git_utils.rs:**
- List files from git
- Fallback to directory walk
- Respect .gitignore

**helix_backend.rs:**
- Stat-based delta detection
- Content hash comparison
- Node creation with properties
- Edge creation for relationships
- Vector storage and retrieval
- Incremental re-indexing
- Change detection (3 stages)

**storage.rs:**
- HelixDecisionStorage trait implementation
- Search with filters
- Chain and related queries

### Integration Tests

**End-to-End Scenarios:**
1. Initial indexing: 10 decisions → search works
2. Modify 1 decision → delta detected, only that file re-indexed
3. Add 3 decisions → only new files indexed
4. Delete 1 decision → node and edges removed
5. Change embedding model → all vectors re-computed
6. Large repo (100+ decisions) → incremental time < 100ms

### Performance Benchmarks
- First sync: < 5 seconds (10 decisions)
- Delta sync: < 100ms (file count invariant)
- Search: < 50ms (k=10 results)
- Vector operations: < 10ms per decision

---

## E. Dependency Analysis

### New Direct Dependencies
```toml
[dependencies]
helix-db = { path = "../../helix-db/helix-db" }  # Already in workspace!
# Everything else (sha2, serde, etc.) already present
```

### Workspace Structure Verification
```bash
helix-tools/
├── helix-graph-ops/         # HelixDB graph helpers
├── helix-decisions/         # ← We're here
├── shared/
│   ├── helix-embeddings/    # Still needed (fastembed)
│   └── ...
└── workspace/
    └── helix-db/            # HelixDB crate (heed3 is LMDB wrapper)
```

**Note:** `helix-db` in workspace is at `./helix-db/helix-db` (nested structure).
Update Cargo.toml to reference correctly.

---

## F. Compatibility & Migration

### Backward Compatibility
- Keep `PersistentDecisionStorage` for fallback
- Index format migration: Use `indexer_version` field
- If manifest missing: Full re-index automatically

### Upgrade Path
1. **User runs `helix-decisions search` on existing repo**
   - `.helix/data/decisions/` exists (JSON backend)
   - New code detects HelixDB not initialized
   - Option A: Auto-migrate (read JSON, write to HelixDB)
   - Option B: Full re-index from `.decisions/` files

2. **Implementation:** 
   ```rust
   pub fn open() -> Result<Self> {
       if db_already_initialized() {
           HelixDecisionStorage::open()  // Use new backend
       } else if legacy_storage_exists() {
           migrate_from_legacy()           // Auto-migrate
       } else {
           initialize_fresh()              // Fresh start
       }
   }
   ```

---

## G. Configuration Updates

### Workspace Cargo.toml
```toml
[workspace]
members = [
    "hbd",
    "helix-decisions",
    "docs",
    "shared/helix-*",
]

[dependencies]
helix-db = { path = "../helix-db/helix-db" }  # ← Add this globally OR...
```

### helix-decisions Cargo.toml
```toml
[dependencies]
helix-db = { path = "../../helix-db/helix-db" }  # Relative from helix-decisions/
# OR if using workspace override:
helix-db = { workspace = true }
```

### Environment Variables
- `HELIX_DB_PATH`: Override default `.helixdb/` location (optional)
- `HELIX_DECISIONS_SKIP_HOOKS`: Already used for hook bypass
- `HELIX_EMBEDDING_MODEL`: Already used in helix-embeddings

---

## H. Execution Sequence

### Session 1 (2-3 hours): Foundation Modules
1. **Start:** Fresh branch `feat/helix-db-integration`
2. **Create `manifest.rs`:** Type definitions + load/save
   - Test serialization/deserialization
   - 6 tests passing
3. **Create `git_utils.rs`:** File enumeration
   - Test with actual git repos
   - 4 tests passing
4. **Commit:** "feat(helix-decisions): add manifest and git_utils modules"

### Session 2 (2-3 hours): Backend Implementation
1. **Create `helix_backend.rs`:** Core incremental indexing
   - Transaction pattern
   - Change detection (3 stages)
   - Node/edge creation
   - Search delegation
   - 12 tests passing
2. **Update `storage.rs`:** Wire trait
   - Implement `DecisionStorage` for wrapper
   - 5 tests passing
3. **Update `lib.rs` and `Cargo.toml`:**
   - Export modules
   - Add `helix-db` dependency
4. **Commit:** "feat(helix-decisions): add HelixDB backend with incremental indexing"

### Session 3 (1-2 hours): Integration & Polish
1. **Integration tests:**
   - End-to-end scenarios
   - Performance benchmarks
   - 8 tests passing
2. **Documentation:**
   - Update README.md (HelixDB architecture section)
   - Add ARCHITECTURE.md (graph schema, incremental indexing)
   - Code comments on complex methods
3. **Cleanup:**
   - Remove `JsonFileBackend` from dependencies? (or keep for fallback)
   - Test with real `.decisions/` directories
4. **Final commit:** "docs(helix-decisions): add HelixDB architecture docs"
5. **Verify:**
   - `cargo test --all` passes
   - `cargo clippy` clean
   - `cargo fmt` applied

---

## I. Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| LMDB is single-writer | Assume sequential `sync()` calls; no concurrent writers |
| Complex transaction logic | Write incremental tests for each tx pattern |
| Node ID collisions | Use UUID v4 for node_id, very low collision risk |
| Embedding model changes | Track model version; re-embed if version mismatch |
| Manifest corruption | Load with fallback to full re-index |
| HelixDB API changes | Pin HelixDB version in Cargo.toml; document API usage |

---

## J. Success Criteria

✅ All 3 new modules created and tested  
✅ `DecisionStorage` trait fully implemented via HelixDB  
✅ `sync()` completes in < 100ms (delta-only)  
✅ Search, chain, related queries work with HelixDB  
✅ 30+ tests passing, 0 clippy warnings  
✅ Documentation complete (architecture, migration guide)  
✅ Integration tests cover all scenarios  
✅ All existing features still work (backward compatible)

---

## K. Future Enhancements (Post-Phase 3)

- [ ] Query language support (HelixQL) for complex searches
- [ ] Distributed indexing (across branches via git sync)
- [ ] Decision snapshots (time-travel queries)
- [ ] Audit log (who changed what, when)
- [ ] Full-text search (BM25) via HelixDB
- [ ] Web UI for decision exploration
- [ ] Integration with GitHub/GitLab PRs
- [ ] Agent-friendly query API

---

## Summary

This plan provides a **clear, incremental path** from MVP (JSON) to production-grade (HelixDB) storage while maintaining backward compatibility and keeping performance penalties minimal. The 3-stage change detection pattern ensures we only re-index what changed, achieving sub-100ms delta times on subsequent runs.

**Key Innovation:** Using HelixDB metadata to store the manifest avoids separate file management and keeps the database as a single source of truth for sync state.
