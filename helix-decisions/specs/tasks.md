# helix-decisions: Implementation Tasks

**Document:** tasks.md  
**Status:** In Progress (2026-01-06)  
**Author:** Kevin Chen

> **Implementation Status**
>
> | Phase | Status | Description |
> |-------|--------|-------------|
> | Phase 1-2 (Core) | ✅ Complete | Types, loader, embeddings, HelixDB storage, CLI, hooks |
> | Phase 3.1 (Foundation) | ✅ Complete | Manifest, git_utils modules |
> | Phase 3.2 (Backend) | ✅ Complete | HelixDB backend, storage rewrite |
> | Phase 3.3 (Integration) | ✅ Complete | Integration tests, SyncStats |
> | Phase 3.4 (Daemon) | ✅ Complete | IPC client, `--sync` flag, daemon integration |
> | Phase 4 (Advanced) | 📋 Future | BM25 hybrid search, query language |

---

## Phase 1-2: Core (COMPLETE)

### Task 1.1: Project Setup ✅
- [x] Create `helix-decisions/` directory structure
- [x] Write `Cargo.toml` with dependencies
- [x] Write `README.md`
- [x] Write `specs/requirements.md`
- [x] Write `specs/design.md`
- [x] Write `specs/tasks.md`
- [x] Add to workspace `Cargo.toml`

### Task 1.2: Types Module ✅
- [x] Define `Status` enum
- [x] Define `DecisionMetadata` struct (serde)
- [x] Define `Decision` struct
- [x] Define `SearchResult` struct
- [x] Define `SearchResponse` struct
- [x] Define `RelationType` enum and `Relationship` struct
- [x] Unit tests for serialization

### Task 1.3: Loader Module ✅
- [x] Implement `load_decisions(dir)` function
- [x] Parse YAML frontmatter with `gray_matter`
- [x] Extract body text
- [x] Compute content hash (SHA256)
- [x] Handle malformed files gracefully
- [x] Unit tests with fixture decisions

### Task 1.4: Embeddings Module ✅
- [x] Implement `Embedder` struct (wraps helix-embeddings)
- [x] Initialize fastembed with BGE-small-en-v1.5
- [x] Implement `embed(text)` method
- [x] Implement `embed_batch(texts)` method
- [x] Integration test with sample text

### Task 1.5: Storage Module ✅
- [x] Define `DecisionStorage` trait
- [x] Implement `HelixDecisionStorage` (HelixDB backend)
- [x] Implement `open()` for project-local storage
- [x] Implement `index(decisions)` method
- [x] Implement `search(embedding, limit)` method
- [x] Implement `get_hashes()` method
- [x] Implement `get_chain()` and `get_related()` methods
- [x] Integration tests

### Task 1.6: Searcher Module ✅
- [x] Implement `DecisionSearcher` struct
- [x] Implement `new()` constructor
- [x] Implement `sync(dir)` method with delta detection
- [x] Implement `search(query, limit, filters)` method
- [x] Integration test end-to-end

### Task 1.7: CLI ✅
- [x] Define `Cli` struct with clap derive
- [x] Parse arguments: query, directory, limit, json
- [x] Commands: search, chain, related
- [x] Output pretty format
- [x] Output JSON format
- [x] Auto-discovery of `.decisions/` directory
- [x] Help text and examples
- [x] Add `check` command for frontmatter + uuid validation

### Task 1.8: Git Hooks ✅
- [x] Implement `hooks.rs` module
- [x] `init-hooks` command to install pre-commit hook
- [x] `remove-hooks` command to uninstall
- [x] Block modifications to accepted decisions
- [x] Allow amendments via `amends: [id]` pattern
- [x] Bypass options: `--no-verify`, env var

### Task 1.9: Configuration ✅
- [x] Implement `config.rs` module
- [x] Load from `~/.helix/config/helix-decisions.toml`
- [x] Load from `.helix/helix-decisions.toml` (repo override)
- [x] `strict` mode toggle

