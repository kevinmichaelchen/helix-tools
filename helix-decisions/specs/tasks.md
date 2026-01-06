# adr-search: Implementation Tasks

**Document:** tasks.md  
**Status:** Concept (2026-01-05)  
**Author:** Kevin Chen

## Phase 1: Core (MVP)

### Task 1.1: Project Setup
- [x] Create `adr-search/` directory structure
- [x] Write `Cargo.toml` with dependencies
- [x] Write `README.md`
- [x] Write `specs/requirements.md`
- [x] Write `specs/design.md`
- [x] Write `specs/tasks.md`
- [ ] Add to workspace `Cargo.toml`

### Task 1.2: Types Module
- [ ] Define `Status` enum
- [ ] Define `ADRMetadata` struct (serde)
- [ ] Define `ADR` struct
- [ ] Define `SearchResult` struct
- [ ] Define `SearchResponse` struct
- [ ] Unit tests for serialization

### Task 1.3: Loader Module
- [ ] Implement `load_adrs(dir)` function
- [ ] Parse YAML frontmatter with `gray_matter`
- [ ] Extract body text
- [ ] Compute content hash (SHA256)
- [ ] Handle malformed files gracefully
- [ ] Unit tests with fixture ADRs

### Task 1.4: Embeddings Module
- [ ] Implement `Embedder` struct
- [ ] Initialize fastembed with `AllMiniLML6V2`
- [ ] Implement `embed(text)` method
- [ ] Implement `embed_batch(texts)` method
- [ ] Integration test with sample text

### Task 1.5: Storage Module
- [ ] Define `ADRStorage` trait
- [ ] Implement `HelixDBStorage` struct
- [ ] Implement `open()` for embedded HelixDB
- [ ] Implement `index(adrs)` method
- [ ] Implement `search(embedding, limit)` method
- [ ] Implement `get_hashes()` method
- [ ] Integration tests

### Task 1.6: Searcher Module
- [ ] Implement `ADRSearcher` struct
- [ ] Implement `new()` constructor
- [ ] Implement `sync(dir)` method (no delta yet)
- [ ] Implement `search(query, limit, filters)` method
- [ ] Integration test end-to-end

### Task 1.7: CLI
- [ ] Define `Cli` struct with clap derive
- [ ] Parse arguments: query, directory, limit, json
- [ ] Call `ADRSearcher::sync()` then `search()`
- [ ] Output pretty format (table)
- [ ] Output JSON format
- [ ] Help text and examples
- [ ] Integration test CLI

### Task 1.8: Testing
- [ ] Create fixture ADRs in `tests/fixtures/.decisions/`
- [ ] Unit tests for each module
- [ ] Integration tests for full workflow
- [ ] Test edge cases (empty dir, malformed YAML)

---

## Phase 2: Polish

### Task 2.1: Delta Indexing
- [ ] Implement `delta.rs` module
- [ ] Implement `compute_delta()` function
- [ ] Store hashes in HelixDB metadata
- [ ] Update `ADRSearcher::sync()` to use delta
- [ ] Implement `remove()` in storage
- [ ] Performance test (should be < 100ms with no changes)

### Task 2.2: Metadata Filtering
- [ ] Add `--status` filter to CLI
- [ ] Add `--tags` filter to CLI
- [ ] Implement filtering in `search()` method
- [ ] Test filtering combinations

### Task 2.3: Output Formatting
- [ ] Improve pretty output with colors
- [ ] Add box drawing for table
- [ ] Detect TTY vs pipe for default format
- [ ] Add `--format` option (pretty/json)

### Task 2.4: Error Handling
- [ ] Use `thiserror` for typed errors
- [ ] Clear error messages with suggestions
- [ ] Proper exit codes (0, 1, 2)
- [ ] Test all error paths

### Task 2.5: Documentation
- [ ] Update README with final CLI
- [ ] Add examples for common use cases
- [ ] Document ADR format requirements
- [ ] Add troubleshooting section

---

## Phase 3: Advanced (Optional)

### Task 3.1: Performance Optimization
- [ ] Benchmark embedding time
- [ ] Benchmark search time
- [ ] Profile and optimize hot paths
- [ ] Consider batch embedding

### Task 3.2: Full-Text Fallback
- [ ] Add BM25 search on title/tags
- [ ] Combine with vector search (RRF)
- [ ] Useful when semantic search misses keywords

### Task 3.3: Multi-Directory Support
- [ ] Support multiple `--directory` args
- [ ] Merge results across directories
- [ ] Useful for monorepos

### Task 3.4: Agent Integration
- [ ] Test with Claude/agent workflows
- [ ] Document MCP integration (if needed)
- [ ] Add `--agent` output mode (if different from JSON)

---

## Dependencies

### Blocked by:
- HelixDB embedded mode availability
- HelixDB Rust client API

### Parallel with:
- Can develop loader, embeddings, CLI in parallel
- Storage module blocked by HelixDB

### Fallback:
- If HelixDB not ready, use simple JSON file storage
- Replace with HelixDB when available

---

## Milestones

| Milestone | Tasks | Target |
|-----------|-------|--------|
| **M1: Skeleton** | 1.1-1.2 | Day 1 |
| **M2: Load & Embed** | 1.3-1.4 | Day 2-3 |
| **M3: Storage** | 1.5 | Day 4-5 |
| **M4: Search** | 1.6-1.7 | Day 6-7 |
| **M5: MVP Complete** | 1.8 | Day 8 |
| **M6: Delta & Filters** | 2.1-2.2 | Day 9-10 |
| **M7: Polish** | 2.3-2.5 | Day 11-12 |
| **M8: v0.1.0 Release** | All Phase 2 | Week 2 |

---

## Notes

- Start with JSON file storage if HelixDB not ready
- Focus on end-to-end workflow first
- Optimize after correctness
- Test with real `.decisions/` directories
