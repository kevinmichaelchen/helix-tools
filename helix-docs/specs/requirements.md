# Requirements

This document defines user stories and acceptance criteria for `helix-docs` using [EARS notation](https://www.iaria.org/conferences2015/filesICCGI15/EARS.pdf) (Easy Approach to Requirements Syntax).

## EARS Notation Reference

| Pattern | Template |
|---------|----------|
| Ubiquitous | THE SYSTEM SHALL `<action>` |
| Event-driven | WHEN `<trigger>` THE SYSTEM SHALL `<action>` |
| State-driven | WHILE `<state>` THE SYSTEM SHALL `<action>` |
| Optional | WHERE `<feature>` THE SYSTEM SHALL `<action>` |
| Complex | IF `<condition>` THEN THE SYSTEM SHALL `<action>` |

---

## 1. Source Management

### US-001: Add GitHub Repository Source

**As a** developer  
**I want to** add a GitHub repository as a documentation source  
**So that** I can fetch and search its documentation locally

| ID | Acceptance Criterion |
|----|---------------------|
| AC-001.1 | WHEN a user runs `helix-docs add https://github.com/owner/repo` THE SYSTEM SHALL create a Source record with type `github` |
| AC-001.2 | WHEN `--docs <path>` is provided THE SYSTEM SHALL only fetch files under that path (e.g., `--docs docs`) |
| AC-001.3 | WHEN `--ref <branch>` is provided THE SYSTEM SHALL fetch from that branch instead of the default branch |
| AC-001.4 | WHEN `--version <label>` is provided THE SYSTEM SHALL tag documents with that version label for filtering |
| AC-001.5 | IF no GitHub token is found THE SYSTEM SHALL use unauthenticated API access (rate-limited) |
| AC-001.6 | IF a GitHub token is found in config THE SYSTEM SHALL use authenticated API access |
| AC-001.7 | THE SYSTEM SHALL auto-detect GitHub tokens from `~/.config/helix/config.toml`, `GITHUB_TOKEN` env var, or `gh auth token` |
| AC-001.8 | WHEN `--json` flag is provided THE SYSTEM SHALL output the created source as JSON |

---

### US-002: Add Website Source (Modeled, Not Implemented)

**As a** developer  
**I want to** add a website as a documentation source  
**So that** I can fetch and search web-based documentation locally

| ID | Acceptance Criterion |
|----|---------------------|
| AC-002.1 | WHEN a user runs `helix-docs add https://docs.example.com` THE SYSTEM SHALL create a Source record with type `website` |
| AC-002.2 | WHEN `--depth <n>` is provided THE SYSTEM SHALL limit crawl depth to n levels from the root URL |
| AC-002.3 | WHEN `--pages <n>` is provided THE SYSTEM SHALL limit the number of pages crawled |
| AC-002.4 | WHEN `--allow <paths>` is provided THE SYSTEM SHALL only crawl URLs matching those path prefixes |
| AC-002.5 | WHEN `--deny <paths>` is provided THE SYSTEM SHALL exclude URLs matching those path prefixes |
| AC-002.6 | THE SYSTEM SHALL auto-discover pages via `llms.txt`, `sitemap.xml`, and `robots.txt` when available |

> **Implementation Note:** Website crawling is modeled in the type system but not implemented in MVP. GitHub sources only for Phase 1-2.

---

### US-003: List Sources

**As a** developer  
**I want to** view all configured documentation sources  
**So that** I can manage what documentation is available locally

| ID | Acceptance Criterion |
|----|---------------------|
| AC-003.1 | WHEN a user runs `helix-docs source list` THE SYSTEM SHALL display all sources with their URL, type, version labels, and last sync time |
| AC-003.2 | THE SYSTEM SHALL indicate sync status (synced, pending, error) for each source |
| AC-003.3 | WHEN `--json` flag is provided THE SYSTEM SHALL output the source list as JSON array |

---

### US-004: Remove Source

**As a** developer  
**I want to** remove a documentation source  
**So that** I can declutter my local documentation cache

| ID | Acceptance Criterion |
|----|---------------------|
| AC-004.1 | WHEN a user runs `helix-docs source remove <id>` THE SYSTEM SHALL delete the source and all its documents from storage |
| AC-004.2 | THE SYSTEM SHALL prompt for confirmation unless `--force` is provided |
| AC-004.3 | WHEN `--json` flag is provided THE SYSTEM SHALL output the removed source as JSON |

---

## 2. Ingestion

### US-005: Ingest All Sources

**As a** developer  
**I want to** fetch and process all configured sources  
**So that** my local documentation cache is up-to-date

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005.1 | WHEN a user runs `helix-docs ingest` THE SYSTEM SHALL fetch new/changed documents from all sources |
| AC-005.2 | THE SYSTEM SHALL use ETag caching for GitHub API calls to avoid redundant downloads |
| AC-005.3 | THE SYSTEM SHALL chunk documents using appropriate strategies (markdown sections, code blocks) |
| AC-005.4 | WHEN `--concurrency <n>` is provided THE SYSTEM SHALL limit parallel fetches to n (default: 5) |
| AC-005.5 | WHEN `--force` is provided THE SYSTEM SHALL re-fetch all documents regardless of cache state |
| AC-005.6 | THE SYSTEM SHALL display progress with document count and estimated time remaining |
| AC-005.7 | WHEN `--json` flag is provided THE SYSTEM SHALL output ingestion results as JSON |

---

### US-006: Ingest with Embeddings

**As a** developer  
**I want to** generate vector embeddings during ingestion  
**So that** I can perform semantic search on documentation

| ID | Acceptance Criterion |
|----|---------------------|
| AC-006.1 | WHEN a user runs `helix-docs ingest --embed` THE SYSTEM SHALL generate embeddings for all chunks |
| AC-006.2 | THE SYSTEM SHALL use fastembed with BGE-small-en-v1.5 model for local embedding generation |
| AC-006.3 | THE SYSTEM SHALL skip embedding generation for chunks that already have embeddings unless `--force` |
| AC-006.4 | IF the embedding model is not downloaded THE SYSTEM SHALL download it automatically on first use |
| AC-006.5 | THE SYSTEM SHALL batch embedding generation for efficiency (default batch size: 32) |

---

### US-007: Seed Sources

**As a** developer  
**I want to** quickly add popular documentation sources  
**So that** I can bootstrap my local cache with commonly used libraries

| ID | Acceptance Criterion |
|----|---------------------|
| AC-007.1 | WHEN a user runs `helix-docs seed` THE SYSTEM SHALL add sources from the built-in seed list |
| AC-007.2 | THE SYSTEM SHALL not duplicate sources that already exist |
| AC-007.3 | WHEN `--ingest` is provided THE SYSTEM SHALL automatically ingest after seeding |
| AC-007.4 | THE SYSTEM SHALL support custom seed files via `--file <path>` |

---

## 3. Library Discovery

### US-008: Search Libraries

**As a** developer  
**I want to** find libraries by name  
**So that** I can discover what documentation is available

| ID | Acceptance Criterion |
|----|---------------------|
| AC-008.1 | WHEN a user runs `helix-docs library "react"` THE SYSTEM SHALL display matching libraries with their versions |
| AC-008.2 | THE SYSTEM SHALL fuzzy-match library names (e.g., "next" matches "vercel/next.js") |
| AC-008.3 | THE SYSTEM SHALL display document count and last sync time for each library |
| AC-008.4 | WHEN `--json` flag is provided THE SYSTEM SHALL output results as JSON |

---

### US-009: Detect Project Dependencies

**As a** developer  
**I want to** detect which libraries my project uses  
**So that** I can scope searches to relevant documentation

| ID | Acceptance Criterion |
|----|---------------------|
| AC-009.1 | WHEN a user runs `helix-docs detect` in a project directory THE SYSTEM SHALL parse dependency manifests |
| AC-009.2 | THE SYSTEM SHALL support: package.json, Cargo.toml, pyproject.toml, go.mod, requirements.txt, Gemfile |
| AC-009.3 | THE SYSTEM SHALL match detected dependencies to available libraries and display version mappings |
| AC-009.4 | THE SYSTEM SHALL suggest adding missing sources for detected dependencies |
| AC-009.5 | WHEN `--json` flag is provided THE SYSTEM SHALL output detected dependencies as JSON |

---

## 4. Search & Retrieval

### US-010: Hybrid Search

**As a** developer  
**I want to** search documentation using both keywords and meaning  
**So that** I can find relevant content even with different terminology

| ID | Acceptance Criterion |
|----|---------------------|
| AC-010.1 | WHEN a user runs `helix-docs search --library react "state management"` THE SYSTEM SHALL return relevant document chunks |
| AC-010.2 | THE SYSTEM SHALL combine BM25 (keyword) and vector (semantic) scores using Reciprocal Rank Fusion |
| AC-010.3 | WHEN `--mode word` is provided THE SYSTEM SHALL use BM25 only |
| AC-010.4 | WHEN `--mode vector` is provided THE SYSTEM SHALL use vector similarity only |
| AC-010.5 | WHEN `--mode hybrid` is provided THE SYSTEM SHALL use RRF fusion (default) |
| AC-010.6 | WHEN `--version <label>` is provided THE SYSTEM SHALL filter results to that version |
| AC-010.7 | THE SYSTEM SHALL display results with: title, path, relevance score, and content preview |
| AC-010.8 | WHEN `--limit <n>` is provided THE SYSTEM SHALL return at most n results (default: 10) |
| AC-010.9 | WHEN `--json` flag is provided THE SYSTEM SHALL output results as JSON array |

---

### US-011: Get Document

**As a** developer  
**I want to** retrieve full document content  
**So that** I can read complete documentation sections

| ID | Acceptance Criterion |
|----|---------------------|
| AC-011.1 | WHEN a user runs `helix-docs get --library react docs/hooks.md` THE SYSTEM SHALL display the document content |
| AC-011.2 | WHEN `--doc <id>` is provided THE SYSTEM SHALL retrieve by document ID instead of path |
| AC-011.3 | WHEN `--slice <start>:<end>` is provided THE SYSTEM SHALL return only lines start through end |
| AC-011.4 | THE SYSTEM SHALL render Markdown with syntax highlighting in terminal (unless `--raw`) |
| AC-011.5 | WHEN `--json` flag is provided THE SYSTEM SHALL output document metadata and content as JSON |

---

## 5. Freshness & Maintenance

### US-012: Check Status

**As a** developer  
**I want to** see the health of my documentation cache  
**So that** I can ensure I have up-to-date information

| ID | Acceptance Criterion |
|----|---------------------|
| AC-012.1 | WHEN a user runs `helix-docs status` THE SYSTEM SHALL display: total sources, documents, chunks, and embeddings count |
| AC-012.2 | THE SYSTEM SHALL display cache size on disk |
| AC-012.3 | THE SYSTEM SHALL list sources with stale documentation (not synced in N days, configurable) |
| AC-012.4 | THE SYSTEM SHALL list sources with newer versions available upstream |
| AC-012.5 | WHEN `--json` flag is provided THE SYSTEM SHALL output status as JSON |

---

### US-013: Cleanup Stale Data

**As a** developer  
**I want to** remove outdated documentation  
**So that** my cache stays lean and relevant

| ID | Acceptance Criterion |
|----|---------------------|
| AC-013.1 | WHEN a user runs `helix-docs cleanup` THE SYSTEM SHALL remove documents from deleted sources |
| AC-013.2 | THE SYSTEM SHALL remove orphaned chunks and embeddings |
| AC-013.3 | WHEN `--older-than <days>` is provided THE SYSTEM SHALL remove documents not accessed in N days |
| AC-013.4 | THE SYSTEM SHALL display what will be removed and prompt for confirmation unless `--force` |

---

### US-014: Freshness Strategies

**As a** developer  
**I want to** configure how documentation freshness is managed  
**So that** I can balance between freshness and bandwidth/storage

| ID | Acceptance Criterion |
|----|---------------------|
| AC-014.1 | THE SYSTEM SHALL support ETag-based incremental sync for GitHub sources |
| AC-014.2 | THE SYSTEM SHALL support time-based staleness detection (configurable threshold) |
| AC-014.3 | THE SYSTEM SHALL support version-change detection (re-sync when upstream version changes) |
| AC-014.4 | WHEN `helix-docs ingest --strategy <name>` is provided THE SYSTEM SHALL use that freshness strategy |
| AC-014.5 | THE SYSTEM SHALL allow composing multiple strategies (e.g., ETag AND time-based) |

---

## 6. Agent Integration

### US-015: MCP Server

**As a** developer using AI coding assistants  
**I want to** expose helix-docs as an MCP server  
**So that** AI agents can search documentation directly

| ID | Acceptance Criterion |
|----|---------------------|
| AC-015.1 | WHEN a user runs `helix-docs mcp` THE SYSTEM SHALL start an MCP server on stdio |
| AC-015.2 | THE SYSTEM SHALL expose tools: `search`, `library`, `get` |
| AC-015.3 | THE SYSTEM SHALL expose resources: library list, document metadata |
| AC-015.4 | THE SYSTEM SHALL handle concurrent requests safely |

---

### US-016: JSON Output

**As a** developer building automation  
**I want to** get structured JSON output from all commands  
**So that** I can parse and process results programmatically

| ID | Acceptance Criterion |
|----|---------------------|
| AC-016.1 | THE SYSTEM SHALL support `--json` flag on all commands |
| AC-016.2 | WHEN `--json` is provided THE SYSTEM SHALL output valid JSON to stdout |
| AC-016.3 | WHEN `--json` is provided THE SYSTEM SHALL output errors as JSON objects with `error` field |
| AC-016.4 | THE SYSTEM SHALL use consistent JSON schemas across commands |

---

### US-017: Session Tracking

**As a** developer analyzing AI agent behavior  
**I want to** track which agent sessions accessed documentation  
**So that** I can understand agent research patterns

| ID | Acceptance Criterion |
|----|---------------------|
| AC-017.1 | WHEN `--agent <id>` is provided THE SYSTEM SHALL record the agent identifier with the operation |
| AC-017.2 | WHEN `--session <id>` is provided THE SYSTEM SHALL group operations under that session |
| AC-017.3 | THE SYSTEM SHALL log search queries and retrieved documents per session |
| AC-017.4 | WHEN a user runs `helix-docs sessions` THE SYSTEM SHALL list recent agent sessions with query summaries |

---

## 7. Configuration

### US-018: Initialize Project

**As a** developer  
**I want to** initialize helix-docs in my project  
**So that** I can use project-specific configuration

| ID | Acceptance Criterion |
|----|---------------------|
| AC-018.1 | WHEN a user runs `helix-docs init` THE SYSTEM SHALL create `.helix/helix-docs.toml` with default settings |
| AC-018.2 | IF `.helix/` directory exists THE SYSTEM SHALL add config alongside existing hbd config |
| AC-018.3 | THE SYSTEM SHALL not overwrite existing configuration unless `--force` |
| AC-018.4 | THE SYSTEM SHALL create `.helix/helix-docs.toml` with documented default values |

---

### US-019: Configuration Hierarchy

**As a** developer  
**I want to** have global and project-level configuration  
**So that** I can share common settings while customizing per-project

| ID | Acceptance Criterion |
|----|---------------------|
| AC-019.1 | THE SYSTEM SHALL read global config from `~/.config/helix/config.toml` |
| AC-019.2 | THE SYSTEM SHALL read project config from `.helix/helix-docs.toml` |
| AC-019.3 | THE SYSTEM SHALL merge configs with project settings overriding global settings |
| AC-019.4 | THE SYSTEM SHALL support environment variable overrides for sensitive values (tokens) |

---

## 8. Storage Abstraction

### US-020: Pluggable Storage Backend

**As a** developer extending helix-docs  
**I want to** swap storage implementations  
**So that** I can use different backends for different use cases

| ID | Acceptance Criterion |
|----|---------------------|
| AC-020.1 | THE SYSTEM SHALL define storage operations via traits (interfaces) |
| AC-020.2 | THE SYSTEM SHALL ship with HelixDB backend as the default implementation |
| AC-020.3 | THE SYSTEM SHALL allow selecting backend via configuration |
| AC-020.4 | THE SYSTEM SHALL decouple domain logic from storage implementation |

> **Design Note:** See design.md for trait definitions: `SourceRepository`, `DocumentRepository`, `ChunkRepository`, `SearchIndex`.

---

## Non-Functional Requirements

### NFR-001: Performance

| ID | Requirement |
|----|-------------|
| NFR-001.1 | Search queries SHALL return results in under 100ms for caches with < 100K chunks |
| NFR-001.2 | Embedding generation SHALL process at least 100 chunks per second on modern hardware |
| NFR-001.3 | GitHub API calls SHALL use conditional requests (ETag) to minimize bandwidth |

### NFR-002: Reliability

| ID | Requirement |
|----|-------------|
| NFR-002.1 | THE SYSTEM SHALL handle network failures gracefully with retries and backoff |
| NFR-002.2 | THE SYSTEM SHALL support resumable ingestion (continue from where it stopped) |
| NFR-002.3 | THE SYSTEM SHALL validate data integrity with content hashes |

### NFR-003: Security

| ID | Requirement |
|----|-------------|
| NFR-003.1 | THE SYSTEM SHALL never log or display authentication tokens |
| NFR-003.2 | THE SYSTEM SHALL store tokens only in config files with appropriate permissions |
| NFR-003.3 | THE SYSTEM SHALL validate URLs before fetching to prevent SSRF |

### NFR-004: Usability

| ID | Requirement |
|----|-------------|
| NFR-004.1 | THE SYSTEM SHALL provide helpful error messages with suggested fixes |
| NFR-004.2 | THE SYSTEM SHALL support tab completion for shells (bash, zsh, fish) |
| NFR-004.3 | THE SYSTEM SHALL provide progress indicators for long-running operations |
