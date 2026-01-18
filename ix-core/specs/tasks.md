# ix-core Tasks

## Phase 0: Skeleton

- [ ] Define the minimal `ix-core` façade API used by `ix-cli` and `ix-mcp`
- [ ] Define storage traits (file/graph/vector) with a small “happy path”
- [ ] Decide the canonical `.ixchel/` (or `.helix/`) on-disk layout and IDs

## Phase 1: Git-first sync

- [ ] Parse Markdown + YAML frontmatter into typed entities
- [ ] Implement file ↔ graph reconciliation with provenance (content hashes)
- [ ] Add incremental sync (changed files only)

## Phase 2: Search and context

- [ ] Chunk + embed pipeline interfaces
- [ ] Hybrid retrieval + graph expansion
- [ ] Context assembly formats (markdown/json/xml)
