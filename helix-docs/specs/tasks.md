# Tasks

This document defines the implementation roadmap for `helix-docs`, organized into phases with dependencies tracked.

## Task Status Legend

| Status | Meaning |
|--------|---------|
| `[ ]` | Not started |
| `[~]` | In progress |
| `[x]` | Completed |
| `[-]` | Blocked |
| `[!]` | Needs review |

---

## Phase 0: Project Foundation

**Goal:** Establish project structure, build system, and core abstractions.

### P0-001: Project Setup
- [ ] Create `helix-docs/` directory in workspace
- [ ] Create `Cargo.toml` with workspace dependencies
- [ ] Add `helix-docs` to workspace `Cargo.toml` members
- [ ] Set up module structure (`src/`, `specs/`)
- [ ] Create `.gitignore` entries for HelixDB data

**Acceptance:** `cargo build -p helix-docs` succeeds with empty binary.

### P0-002: Domain Types
- [ ] Define `SourceId`, `DocId`, `ChunkId` strongly-typed IDs (`src/domain/id.rs`)
- [ ] Define `Source`, `SourceType`, `SourceConfig` (`src/domain/source.rs`)
- [ ] Define `Document`, `DocumentMetadata` (`src/domain/document.rs`)
- [ ] Define `Chunk`, `ChunkPosition`, `ChunkMetadata` (`src/domain/chunk.rs`)
- [ ] Define `SearchQuery`, `SearchResult`, `SearchMode` (`src/domain/search.rs`)
- [ ] Define `Library`, `Version` (`src/domain/library.rs`)
- [ ] Add serde derives and Display implementations

**Depends on:** P0-001  
**Acceptance:** All domain types compile with serde serialization.

### P0-003: Port Traits (Interfaces)
- [ ] Define `SourceRepository` trait (`src/ports/repository.rs`)
- [ ] Define `DocumentRepository` trait
- [ ] Define `ChunkRepository` trait
- [ ] Define `EmbeddingRepository` trait
- [ ] Define `SearchIndex` trait (`src/ports/search.rs`)
- [ ] Define `FetchClient` trait (`src/ports/fetch.rs`)
- [ ] Define `EmbeddingGenerator` trait (`src/ports/embed.rs`)

**Depends on:** P0-002  
**Acceptance:** All traits defined with async-trait, no implementations yet.

### P0-004: Error Types
- [ ] Create `HelixDocsError` enum (`src/error.rs`)
- [ ] Implement `From` conversions for common errors
- [ ] Implement exit codes for each error variant
- [ ] Create `Result<T>` type alias

**Depends on:** P0-001  
**Acceptance:** Error types cover all documented error cases.

### P0-005: CLI Scaffold
- [ ] Set up clap with derive (`src/main.rs`)
- [ ] Define root `Cli` struct with `--json` flag
- [ ] Define `Commands` enum with all subcommands (stubs)
- [ ] Create empty command handlers in `src/cli/`
- [ ] Implement `--help` for all commands
- [ ] Implement `--version` flag

**Depends on:** P0-004  
**Acceptance:** `helix-docs --help` shows all commands, each command has `--help`.

---

## Phase 1: GitHub Ingestion (MVP)

**Goal:** Fetch and store GitHub documentation with BM25 search.

### P1-001: Configuration System
- [ ] Define `Config` struct (`src/config/mod.rs`)
- [ ] Implement global config loading (`~/.config/helix/config.toml`)
- [ ] Implement project config loading (`.helix/helix-docs.toml`)
- [ ] Implement config merging (project overrides global)
- [ ] Implement environment variable overrides (`GITHUB_TOKEN`)
- [ ] Auto-detect GitHub token from `gh auth token`

**Depends on:** P0-004  
**Acceptance:** Config loads from all sources with correct precedence.

### P1-002: GitHub Fetch Client
- [ ] Create `GitHubClient` struct (`src/adapters/github/mod.rs`)
- [ ] Implement `FetchClient` trait for `GitHubClient`
- [ ] Implement tree API listing (`/repos/{owner}/{repo}/git/trees/{sha}`)
- [ ] Implement blob fetching (`/repos/{owner}/{repo}/git/blobs/{sha}`)
- [ ] Implement ETag caching for conditional requests
- [ ] Implement rate limit handling with backoff
- [ ] Support `--docs` path filtering
- [ ] Support `--ref` branch/tag selection

**Depends on:** P0-003, P1-001  
**Acceptance:** Can fetch Markdown files from public GitHub repos.

