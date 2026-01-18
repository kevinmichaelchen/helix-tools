# ix-core Tasks

## Phase 0: Repo + Layout

- [x] Decide canonical `.ixchel/` on-disk layout
- [x] Provide repo discovery (git-root + `.ixchel` marker)
- [x] Define a minimal façade API used by `ix-cli`

## Phase 1: Git-First Entities

- [x] Parse Markdown + YAML frontmatter
- [x] Create/list/show/link/unlink file-first entities
- [x] Validate relationship targets (broken-link detection)

## Phase 2: Index + Search

- [x] Define an index backend trait (`IndexBackend`)
- [x] Wire a working `sync` + `search` path via HelixDB backend
- [ ] Add incremental sync (changed files only) + deletions/renames

## Phase 3: Graph + Context

- [x] Provide a basic graph view from file relationships
- [x] Provide a basic context builder (1-hop expansion)
- [ ] Add configurable depth + edge-type prioritization
