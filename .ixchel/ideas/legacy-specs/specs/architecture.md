# Architecture Specification (Knowledge-First, Agent-Aware)

This document describes the architecture of **Ixchel** with a focus on durable
knowledge artifacts. Agent attribution is supported where available; run logs,
patches, and code-surface indexing are deferred.

## System Overview

```
┌────────────────────────────────────────────────────────────────────────────┐
│                               Interfaces                                   │
│  CLI (ixchel)   MCP Server (ixchel-mcp)   Library (ix-core)   TUI (future) │
└──────────────┬─────────────────────────────────────────────────────────────┘
               │
┌──────────────▼─────────────────────────────────────────────────────────────┐
│                               Core Services                                │
│  Repo/layout     | `.ixchel/` discovery + canonical layout                 │
│  Entity model    | frontmatter parsing + validation                        │
│  Linking         | relationship inference from frontmatter values          │
│  Context builder | 1-hop expansion (MVP)                                   │
│  Sync/search     | delegated to an `IndexBackend` implementation           │
└──────────────┬─────────────────────────────────────────────────────────────┘
               │ IndexBackend (sync/search)
┌──────────────▼─────────────────────────────────────────────────────────────┐
│                                 Storage                                    │
│  Canonical: Markdown files in `.ixchel/`                                   │
│  Cache: HelixDB graph + vectors in `.ixchel/data/` (gitignored)            │
└──────────────┬─────────────────────────────────────────────────────────────┘
               │
┌──────────────▼─────────────────────────────────────────────────────────────┐
│                               Persistence                                  │
│  Git repo containing `.ixchel/**/*.md` (source of truth)                   │
└────────────────────────────────────────────────────────────────────────────┘
```

## Crate Structure (Current Workspace)

- `ix-core`: entity model, markdown parsing, repo discovery/layout, validation,
  and storage-agnostic traits
- `ix-cli`: the `ixchel` binary (thin frontend over `ix-core`)
- `ix-mcp`: the `ixchel-mcp` binary (MCP tool surface)
- `ix-storage-helixdb`: HelixDB-backed cache/index implementing `IndexBackend`
- Shared libs as top-level crates (`ix-id`, `ix-config`, `ix-embeddings`, `ix-daemon`)

## Persistence Model

Ixchel is **git-first**:

- Markdown manifests are canonical (`.ixchel/**/*.md`)
- The HelixDB index is rebuildable and may be deleted/recreated at any time
- `.ixchel/data/` and `.ixchel/models/` are gitignored caches

## Sync/Search Model (MVP)

- `ix-core` defines an `IndexBackend` trait (`sync`, `search`, `health_check`)
- `ix-storage-helixdb` provides a concrete backend that:
  - Rebuilds HelixDB state on `sync` (full rebuild today)
  - Embeds entity text with `fastembed`
  - Performs vector search and returns hits with entity IDs and titles

## Context Generation (MVP)

`ixchel context <id>` builds a simple context pack by:

1. Reading the root entity
2. Expanding 1-hop outgoing relationships (IDs referenced in frontmatter)
3. Returning bodies for the root and linked entities

Future work can add configurable depth, edge-type prioritization, and chunk-span
provenance.

## Relationship Inference (Planned)

For mining large folders of reports/decisions/issues:

1. Chunk retrieval (vector search over chunks)
2. Pair filtering (type-specific heuristics)
3. Rerank/classify (cross-encoder scoring + calibrated confidence)
4. Materialize suggestions (edges with `confidence` + provenance)