---

## Phase 3: Indexer + Daemon (PLANNED)

> **Reference Documents:**
> - `docs/phase3/PHASE_3_PLAN.md` - Detailed architecture
> - `docs/phase3/PHASE_3_CORRECTIONS.md` - API alignment fixes
> - `docs/phase3/CORRECTIONS_QUICK_REFERENCE.txt` - Quick lookup

### Task 3.1: Foundation Modules (Session 1) ✅

#### Task 3.1.1: manifest.rs ✅
- [x] Define `ManifestEntry` struct
  - file_path, mtime, size, content_hash, decision_id, uuid, vector_id, embedding_model, indexer_version
- [x] Define `IndexManifest` struct (HashMap of entries)
- [x] Define `MANIFEST_KEY` constant: `"manifest:helix-decisions:v1"`
- [x] Implement `from_bytes()` / `to_bytes()` for JSON serialization
- [x] Implement CRUD: `get()`, `contains()`, `upsert()`, `remove()`
- [x] Implement helpers: `find_by_content_hash()`, `find_by_decision_id()`
- [x] Unit tests (11 tests):
  - Entry creation, stats_changed, content_changed, needs_reembed
  - CRUD operations, serialization round-trip
  - Vector ID field handling
  - Find by content hash / decision ID

> **Note:** HelixDB metadata integration deferred to Task 3.2 (backend).
> Manifest uses JSON bytes for storage-agnostic serialization.

#### Task 3.1.2: git_utils.rs ✅
- [x] Implement `list_decision_files(repo_root, decisions_dir)` function
- [x] Run `git ls-files '.decisions/**/*.md'` internally
- [x] Implement directory walk fallback if git unavailable
- [x] Return sorted `Vec<PathBuf>`
- [x] Unit tests (4 tests):
  - Directory walk finds .md files
  - Results are sorted
  - Error handling for missing directory
  - Fallback behavior

### Task 3.2: Backend Implementation (Session 2)

#### Task 3.2.1: helix_backend.rs (~600 lines)
- [ ] Define `HelixDecisionBackend` struct
  - engine: HelixGraphEngine
  - manifest: IndexManifest
  - embedding_model: String
- [ ] Implement `new(repo_root)` constructor
  - Respect `HELIX_DB_PATH` env var
  - Pass path through `HelixGraphEngineOpts`
  - Create secondary indices on `decision_id`, `vector_id`
  - Load manifest from metadata
- [ ] Implement 3-stage incremental `sync()`:
  - Stage 1: Stat check (mtime + size)
  - Stage 2: Content hash check
  - Stage 3: Full re-index (parse, embed, upsert)
  - Handle deletions (tombstone node + vector)
  - Attempt rename match before deletion (content hash + uuid only)
  - If uuid is missing, treat rename as delete + add
  - Reuse vector_id when content hash + embedding model unchanged
  - Update vectors in place when content changes
  - Batch writes into LMDB transactions (nodes/vectors first, edges second)
  - Remove and recreate outgoing edges on change (same batch)
  - Save manifest after sync
  - Return `SyncStats` (files_scanned, added, modified, deleted, duration_ms)
- [ ] Implement `upsert_decision_node()`:
  - Allocate arena with `Bump::new()`
  - Insert vector first, get vector_id
  - Build `ImmutablePropertiesMap` in arena
  - Create `Node<'arena>` with arena-allocated label
  - Use `HelixGraphStorage::node_key()` helper
  - Store node with `nodes_db.put()`
- [ ] Implement `create_relationship_edges()`:
  - Write to 3 databases per edge:
    - `edges_db.put()` for edge data
    - `out_edges_db.put()` for outgoing adjacency
    - `in_edges_db.put()` for incoming adjacency
  - Use `hash_label()` for adjacency keys
  - Use `pack_edge_data()` helper
