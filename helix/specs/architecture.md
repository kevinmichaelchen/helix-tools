# Architecture Specification (Knowledge-First, Agent-Aware)

This document describes the architecture of Helix with a focus on durable knowledge artifacts. Agent attribution is supported; run logs, patches, and code-surface indexing are deferred.

## System Overview

```
┌──────────────────────────────────────────────────────────────────────────┐
│                             Interfaces                                   │
│  CLI (helix)  TUI (helix-tui)  MCP Server (helix-mcp)  Library (helix-core) │
└──────────────┬───────────────────────────────┬────────────────────────────┘
               │                               │
┌──────────────▼───────────────────────────────▼────────────────────────────┐
│                             Core Services                                 │
│  Entity/Type Registry   | Dynamic entities (built-ins + custom)          │
│  Relationship Registry  | Validity matrix + lease/confidence rules       │
│  Chunker & Embedder     | Section chunking + vector generation           │
│  Validator              | Schema + edge validity enforcement             │
│  Context Builder        | Agent-ready context assembly + reranking       │
└──────────────┬───────────────────────────────────────────────────────────┘
               │ Unified Storage API (CRUD, search, graph traversal, sync)
┌──────────────▼───────────────────────────────────────────────────────────┐
│                               Storage                                     │
│  FileStorage   | Markdown manifests for knowledge + attribution entities  │
│  GraphStorage  | HelixDB typed graph (edges, leases, confidence)          │
│  VectorStorage | HNSW index of chunk embeddings + centroid vectors        │
└───────────────┴──────────────────────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────────────────────────────┐
│                            Persistence (Git)                             │
│  .helix/decisions/*.md, issues/*.md, ideas/*.md, reports/*.md            │
│  .helix/sources/*.md, citations/*.md, agents/*.md, sessions/*.md         │
│  .helix/data/helix.db (graph + vectors, gitignored)                      │
└──────────────────────────────────────────────────────────────────────────┘
```

## Crate Structure (high level)

```
helix/               # CLI
helix-core/          # Core library
helix-tui/           # Terminal UI
helix-mcp/           # MCP server
shared/helix-db/     # Graph + vector engine
shared/helix-embeddings/ # Embedder + chunker abstractions
shared/helix-config/ # Config + registry loading
```

Key modules inside `helix-core`:

- `entity/` — dynamic entity model, registry loading (built-ins + `.helix/entities/*.toml`)
- `relationship/` — relationship registry, validity matrix, lease/confidence handling
- `storage/` — file, graph, vector backends + unified façade
- `search/` — hybrid search, vector reranking, filter DSL
- `context/` — context graph expansion + chunk assembly for agents
- `sync/` — file↔graph reconciliation

Deferred modules (future):

- `runlog/` — run/plan/patch/snapshot ingestion
- `code/` — file/symbol/test extraction and ownership/reservations

## Dynamic Entity Model

- All entities share a `DynamicEntity` struct with a `type_name`, `properties`, and `relationships`.
- Built-ins are registered from `entities.md`; custom types load from `.helix/entities/*.toml`.
- The relationship validator enforces the validity rules in `graph-schema.md` and lease semantics on `CLAIMS`.

## Embedding & Chunking

- Chunk textual entities by headings (~512 tokens, 64 overlap); store per-chunk vectors + document centroid.
- Embed citations/sources separately for quotes vs abstracts to improve retrieval precision.

## Context Generation

- Context builder expands graph to a configurable depth, preferring `BLOCKS/DEPENDS_ON`, `SPAWNS`, `SUPPORTS/CONTRADICTS`, `CITES`, and `SUMMARIZES`.
- Output formats: Markdown, JSON, XML. Chunk payloads are pulled for referenced nodes to keep context grounded.
- Confidence and lease metadata are surfaced so agents can decide whether to trust or refresh links.

## Validation & Safety

- Strict mode applies the validity rules in `graph-schema.md`.
- `CLAIMS` edges require a `lease_expires_at`; expired leases are ignored during coordination queries.
- `BLOCKS` and `SUPERSEDES` are cycle-checked.

## Git Model

- Markdown manifests are the source of truth.
- HelixDB (`.helix/data/helix.db`) is a rebuildable cache (graph + vectors).

## MCP Surface (agent-aware)

- Tools expose graph search, creation, linking, and context generation.
- Responses include `confidence` and `lease_expires_at` when relevant for coordination.