### P1-003: HelixDB Adapter - Storage
- [ ] Create `HelixAdapter` struct (`src/adapters/helix/mod.rs`)
- [ ] Implement schema initialization (`schema.hx`)
- [ ] Implement `SourceRepository` for `HelixAdapter`
- [ ] Implement `DocumentRepository` for `HelixAdapter`
- [ ] Implement `ChunkRepository` for `HelixAdapter`
- [ ] Test CRUD operations for all entity types

**Depends on:** P0-003  
**Acceptance:** Can persist and retrieve all domain entities.

### P1-004: Document Chunking
- [ ] Create `Chunker` trait (`src/chunk/mod.rs`)
- [ ] Implement `MarkdownChunker` (`src/chunk/markdown.rs`)
  - [ ] Split by headings (##, ###, etc.)
  - [ ] Keep code blocks intact
  - [ ] Track line numbers
  - [ ] Respect max chunk size with overlap
- [ ] Create `ChunkMetadata` extraction (section titles, code language)

**Depends on:** P0-002  
**Acceptance:** Markdown documents chunked with preserved structure.

### P1-005: Source Service
- [ ] Create `SourceService` (`src/services/source.rs`)
- [ ] Implement `add_source()` - validates URL, creates Source
- [ ] Implement `list_sources()` - returns all sources with stats
- [ ] Implement `remove_source()` - deletes source and cascades
- [ ] Implement `get_source()` - retrieves by ID or URL

**Depends on:** P1-003  
**Acceptance:** Source CRUD operations work via service layer.

### P1-006: Ingestion Service
- [ ] Create `IngestionService` (`src/services/ingestion.rs`)
- [ ] Implement `ingest_source()` - fetches and chunks one source
- [ ] Implement `ingest_all()` - ingests all sources with concurrency
- [ ] Implement freshness checking (skip unchanged documents)
- [ ] Implement progress reporting callback
- [ ] Implement resumable ingestion (track progress per source)

**Depends on:** P1-002, P1-003, P1-004, P1-005  
**Acceptance:** Can ingest GitHub repos and store documents + chunks.

### P1-007: HelixDB Adapter - BM25 Search
- [ ] Implement BM25 index configuration in schema
- [ ] Implement `SearchIndex::search_bm25()` for `HelixAdapter`
- [ ] Implement `SearchIndex::index_document()` for `HelixAdapter`
- [ ] Implement `SearchIndex::remove_document()` for `HelixAdapter`
- [ ] Test keyword search across documents

**Depends on:** P1-003  
**Acceptance:** BM25 search returns relevant results for keywords.

### P1-008: Search Service (BM25 Only)
- [ ] Create `SearchService` (`src/services/search.rs`)
- [ ] Implement `search()` with mode routing
- [ ] Implement `search_bm25()` - keyword search
- [ ] Implement library/version filtering
- [ ] Implement result formatting

**Depends on:** P1-007  
**Acceptance:** `helix-docs search --mode word` returns results.

### P1-009: CLI Commands - Source Management
- [ ] Implement `helix-docs add <url>` (`src/cli/add.rs`)
- [ ] Implement `helix-docs source list` (`src/cli/source.rs`)
- [ ] Implement `helix-docs source remove <id>`
- [ ] Implement `--json` output for all source commands

**Depends on:** P1-005, P0-005  
**Acceptance:** Can add, list, remove sources via CLI.

### P1-010: CLI Commands - Ingestion
- [ ] Implement `helix-docs ingest` (`src/cli/ingest.rs`)
- [ ] Implement `--force` flag (re-fetch all)
- [ ] Implement `--concurrency` flag
- [ ] Implement progress display
- [ ] Implement `--json` output

**Depends on:** P1-006, P0-005  
**Acceptance:** `helix-docs ingest` fetches and indexes documents.

### P1-011: CLI Commands - Search
- [ ] Implement `helix-docs search --library <lib> <query>` (`src/cli/search.rs`)
- [ ] Implement `--mode word` (BM25 only for Phase 1)
- [ ] Implement `--version` filtering
- [ ] Implement `--limit` flag
- [ ] Implement `--json` output
- [ ] Implement human-readable result formatting

**Depends on:** P1-008, P0-005  
**Acceptance:** Can search indexed documentation from CLI.

### P1-012: CLI Commands - Get Document
- [ ] Implement `helix-docs get --library <lib> <path>` (`src/cli/get.rs`)
- [ ] Implement `--doc <id>` alternative
- [ ] Implement `--slice <start>:<end>` line range
- [ ] Implement `--raw` flag (no formatting)
- [ ] Implement `--json` output

**Depends on:** P1-003, P0-005  
**Acceptance:** Can retrieve and display document content.

---

## Phase 2: Semantic Search

**Goal:** Add vector embeddings and hybrid search.

### P2-001: Fastembed Adapter
- [ ] Create `FastembedGenerator` (`src/adapters/fastembed/mod.rs`)
- [ ] Implement `EmbeddingGenerator` trait
- [ ] Implement model download on first use
- [ ] Implement batch embedding generation
- [ ] Configure for BGE-small-en-v1.5 (384 dimensions)

**Depends on:** P0-003  
**Acceptance:** Can generate embeddings locally without network.

### P2-002: HelixDB Adapter - Vector Storage
- [ ] Add HNSW index configuration to schema
- [ ] Implement `EmbeddingRepository` for `HelixAdapter`
- [ ] Implement `SearchIndex::search_vector()` for cosine similarity
- [ ] Test vector search accuracy

**Depends on:** P1-003  
**Acceptance:** Can store and query vector embeddings.

### P2-003: Embedding Integration
- [ ] Extend `IngestionService` with `--embed` flag support
- [ ] Implement batch embedding during ingestion
- [ ] Implement skip logic for existing embeddings
- [ ] Implement embedding stats in status output

**Depends on:** P2-001, P2-002, P1-006  
**Acceptance:** `helix-docs ingest --embed` generates embeddings.

### P2-004: Hybrid Search
- [ ] Implement RRF (Reciprocal Rank Fusion) in `SearchService`
- [ ] Implement parallel BM25 + vector search
- [ ] Implement `--mode hybrid` (default)
- [ ] Implement `--mode vector` for similarity-only search
- [ ] Tune RRF k constant (default: 60)

**Depends on:** P2-002, P1-008  
**Acceptance:** Hybrid search outperforms BM25 alone on semantic queries.

### P2-005: CLI Commands - Embed Flag
- [ ] Add `--embed` flag to `helix-docs ingest`
- [ ] Implement embedding progress reporting
- [ ] Update `helix-docs status` with embedding counts

**Depends on:** P2-003  
**Acceptance:** `helix-docs ingest --embed` works from CLI.

### P2-006: CLI Commands - Search Modes
- [ ] Enable `--mode hybrid` (default)
- [ ] Enable `--mode vector`
- [ ] Warn if vector mode requested but no embeddings exist

**Depends on:** P2-004  
**Acceptance:** All search modes work from CLI.

---

## Phase 3: Library Discovery & Detection

**Goal:** Find libraries and detect project dependencies.

### P3-001: Library Service
- [ ] Create `LibraryService` (`src/services/library.rs`)
- [ ] Implement `find_library()` - fuzzy search by name
- [ ] Implement `list_versions()` - versions for a library
- [ ] Implement library stats (doc count, last sync)

**Depends on:** P1-003  
**Acceptance:** Can search for libraries by name.

### P3-002: Dependency Detection
- [ ] Create manifest parsers (`src/services/detect.rs`)
  - [ ] `package.json` (npm)
  - [ ] `Cargo.toml` (Rust)
  - [ ] `pyproject.toml` / `requirements.txt` (Python)
  - [ ] `go.mod` (Go)
  - [ ] `Gemfile` (Ruby)
- [ ] Implement `detect_dependencies()` - scan current directory
- [ ] Implement library matching (detected deps → available libraries)

**Depends on:** P3-001  
**Acceptance:** Detects dependencies from common manifests.

### P3-003: CLI Commands - Library
- [ ] Implement `helix-docs library <name>` (`src/cli/library.rs`)
- [ ] Display matching libraries with versions
- [ ] Display document counts per library
- [ ] Implement `--json` output

**Depends on:** P3-001  
**Acceptance:** Can find libraries from CLI.

### P3-004: CLI Commands - Detect
- [ ] Implement `helix-docs detect` (`src/cli/detect.rs`)
- [ ] Display detected dependencies
- [ ] Display matched libraries (available locally)
- [ ] Suggest missing sources to add
- [ ] Implement `--json` output

**Depends on:** P3-002  
**Acceptance:** Can detect and match project dependencies.

---

## Phase 4: Freshness & Maintenance

**Goal:** Keep documentation fresh and cache clean.

### P4-001: Freshness Strategies
- [ ] Create `FreshnessService` (`src/services/freshness.rs`)
- [ ] Implement ETag-based freshness (GitHub)
- [ ] Implement time-based staleness detection
- [ ] Implement version-change detection
- [ ] Implement strategy composition

**Depends on:** P1-002, P1-003  
**Acceptance:** Can determine if source needs re-sync.

### P4-002: Status Command
- [ ] Implement `helix-docs status` (`src/cli/status.rs`)
- [ ] Display total sources, documents, chunks
- [ ] Display embedding coverage
- [ ] Display cache size on disk
- [ ] List stale sources
- [ ] List sources with upstream updates
- [ ] Implement `--json` output

**Depends on:** P4-001, P1-003  
**Acceptance:** Status shows comprehensive cache health.

### P4-003: Cleanup Command
- [ ] Implement `helix-docs cleanup` (`src/cli/cleanup.rs`)
- [ ] Remove orphaned documents (deleted sources)
- [ ] Remove orphaned chunks and embeddings
- [ ] Implement `--older-than <days>` flag
- [ ] Implement dry-run mode (default)
- [ ] Implement `--force` flag
- [ ] Display cleanup summary

**Depends on:** P1-003  
**Acceptance:** Can clean up stale data from CLI.

### P4-004: Incremental Sync
- [ ] Extend `IngestionService` to use freshness checks
- [ ] Skip unchanged documents (ETag match)
- [ ] Update only changed documents
- [ ] Track sync progress per source
- [ ] Support resumable sync (continue after interruption)

**Depends on:** P4-001, P1-006  
**Acceptance:** Incremental sync significantly faster than full sync.

---

## Phase 5: Agent Integration

**Goal:** Expose helix-docs to AI agents via MCP and session tracking.

### P5-001: Session Tracking
- [ ] Define `Session`, `SessionEvent` domain types
- [ ] Extend repository traits for session logging
- [ ] Log search queries per session
- [ ] Log document retrievals per session
- [ ] Implement session summary generation

**Depends on:** P1-003  
**Acceptance:** Sessions logged to storage.

### P5-002: CLI Commands - Sessions
- [ ] Implement `helix-docs sessions` (`src/cli/sessions.rs`)
- [ ] List recent sessions with agent IDs
- [ ] Display query summaries per session
- [ ] Implement `--json` output

**Depends on:** P5-001  
**Acceptance:** Can view agent session history.

### P5-003: MCP Server
- [ ] Create MCP server (`src/mcp.rs`)
- [ ] Implement `search` tool (hybrid search)
- [ ] Implement `library` tool (find library)
- [ ] Implement `get` tool (retrieve document)
- [ ] Implement stdio transport
- [ ] Handle concurrent requests safely

**Depends on:** P1-008, P3-001, P1-012  
**Acceptance:** MCP server works with Claude Code.

### P5-004: Agent Flags
- [ ] Add `--agent <id>` flag to mutating commands
- [ ] Add `--session <id>` flag to mutating commands
- [ ] Record agent/session in session log

**Depends on:** P5-001  
**Acceptance:** Agent operations tracked with metadata.

---

## Phase 6: Seed & Quality of Life

**Goal:** Easy bootstrap and polished UX.

### P6-001: Seed Sources
- [ ] Create `data/libraries.yml` seed file
- [ ] Populate with popular libraries (React, Next.js, Tokio, etc.)
- [ ] Implement `helix-docs seed` (`src/cli/seed.rs`)
- [ ] Skip already-existing sources
- [ ] Implement `--ingest` flag (auto-ingest after seeding)
- [ ] Support custom seed files via `--file`

**Depends on:** P1-005  
**Acceptance:** `helix-docs seed` bootstraps with common libraries.

### P6-002: Init Command
- [ ] Implement `helix-docs init` (`src/cli/init.rs`)
- [ ] Create `.helix/helix-docs.toml` with defaults
- [ ] Document all config options with comments
- [ ] Skip if already initialized (unless `--force`)

**Depends on:** P1-001  
**Acceptance:** Projects can be initialized with defaults.

### P6-003: Shell Completions
- [ ] Generate completions for bash
- [ ] Generate completions for zsh
- [ ] Generate completions for fish
- [ ] Add `helix-docs completions <shell>` command

**Depends on:** P0-005  
**Acceptance:** Tab completion works in major shells.

### P6-004: Progress & Formatting
- [ ] Implement progress bars for ingestion
- [ ] Implement syntax highlighting for Markdown output
- [ ] Implement colored terminal output
- [ ] Respect `NO_COLOR` environment variable

**Depends on:** P1-010, P1-012  
**Acceptance:** CLI has polished, informative output.

---

## Phase 7: Website Crawling (Future)

**Goal:** Support website documentation sources.

> **Note:** This phase is modeled but not scheduled for MVP. Tasks are documented for future reference.

### P7-001: Web Crawler
- [ ] Create `WebCrawler` struct (`src/adapters/web/crawler.rs`)
- [ ] Implement URL normalization and deduplication
- [ ] Implement link extraction from HTML
- [ ] Implement depth limiting
- [ ] Implement page limiting
- [ ] Implement allow/deny path filtering

### P7-002: Discovery Mechanisms
- [ ] Implement `llms.txt` parsing
- [ ] Implement `sitemap.xml` parsing
- [ ] Implement `robots.txt` parsing
- [ ] Auto-select discovery method

### P7-003: Headless Browser
- [ ] Integrate headless Chrome for SPA sites
- [ ] Detect CSR/SPA sites automatically
- [ ] Implement JavaScript rendering
- [ ] Handle infinite scroll / lazy loading

### P7-004: Web Fetch Client
- [ ] Create `WebClient` implementing `FetchClient`
- [ ] Integrate crawler and discovery
- [ ] Implement crawl state persistence
- [ ] Implement resumable crawling

---

## Dependency Graph

```
Phase 0 (Foundation)
├── P0-001 Project Setup
├── P0-002 Domain Types ────────────────────────────┐
├── P0-003 Port Traits ─────────────────────────────┤
├── P0-004 Error Types                              │
└── P0-005 CLI Scaffold                             │
                                                    │
Phase 1 (GitHub + BM25)                             │
├── P1-001 Config ──────────────────────────────────┤
├── P1-002 GitHub Client ───────────────────────────┤
├── P1-003 HelixDB Storage ─────────────────────────┤
├── P1-004 Chunking ────────────────────────────────┤
├── P1-005 Source Service ──────────────────────────┤
├── P1-006 Ingestion Service ───────────────────────┤
├── P1-007 HelixDB BM25 ────────────────────────────┤
├── P1-008 Search Service ──────────────────────────┤
├── P1-009 CLI Source ──────────────────────────────┤
├── P1-010 CLI Ingest ──────────────────────────────┤
├── P1-011 CLI Search ──────────────────────────────┤
└── P1-012 CLI Get ─────────────────────────────────┘
                │
Phase 2 (Vectors)
├── P2-001 Fastembed ───────────────────────────────┐
├── P2-002 HelixDB Vectors ─────────────────────────┤
├── P2-003 Embed Integration ───────────────────────┤
├── P2-004 Hybrid Search ───────────────────────────┤
├── P2-005 CLI Embed ───────────────────────────────┤
└── P2-006 CLI Search Modes ────────────────────────┘
                │
Phase 3 (Discovery)
├── P3-001 Library Service ─────────────────────────┐
├── P3-002 Dependency Detection ────────────────────┤
├── P3-003 CLI Library ─────────────────────────────┤
└── P3-004 CLI Detect ──────────────────────────────┘
                │
Phase 4 (Freshness)
├── P4-001 Freshness Strategies ────────────────────┐
├── P4-002 CLI Status ──────────────────────────────┤
├── P4-003 CLI Cleanup ─────────────────────────────┤
└── P4-004 Incremental Sync ────────────────────────┘
                │
Phase 5 (Agents)
├── P5-001 Session Tracking ────────────────────────┐
├── P5-002 CLI Sessions ────────────────────────────┤
├── P5-003 MCP Server ──────────────────────────────┤
└── P5-004 Agent Flags ─────────────────────────────┘
                │
Phase 6 (Polish)
├── P6-001 Seed Sources
├── P6-002 Init Command
├── P6-003 Shell Completions
└── P6-004 Progress & Formatting
```

---

## Estimated Timeline

| Phase | Scope | Estimated Effort |
|-------|-------|------------------|
| Phase 0 | Foundation | 2-3 days |
| Phase 1 | GitHub + BM25 MVP | 5-7 days |
| Phase 2 | Semantic Search | 3-4 days |
| Phase 3 | Library Discovery | 2-3 days |
| Phase 4 | Freshness | 2-3 days |
| Phase 5 | Agent Integration | 3-4 days |
| Phase 6 | Polish | 2-3 days |
| Phase 7 | Web Crawling | TBD (future) |

**Total MVP (Phases 0-2):** ~10-14 days  
**Full Feature Set (Phases 0-6):** ~20-27 days