- [ ] Implement `delete_decision_node()`:
  - Extract vector_id from node properties BEFORE deletion
  - Call `storage.drop_node()` (drops edges + indices)
  - Call `storage.drop_vector()` (tombstone)
- [ ] Implement `search()`:
  - Convert f32 → f64 for HNSW
  - Search vectors
  - Map vector_id → node via secondary index
  - Return `Vec<(Decision, f32)>`
- [ ] Implement `get_chain()`:
  - Traverse `out_edges_db` for SUPERSEDES edges
  - Follow chain to leaf
  - Return `Vec<ChainNode>`
- [ ] Implement `get_related()`:
  - Query all edge types (both directions)
  - Return `Vec<RelatedDecision>`
- [ ] Unit tests (20+ tests):
  - Arena allocation
  - Properties round-trip
  - Key helper usage
  - Vector f32→f64 conversion
  - Vector ID storage in properties
  - Delete tombstones vector
  - Edge creation (all 3 DBs)
  - Label hashing
  - Adjacency keys correct
  - Traversal finds edges
  - All 3 stages of change detection
  - Secondary index lookups

#### Task 3.2.2: Update storage.rs
- [ ] Add `HelixDecisionStorage` wrapper struct
- [ ] Implement `DecisionStorage` trait for `HelixDecisionStorage`
- [ ] Delegate methods to `HelixDecisionBackend`
- [ ] Unit tests (5 tests):
  - Trait implementation
  - Search with filters
  - Chain queries
  - Related queries
  - Secondary index usage

#### Task 3.2.3: Update lib.rs and Cargo.toml
- [ ] Export new modules: `manifest`, `git_utils`, `helix_backend`
- [ ] Add `helix-db` dependency
- [ ] Add `bumpalo` dependency (for arena allocation)
- [ ] Verify workspace version alignment

### Task 3.3: Integration & Polish (Session 3)

#### Task 3.3.1: Integration Tests ✅
- [x] Scenario 1: Initial indexing (10 decisions)
- [x] Scenario 2: Modify 1 decision (delta detected)
- [x] Scenario 3: Add 3 decisions (only new files indexed)
- [x] Scenario 4: Delete 1 decision (node + vector removed)
- [ ] Scenario 5: Embedding model change (re-embed all) - deferred, requires model change detection
- [x] Scenario 6: Large repo (100+ decisions, <100ms delta)
- [x] Scenario 7: Chain traversal across supersedes
- [x] Scenario 8: Related query with all edge types

> **Note:** Integration tests require embedding model (~30MB). Run with:
> `cargo test -p helix-decisions --test integration -- --ignored`

#### Task 3.3.2: Performance Benchmarks
- [ ] First sync (10 decisions): < 5 seconds
- [ ] Delta sync (no changes): < 50ms
- [ ] Delta sync (1 file changed): < 100ms
- [ ] Search (k=10): < 50ms
- [ ] Graph traversal: < 50ms

#### Task 3.3.3: Documentation
- [ ] Update README.md with HelixDB architecture section
- [ ] Document graph schema
- [ ] Add code comments on complex methods

#### Task 3.3.4: Cleanup ✅
- [x] Run `cargo test --all` - 37 unit tests pass, 11 integration tests (ignored)
- [x] Run `cargo clippy` (0 warnings)
- [x] Run `cargo fmt`
- [x] Verify backward compatibility

#### Task 3.4: Indexer Daemon and Consistency

##### Task 3.4.1: Daemon Process ✅
- [x] ~~Add `helix-decisions daemon` subcommand~~ → Implemented as shared `helixd` binary
- [x] Implement a global per-user daemon with `{repo_root, tool}` namespacing
- [x] Define a stable socket naming scheme for helix-tools
- [x] Use Unix socket path `~/.helix/run/helixd.sock` (named pipe equivalent on Windows)
- [x] Auto-start daemon on first CLI invocation if socket is missing
- [x] Maintain a single-writer lock for LMDB (via SyncQueue namespacing)
- [x] Process a queue of sync requests (repo path + decision dir)
- [x] Exit cleanly after idle timeout (configurable, default 5 min)

