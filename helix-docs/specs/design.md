# Design

This document defines the technical architecture, data model, trait abstractions, and implementation strategies for `helix-docs`.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Shared Dependencies](#shared-dependencies)
3. [Design Principles](#design-principles)
4. [Module Structure](#module-structure)
5. [Core Traits (Abstractions)](#core-traits-abstractions)
6. [Data Model](#data-model)
7. [Storage Layer](#storage-layer)
8. [Ingestion Pipeline](#ingestion-pipeline)
9. [Search Strategy](#search-strategy)
10. [Configuration](#configuration)
11. [Error Handling](#error-handling)
12. [File Formats](#file-formats)

---

## Architecture Overview

### Target Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CLI Layer (helix-docs)                             │
│                                                                              │
│   helix-docs add, ingest, search, library, get, detect, status, mcp, ...    │
│   - Rust CLI with clap                                                       │
│   - All commands support --json for AI agents                                │
│   - Commands delegate to Application Services                                │
└────────────────────────────────────────┬────────────────────────────────────┘
                                         │
┌────────────────────────────────────────┴────────────────────────────────────┐
│                         Application Services                                 │
│                                                                              │
│   SourceService      IngestionService      SearchService      LibraryService │
│   - Add/remove       - Fetch documents     - Hybrid search    - Find libs    │
│   - List sources     - Chunk content       - BM25 + vector    - Detect deps  │
│   - Validate URLs    - Generate embeds     - RRF fusion       - Versions     │
│                                                                              │
│   Orchestrates domain logic, delegates to repositories via traits            │
└────────────────────────────────────────┬────────────────────────────────────┘
                                         │
┌────────────────────────────────────────┴────────────────────────────────────┐
│                            Domain Layer                                      │
│                                                                              │
│   Source          Document         Chunk           Embedding                 │
│   - id            - id             - id            - chunk_id                │
│   - url           - source_id      - doc_id        - vector                  │
│   - type          - path           - text          - model                   │
│   - config        - title          - metadata      - created_at              │
│                   - version        - position                                │
│                   - content_hash                                             │
│                                                                              │
│   Pure domain types with no I/O dependencies                                 │
└────────────────────────────────────────┬────────────────────────────────────┘
                                         │
┌────────────────────────────────────────┴────────────────────────────────────┐
│                         Repository Traits (Ports)                            │
│                                                                              │
│   trait SourceRepository        trait DocumentRepository                     │
│   trait ChunkRepository         trait EmbeddingRepository                    │
│   trait SearchIndex             trait FetchClient                            │
│   trait EmbeddingGenerator                                                   │
│                                                                              │
│   Abstract interfaces - domain logic programs against these                  │
└────────────────────────────────────────┬────────────────────────────────────┘
                                         │
         ┌───────────────────────────────┼───────────────────────────────┐
         │                               │                               │
         v                               v                               v
┌─────────────────────┐   ┌─────────────────────────┐   ┌─────────────────────┐
│   HelixDB Adapter   │   │    GitHub Fetch Client  │   │   Fastembed Adapter │
│                     │   │                         │   │                     │
│  - Source storage   │   │  - Tree API             │   │  - BGE-small-en     │
│  - Document store   │   │  - Blob fetching        │   │  - Local inference  │
│  - Vector search    │   │  - ETag caching         │   │  - Batch processing │
│  - BM25 index       │   │  - Rate limiting        │   │                     │
│  - Graph traversal  │   │                         │   │                     │
└─────────────────────┘   └─────────────────────────┘   └─────────────────────┘
         │                               │                               │
         └───────────────────────────────┼───────────────────────────────┘
                                         │
                              ┌──────────┴──────────┐
                              │     Infrastructure  │
                              │                     │
                              │  - File system      │
                              │  - Network (reqwest)│
                              │  - Config files     │
                              └─────────────────────┘
```

### Daemon Integration (Planned)

helix-docs uses the global helixd daemon for background ingestion and sync. The CLI
enqueues work via IPC and optionally waits with `--sync`. Protocol details live in
`shared/helix-daemon/specs/design.md`.

### Layered Architecture (Hexagonal/Ports & Adapters)

```
                    ┌─────────────────────────────────────┐
                    │            CLI (Driving)            │
                    │         helix-docs commands         │
                    └─────────────────┬───────────────────┘
                                      │ calls
                                      v
┌─────────────────────────────────────────────────────────────────────────────┐
│                              APPLICATION CORE                                │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                        Application Services                            │  │
│  │  SourceService, IngestionService, SearchService, LibraryService       │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                      │                                       │
│                                      │ uses                                  │
│                                      v                                       │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                           Domain Model                                 │  │
│  │  Source, Document, Chunk, Embedding, SearchResult, Library            │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                      │                                       │
│                                      │ depends on (traits)                   │
│                                      v                                       │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         Port Traits (Interfaces)                       │  │
│  │  SourceRepository, DocumentRepository, ChunkRepository,               │  │
│  │  EmbeddingRepository, SearchIndex, FetchClient, EmbeddingGenerator    │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                    implements        │         implements
         ┌────────────────────────────┼────────────────────────────┐
         v                            v                            v
┌─────────────────┐        ┌─────────────────┐        ┌─────────────────────┐
│  HelixDB Adapter│        │  GitHub Client  │        │  Fastembed Adapter  │
│    (Driven)     │        │    (Driven)     │        │      (Driven)       │
└─────────────────┘        └─────────────────┘        └─────────────────────┘
```

---

## Shared Dependencies

`helix-docs` leverages shared libraries from the helix-tools workspace to avoid code duplication and ensure consistency across tools.

### helix-id

**Purpose:** Hash-based ID generation for all entity types.

**Usage in helix-docs:**
- `SourceId` (`src-xxxxxx`) - Identifies documentation sources (deterministic from URL)
- `DocId` (`doc-xxxxxx`) - Identifies documents (deterministic from source + path)
- `ChunkId` (`chk-xxxxxx`) - Identifies document chunks (deterministic from doc + index)

**ID Strategy:**
```rust
use helix_id::define_id;

define_id!(SourceId, "src");
define_id!(DocId, "doc");
define_id!(ChunkId, "chk");

// Sources use from_key for global deduplication
let source_id = SourceId::from_key("github.com/facebook/react");
// Same URL always produces same ID → natural deduplication

// Documents use from_parts (source + path)
let doc_id = DocId::from_parts(&[source_id.as_str(), "docs/hooks.md"]);

// Chunks use from_parts (doc + index)
let chunk_id = ChunkId::from_parts(&[doc_id.as_str(), "0"]);
```

**Why deterministic IDs?**
- Global cache means same source added from different projects should deduplicate
- Re-ingesting same content produces same IDs (idempotent)
- Enables "upsert" semantics without coordination

See: [shared/helix-id/specs/design.md](../../shared/helix-id/specs/design.md)

### helix-config

**Purpose:** Hierarchical configuration loading with global/project/env precedence.

**Usage in helix-docs:**
- Global config from `~/.helix/config/config.toml`
- Tool config from `~/.helix/config/helix-docs.toml`
- Project config from `.helix/helix-docs.toml`
- GitHub token auto-detection
- Path helpers for data directories

**Integration:**
```rust
use helix_config::{load_config, detect_github_token, helix_data_dir};

let config: HelixDocsConfig = load_config("helix-docs")?;
let token = detect_github_token();
let docs_cache = helix_data_dir().join("docs");  // ~/.helix/data/docs
```

See: [shared/helix-config/specs/design.md](../../shared/helix-config/specs/design.md)

### Future Shared Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| `helix-embed` | Local embeddings via fastembed | Planned |
| `helix-search` | Hybrid search (BM25 + vector + RRF) | Planned |
| `helix-chunk` | Document chunking strategies | Planned |

When implemented, these will replace the local implementations in `helix-docs`.

---

## Design Principles

### 1. Single Responsibility

Each module has one reason to change:
- `domain/` - Business rules only
- `services/` - Use case orchestration only
- `adapters/` - External system integration only
- `cli/` - User interface only

### 2. Dependency Inversion

High-level modules (services) depend on abstractions (traits), not concrete implementations.

```rust
// Service depends on trait, not concrete type
pub struct IngestionService<R: DocumentRepository, F: FetchClient> {
    repo: R,
    client: F,
}
```

### 3. Interface Segregation

Many small, focused traits instead of one large interface:

```rust
// Good: focused traits
trait SourceRepository { /* source operations */ }
trait DocumentRepository { /* document operations */ }
trait SearchIndex { /* search operations */ }

// Bad: one giant trait
trait Database { /* everything */ }
```

### 4. Open/Closed

New adapters can be added without modifying existing code:

```rust
// Add new storage backend by implementing traits
struct SqliteAdapter;
impl SourceRepository for SqliteAdapter { /* ... */ }
impl DocumentRepository for SqliteAdapter { /* ... */ }
```

---

## Module Structure

```
helix-docs/
├── Cargo.toml
├── specs/
│   ├── requirements.md
│   ├── design.md          # This file
│   └── tasks.md
└── src/
    ├── main.rs            # Entry point, CLI setup
    ├── lib.rs             # Public API re-exports
    │
    ├── cli/               # CLI layer (clap commands)
    │   ├── mod.rs
    │   ├── add.rs         # helix-docs add
    │   ├── source.rs      # helix-docs source list/remove
    │   ├── ingest.rs      # helix-docs ingest
    │   ├── search.rs      # helix-docs search
    │   ├── library.rs     # helix-docs library
    │   ├── get.rs         # helix-docs get
    │   ├── detect.rs      # helix-docs detect
    │   ├── status.rs      # helix-docs status
    │   └── mcp.rs         # helix-docs mcp
    │
    ├── domain/            # Pure domain types (no I/O)
    │   ├── mod.rs
    │   ├── source.rs      # Source, SourceType, SourceConfig
    │   ├── document.rs    # Document, DocumentMetadata
    │   ├── chunk.rs       # Chunk, ChunkMetadata, ChunkPosition
    │   ├── embedding.rs   # Embedding, EmbeddingModel
    │   ├── search.rs      # SearchQuery, SearchResult, SearchMode
    │   └── library.rs     # Library, Version, Dependency
    │   # Note: IDs (SourceId, DocId, ChunkId) come from helix-id crate
    │
    ├── services/          # Application services (use cases)
    │   ├── mod.rs
    │   ├── source.rs      # SourceService
    │   ├── ingestion.rs   # IngestionService
    │   ├── search.rs      # SearchService
    │   ├── library.rs     # LibraryService
    │   └── freshness.rs   # FreshnessService (staleness strategies)
    │
    ├── ports/             # Trait definitions (interfaces)
    │   ├── mod.rs
    │   ├── repository.rs  # SourceRepository, DocumentRepository, etc.
    │   ├── fetch.rs       # FetchClient trait
    │   ├── embed.rs       # EmbeddingGenerator trait
    │   └── search.rs      # SearchIndex trait
    │
    ├── adapters/          # Concrete implementations
    │   ├── mod.rs
    │   ├── helix/         # HelixDB implementation
    │   │   ├── mod.rs
    │   │   ├── source.rs
    │   │   ├── document.rs
    │   │   ├── chunk.rs
    │   │   └── search.rs
    │   ├── github/        # GitHub API client
    │   │   ├── mod.rs
    │   │   ├── client.rs
    │   │   ├── tree.rs
    │   │   └── blob.rs
    │   ├── web/           # Web crawler (modeled, not implemented)
    │   │   └── mod.rs
    │   └── fastembed/     # Local embeddings
    │       └── mod.rs
    │
    ├── chunk/             # Document chunking strategies
    │   ├── mod.rs
    │   ├── markdown.rs    # Markdown section chunking
    │   └── code.rs        # Code-aware chunking
    │
    # Note: Configuration loading uses helix-config crate
    # Tool-specific config types defined in main.rs or dedicated module
    │
    └── error.rs           # Error types
```

---

## Core Traits (Abstractions)

### Repository Traits

```rust
// src/ports/repository.rs

use crate::domain::{Source, SourceId, Document, DocId, Chunk, ChunkId};
use async_trait::async_trait;

/// Repository for managing documentation sources
#[async_trait]
pub trait SourceRepository: Send + Sync {
    /// Create a new source
    async fn create(&self, source: &Source) -> Result<SourceId>;
    
    /// Get source by ID
    async fn get(&self, id: &SourceId) -> Result<Option<Source>>;
    
    /// Get source by URL
    async fn get_by_url(&self, url: &str) -> Result<Option<Source>>;
    
    /// List all sources
    async fn list(&self) -> Result<Vec<Source>>;
    
    /// Update source metadata (e.g., last_synced_at)
    async fn update(&self, source: &Source) -> Result<()>;
    
    /// Delete source and cascade to documents
    async fn delete(&self, id: &SourceId) -> Result<()>;
}

/// Repository for managing documents
#[async_trait]
pub trait DocumentRepository: Send + Sync {
    /// Create or update a document
    async fn upsert(&self, doc: &Document) -> Result<DocId>;
    
    /// Get document by ID
    async fn get(&self, id: &DocId) -> Result<Option<Document>>;
    
    /// Get document by source and path
    async fn get_by_path(&self, source_id: &SourceId, path: &str) -> Result<Option<Document>>;
    
    /// List documents for a source
    async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Document>>;
    
    /// List documents by library name pattern
    async fn list_by_library(&self, pattern: &str) -> Result<Vec<Document>>;
    
    /// Delete document and cascade to chunks
    async fn delete(&self, id: &DocId) -> Result<()>;
    
    /// Delete all documents for a source
    async fn delete_by_source(&self, source_id: &SourceId) -> Result<()>;
    
    /// Get documents not accessed since timestamp
    async fn list_stale(&self, since: DateTime<Utc>) -> Result<Vec<Document>>;
}

/// Repository for managing chunks
#[async_trait]
pub trait ChunkRepository: Send + Sync {
    /// Create chunks for a document (replaces existing)
    async fn create_for_document(&self, doc_id: &DocId, chunks: &[Chunk]) -> Result<()>;
    
    /// Get chunks for a document
    async fn get_by_document(&self, doc_id: &DocId) -> Result<Vec<Chunk>>;
    
    /// Get chunk by ID
    async fn get(&self, id: &ChunkId) -> Result<Option<Chunk>>;
    
    /// Count chunks without embeddings
    async fn count_without_embeddings(&self) -> Result<usize>;
    
    /// Get chunks needing embeddings (paginated)
    async fn list_needing_embeddings(&self, limit: usize, offset: usize) -> Result<Vec<Chunk>>;
}

/// Repository for managing embeddings
#[async_trait]
pub trait EmbeddingRepository: Send + Sync {
    /// Store embeddings for chunks
    async fn store(&self, embeddings: &[(ChunkId, Vec<f32>)]) -> Result<()>;
    
    /// Check if chunk has embedding
    async fn has_embedding(&self, chunk_id: &ChunkId) -> Result<bool>;
    
    /// Delete embeddings for document's chunks
    async fn delete_by_document(&self, doc_id: &DocId) -> Result<()>;
}
```

### Search Trait

```rust
// src/ports/search.rs

use crate::domain::{SearchQuery, SearchResult, SearchMode};

/// Search index for hybrid search operations
#[async_trait]
pub trait SearchIndex: Send + Sync {
    /// Perform hybrid search (BM25 + vector with RRF fusion)
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    
    /// BM25 keyword search only
    async fn search_bm25(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    
    /// Vector similarity search only
    async fn search_vector(&self, embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    
    /// Index a document's chunks for BM25
    async fn index_document(&self, doc_id: &DocId, chunks: &[Chunk]) -> Result<()>;
    
    /// Remove document from index
    async fn remove_document(&self, doc_id: &DocId) -> Result<()>;
}
```

### Fetch Client Trait

```rust
// src/ports/fetch.rs

use crate::domain::{Source, FetchedDocument};

/// Client for fetching documents from external sources
#[async_trait]
pub trait FetchClient: Send + Sync {
    /// Check if this client can handle the source type
    fn supports(&self, source: &Source) -> bool;
    
    /// List all document paths in the source
    async fn list_paths(&self, source: &Source) -> Result<Vec<String>>;
    
    /// Fetch a document's content
    async fn fetch(&self, source: &Source, path: &str) -> Result<FetchedDocument>;
    
    /// Check if document has changed (returns new ETag if changed)
    async fn check_freshness(&self, source: &Source, path: &str, etag: Option<&str>) 
        -> Result<FreshnessCheck>;
}

pub enum FreshnessCheck {
    Fresh,
    Stale { new_etag: Option<String> },
    Unknown,
}
```

### Embedding Generator Trait

```rust
// src/ports/embed.rs

/// Generator for text embeddings
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    
    /// Generate embeddings for multiple texts (batched)
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    
    /// Get the embedding dimension
    fn dimension(&self) -> usize;
    
    /// Get the model name
    fn model_name(&self) -> &str;
}
```

---

## Data Model

### Domain Types

```rust
// src/domain/source.rs

/// A documentation source (GitHub repo or website)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: SourceId,
    pub url: String,
    pub source_type: SourceType,
    pub config: SourceConfig,
    pub created_at: DateTime<Utc>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    GitHub,
    Website,  // Modeled but not implemented
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Path prefix to fetch (e.g., "docs/")
    pub docs_path: Option<String>,
    
    /// Branch/tag/commit ref (GitHub)
    pub git_ref: Option<String>,
    
    /// Version label for filtering
    pub version: Option<String>,
    
    /// Last known ETag for conditional requests
    pub etag: Option<String>,
    
    // Website-specific (modeled, not implemented)
    pub crawl_depth: Option<u32>,
    pub max_pages: Option<u32>,
    pub allow_paths: Vec<String>,
    pub deny_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,
    Syncing,
    Synced,
    Error(String),
}
```

```rust
// src/domain/document.rs

/// A fetched document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocId,
    pub source_id: SourceId,
    pub path: String,
    pub title: Option<String>,
    pub content: String,
    pub content_hash: String,
    pub version: Option<String>,
    pub fetched_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub language: Option<String>,
    pub size_bytes: usize,
    pub line_count: usize,
}
```

```rust
// src/domain/chunk.rs

/// A chunk of a document (for search indexing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: ChunkId,
    pub doc_id: DocId,
    pub text: String,
    pub position: ChunkPosition,
    pub metadata: ChunkMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPosition {
    pub index: usize,           // Chunk index within document
    pub start_line: usize,
    pub end_line: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub section_title: Option<String>,  // For markdown headings
    pub language: Option<String>,       // For code blocks
    pub chunk_type: ChunkType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ChunkType {
    #[default]
    Prose,
    CodeBlock,
    Heading,
    List,
}
```

```rust
// src/domain/search.rs

/// Search query parameters
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub library: Option<String>,
    pub version: Option<String>,
    pub mode: SearchMode,
    pub limit: usize,
}

#[derive(Debug, Clone, Default)]
pub enum SearchMode {
    #[default]
    Hybrid,
    Word,    // BM25 only
    Vector,  // Vector similarity only
}

/// A single search result
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub chunk_id: ChunkId,
    pub doc_id: DocId,
    pub doc_path: String,
    pub doc_title: Option<String>,
    pub library: String,
    pub version: Option<String>,
    pub text: String,
    pub score: f32,
    pub position: ChunkPosition,
}
```

```rust
// IDs are provided by the helix-id crate
// See: shared/helix-id/specs/design.md

use helix_id::define_id;

define_id!(SourceId, "src");
define_id!(DocId, "doc");
define_id!(ChunkId, "chk");

// ID generation strategy:
//
// SourceId: Deterministic from normalized URL
//   SourceId::from_key("github.com/facebook/react")
//   → Same URL always produces same ID (global deduplication)
//
// DocId: Deterministic from source + path
//   DocId::from_parts(&[source_id.as_str(), "docs/hooks.md"])
//   → Same document always produces same ID (idempotent ingestion)
//
// ChunkId: Deterministic from doc + chunk index
//   ChunkId::from_parts(&[doc_id.as_str(), "0"])
//   → Stable chunk IDs for embedding updates
```

### HelixQL Schema (for HelixDB)

```helix
// .helix/schema.hx

// Nodes

N::Source {
    id: String @index,
    url: String @index,
    source_type: String,
    config_json: String,
    created_at: String,
    last_synced_at: String?,
    sync_status: String,
}

N::Document {
    id: String @index,
    path: String @index,
    title: String?,
    content: String,
    content_hash: String,
    version: String?,
    fetched_at: String,
    last_accessed_at: String,
    metadata_json: String,
}
N::Document { content @bm25 }

N::Chunk {
    id: String @index,
    text: String,
    position_json: String,
    metadata_json: String,
}
N::Chunk { text @bm25 }

N::Embedding {
    id: String @index,
    vector: [F32; 384] @hnsw(cosine),
    model: String,
    created_at: String,
}

// Edges

E::HasDocument {
    from: Source,
    to: Document,
}

E::HasChunk {
    from: Document,
    to: Chunk,
}

E::HasEmbedding {
    from: Chunk,
    to: Embedding,
}
```

---

## Storage Layer

### HelixDB Adapter Implementation

```rust
// src/adapters/helix/mod.rs

pub struct HelixAdapter {
    db: HelixDB,
}

impl HelixAdapter {
    pub async fn new(path: &Path) -> Result<Self> {
        let db = HelixDB::open(path).await?;
        Ok(Self { db })
    }
    
    pub async fn from_config(config: &Config) -> Result<Self> {
        let path = config.db_path();
        Self::new(&path).await
    }
}

// Implement all repository traits for HelixAdapter
impl SourceRepository for HelixAdapter { /* ... */ }
impl DocumentRepository for HelixAdapter { /* ... */ }
impl ChunkRepository for HelixAdapter { /* ... */ }
impl EmbeddingRepository for HelixAdapter { /* ... */ }
impl SearchIndex for HelixAdapter { /* ... */ }
```

### Storage Location

Documentation is cached **globally** since the same library (e.g., `facebook/react`) is identical regardless of which project uses it. This avoids redundant storage and API calls.

```
~/.helix/
├── config/
│   ├── config.toml          # Global config (shared with hbd)
│   └── helix-docs.toml      # Global helix-docs settings
│
└── data/
    └── docs/                # Global documentation cache
        ├── helix-docs.db/   # HelixDB data directory
        │   ├── data.mdb
        │   └── lock.mdb
        └── schema.hx        # HelixQL schema

.helix/                      # Project-local (optional)
└── helix-docs.toml          # Project-specific settings (e.g., preferred sources)
```

**Why global cache?**
- Same `facebook/react` docs used by multiple projects
- Fetch once, share everywhere
- Reduces disk usage and API rate limit consumption

**Project-local config (optional):**
Projects can have a `.helix/helix-docs.toml` to specify preferred sources or search filters, but the actual documentation data lives in the global cache.

---

## Ingestion Pipeline

### Sequence Diagram

```
┌──────┐     ┌─────────────────┐     ┌──────────────┐     ┌──────────────┐
│ CLI  │     │IngestionService │     │ FetchClient  │     │  Repository  │
└──┬───┘     └────────┬────────┘     └──────┬───────┘     └──────┬───────┘
   │                  │                     │                    │
   │  ingest()        │                     │                    │
   │─────────────────>│                     │                    │
   │                  │                     │                    │
   │                  │  list_sources()     │                    │
   │                  │────────────────────────────────────────->│
   │                  │<────────────────────────────────────────│
   │                  │                     │                    │
   │                  │  for each source:   │                    │
   │                  │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                    │
   │                  │                     │                    │
   │                  │  list_paths()       │                    │
   │                  │────────────────────>│                    │
   │                  │<────────────────────│                    │
   │                  │                     │                    │
   │                  │  for each path:     │                    │
   │                  │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                    │
   │                  │                     │                    │
   │                  │  check_freshness()  │                    │
   │                  │────────────────────>│                    │
   │                  │<────────────────────│                    │
   │                  │                     │                    │
   │                  │  [if stale]         │                    │
   │                  │  fetch()            │                    │
   │                  │────────────────────>│                    │
   │                  │<────────────────────│                    │
   │                  │                     │                    │
   │                  │  chunk_document()   │                    │
   │                  │  (internal)         │                    │
   │                  │                     │                    │
   │                  │  upsert_document()  │                    │
   │                  │────────────────────────────────────────->│
   │                  │<────────────────────────────────────────│
   │                  │                     │                    │
   │                  │  [if --embed]       │                    │
   │                  │  generate_embeddings()                   │
   │                  │  store_embeddings() │                    │
   │                  │────────────────────────────────────────->│
   │                  │                     │                    │
   │<─────────────────│                     │                    │
   │  IngestionResult │                     │                    │
```

### Chunking Strategy

```rust
// src/chunk/mod.rs

pub trait Chunker: Send + Sync {
    fn chunk(&self, content: &str, metadata: &DocumentMetadata) -> Vec<Chunk>;
}

pub struct MarkdownChunker {
    max_chunk_size: usize,
    overlap: usize,
}

impl Chunker for MarkdownChunker {
    fn chunk(&self, content: &str, metadata: &DocumentMetadata) -> Vec<Chunk> {
        // 1. Split by headings (##, ###, etc.)
        // 2. Keep code blocks intact
        // 3. Respect max_chunk_size with overlap
        // 4. Track line numbers for each chunk
    }
}
```

---

## Search Strategy

### Hybrid Search with RRF

```rust
// src/services/search.rs

impl<S: SearchIndex, E: EmbeddingGenerator> SearchService<S, E> {
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        match query.mode {
            SearchMode::Word => self.search_bm25(&query).await,
            SearchMode::Vector => self.search_vector(&query).await,
            SearchMode::Hybrid => self.search_hybrid(&query).await,
        }
    }
    
    async fn search_hybrid(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        // 1. Run BM25 and vector search in parallel
        let (bm25_results, vector_results) = tokio::join!(
            self.index.search_bm25(&query.query, query.limit * 2),
            self.search_vector_internal(&query.query, query.limit * 2),
        );
        
        // 2. Fuse results using Reciprocal Rank Fusion
        let fused = self.rrf_fusion(&bm25_results?, &vector_results?, K_CONSTANT);
        
        // 3. Apply filters (library, version)
        let filtered = self.apply_filters(fused, query);
        
        // 4. Return top N
        Ok(filtered.into_iter().take(query.limit).collect())
    }
    
    fn rrf_fusion(
        &self,
        bm25: &[SearchResult],
        vector: &[SearchResult],
        k: f32,
    ) -> Vec<SearchResult> {
        // RRF score = sum(1 / (k + rank)) for each result list
        let mut scores: HashMap<ChunkId, f32> = HashMap::new();
        
        for (rank, result) in bm25.iter().enumerate() {
            *scores.entry(result.chunk_id.clone()).or_default() += 
                1.0 / (k + rank as f32);
        }
        
        for (rank, result) in vector.iter().enumerate() {
            *scores.entry(result.chunk_id.clone()).or_default() += 
                1.0 / (k + rank as f32);
        }
        
        // Sort by RRF score and return
        // ...
    }
}
```

---

## Configuration

### Global Config (`~/.helix/config/config.toml`)

```toml
# Shared across all helix tools

[github]
# Token auto-detected from: GITHUB_TOKEN, gh auth token, or this field
token = "ghp_xxx"

[embedding]
model = "BAAI/bge-small-en-v1.5"
batch_size = 32
```

### Global helix-docs Config (`~/.helix/config/helix-docs.toml`)

```toml
# Global helix-docs settings (applies to all projects)

[storage]
# Cache location (default: ~/.helix/data/docs)
cache_dir = "~/.helix/data/docs"

[ingest]
# Default concurrency for fetching
concurrency = 5
# File extensions to include
extensions = ["md", "mdx", "txt", "rst"]

[search]
# Default search mode
default_mode = "hybrid"
# Default result limit
default_limit = 10
# RRF k constant
rrf_k = 60.0

[freshness]
# Days before considering a source stale
stale_days = 7
# Enable ETag-based caching
use_etag = true
```

### Project Config (`.helix/helix-docs.toml`) - Optional

```toml
# Project-specific overrides (optional)
# The documentation cache is global, but projects can customize behavior

[search]
# Limit searches to specific libraries for this project
preferred_libraries = [
  "facebook/react",
  "vercel/next.js",
]

# Override default search limit for this project
default_limit = 5
```

**Note:** Project config only customizes behavior (search filters, limits). The actual documentation data lives in the global cache at `~/.helix/data/docs/`.

### Configuration Loading

```rust
// src/config/mod.rs

#[derive(Debug, Clone)]
pub struct Config {
    pub github_token: Option<String>,
    pub db_path: PathBuf,
    pub ingest: IngestConfig,
    pub search: SearchConfig,
    pub freshness: FreshnessConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        // 1. Load global config from ~/.config/helix/config.toml
        let global = GlobalConfig::load()?;
        
        // 2. Load project config from .helix/helix-docs.toml (if exists)
        let project = ProjectConfig::load_if_exists()?;
        
        // 3. Merge with project overriding global
        let merged = global.merge(project);
        
        // 4. Apply environment variable overrides
        let config = merged.with_env_overrides();
        
        Ok(config)
    }
    
    fn with_env_overrides(mut self) -> Self {
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            self.github_token = Some(token);
        }
        self
    }
}
```

---

## Error Handling

### Error Types

```rust
// src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HelixDocsError {
    // Source errors
    #[error("Source not found: {0}")]
    SourceNotFound(String),
    
    #[error("Source already exists: {0}")]
    SourceExists(String),
    
    #[error("Invalid source URL: {0}")]
    InvalidSourceUrl(String),
    
    // Fetch errors
    #[error("GitHub API error: {0}")]
    GitHubApi(String),
    
    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),
    
    #[error("Document not found: {path} in {source}")]
    DocumentNotFound { source: String, path: String },
    
    // Search errors
    #[error("Library not found: {0}")]
    LibraryNotFound(String),
    
    #[error("No embeddings available, run `helix-docs ingest --embed` first")]
    NoEmbeddings,
    
    // Storage errors
    #[error("Database error: {0}")]
    Database(#[from] helix_db::Error),
    
    // Config errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    // I/O errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl HelixDocsError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::SourceNotFound(_) | Self::DocumentNotFound { .. } => 1,
            Self::SourceExists(_) => 2,
            Self::InvalidSourceUrl(_) | Self::Config(_) => 3,
            Self::GitHubApi(_) | Self::RateLimited(_) => 4,
            Self::NoEmbeddings => 5,
            Self::LibraryNotFound(_) => 6,
            Self::Database(_) | Self::Io(_) => 10,
        }
    }
}

pub type Result<T> = std::result::Result<T, HelixDocsError>;
```

---

## Future Work: Version Handling

**Current approach (v1):** Sources are identified by URL only. Version/ref is metadata, not part of the ID.

```rust
// v1: URL-only key
let id = SourceId::from_key("github.com/facebook/react");
// Same ID regardless of version
```

**Problem:** User A wants React 17 docs, User B wants React 18 docs. With URL-only keys, they conflict.

**Future approach (v2):** Include version/ref in the key when specified.

```rust
// v2: URL + version key (when version is specified)
let id = SourceId::from_parts(&["github.com/facebook/react", "v18.2.0"]);
// Different ID per version

// Unversioned still works (defaults to latest/main)
let id = SourceId::from_key("github.com/facebook/react");
```

**Design considerations for v2:**
- Version could be: git tag, branch, commit SHA, or semantic version
- Need to normalize versions (e.g., `v18.2.0` vs `18.2.0`)
- Search should support filtering by version
- May need garbage collection for old versions

**For now:** Punt on versioning. Cache based on URL only, always fetch default branch.

---

## File Formats

### Seed Library List (`data/libraries.yml`)

```yaml
# Popular documentation sources for seeding

- url: https://github.com/facebook/react
  docs: docs
  version: "18.x"

- url: https://github.com/vercel/next.js
  docs: docs
  version: "14.x"

- url: https://github.com/tokio-rs/tokio
  docs: docs
  
- url: https://github.com/honojs/hono
  docs: docs

# Future: website sources (not implemented)
# - url: https://docs.rs
#   type: website
#   allow: ["/tokio/", "/serde/"]
```

### JSON Output Schemas

```json
// Source list output
{
  "sources": [
    {
      "id": "src-a1b2c3",
      "url": "https://github.com/facebook/react",
      "type": "github",
      "version": "18.x",
      "last_synced_at": "2026-01-05T10:30:00Z",
      "sync_status": "synced",
      "document_count": 45
    }
  ]
}

// Search results output
{
  "query": "hooks",
  "mode": "hybrid",
  "results": [
    {
      "chunk_id": "chk-x1y2z3",
      "doc_id": "doc-a1b2c3",
      "doc_path": "docs/hooks.md",
      "doc_title": "Hooks",
      "library": "facebook/react",
      "version": "18.x",
      "text": "Hooks let you use state and other React features...",
      "score": 0.92,
      "position": {
        "start_line": 10,
        "end_line": 25
      }
    }
  ]
}
```