> **Note:** Daemon implemented in `shared/helix-daemon` crate, not helix-decisions specific.
> See `shared/helix-daemon/specs/tasks.md` for daemon implementation details.

##### Task 3.4.2: IPC + Queue ✅
- [x] Use `shared/helix-daemon` IPC client
- [x] CLI sends `enqueue_sync` on each invocation
- [x] CLI uses `wait_sync` for `--sync`

##### Task 3.4.3: Strong Consistency Flag ✅
- [x] Add `--sync` to block until the queued sync completes
- [x] Implement `wait_sync` with timeout and clear error reporting
- [x] If daemon is unavailable, run a direct sync under the writer lock
- [x] Emit a warning when serving potentially stale results

##### Task 3.4.4: Sync Worker (PLANNED)
> The daemon currently stubs sync execution. This task implements actual sync logic.

- [ ] Implement sync callback registration in daemon
- [ ] Wire helix-decisions sync logic (load decisions, embed, store) to daemon worker
- [ ] Handle incremental updates (manifest-based delta detection)

---

## Phase 4: Advanced (Future)

### Task 4.1: BM25 Hybrid Search
- [ ] Enable HelixDB BM25 index on title/body
- [ ] Combine with vector search (RRF)
- [ ] Useful when semantic search misses keywords

### Task 4.2: Query Language
- [ ] Support HelixQL for complex queries
- [ ] Find all decisions in a status chain
- [ ] Find decisions with specific tag combinations

### Task 4.3: File Watcher Enhancements (Optional)
- [ ] File watching for continuous re-indexing (fs events)
- [ ] Smarter debounce/coalesce for bursty file changes
- [ ] Push notifications for stale index warnings

---

## Dependencies

### Phase 3 Requires:
- HelixDB embedded mode (available in helix-db crate)
- Understanding of LMDB transaction model
- Arena allocation patterns (bumpalo)

### Reference Files:
- `helix-db/src/helix_engine/tests/storage_tests.rs` - Canonical examples
- `helix-db/src/helix_engine/storage_core/mod.rs` - Key helpers
- `helix-db/src/utils/items.rs` - Node/Edge structs
- `helix-db/src/utils/properties.rs` - ImmutablePropertiesMap

---

## Milestones

| Milestone | Tasks | Status | Target |
|-----------|-------|--------|--------|
| **M1: Core Types** | 1.1-1.2 | ✅ Complete | - |
| **M2: Load & Embed** | 1.3-1.4 | ✅ Complete | - |
| **M3: Storage** | 1.5 | ✅ Complete | - |
| **M4: Search** | 1.6-1.7 | ✅ Complete | - |
| **M5: Hooks** | 1.8-1.9 | ✅ Complete | - |
| **M6: Daemon Integration** | 3.4 | ✅ Complete | - |
| **M7: Phase 3 Foundation** | 3.1 | ✅ Complete | - |
| **M8: Phase 3 Backend** | 3.2 | 🚧 Planned | Session 2 (2-3 hrs) |
| **M9: Phase 3 Integration** | 3.3 | 🚧 Planned | Session 3 (1-2 hrs) |
| **M10: v1.0.0 Release** | All Phase 3 | 🚧 Planned | Total: 6-8 hrs |

---

## Notes

- Core complete with HelixDB storage - fully functional for small-medium repos
- Phase 3 adds HelixDB for performance at scale (100+ decisions)
- HelixDB patterns MUST follow corrections in `docs/phase3/PHASE_3_CORRECTIONS.md`
- Key insight: Edges require 3 DB writes (edges_db, out_edges_db, in_edges_db)
- Key insight: Vectors stored separately, linked via vector_id property
- Test with real `.decisions/` directories before marking complete
