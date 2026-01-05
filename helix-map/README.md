# helix-map

A HelixDB-backed codebase indexer that builds a compact "skeleton" of a
repository (function signatures, types, exports, and module structure) for LLM
context and fast code search. This scaffold defines the product intent and
architecture and a working tracer-bullet proof of concept.

## Why

LLM context windows are too expensive for full source. A skeleton index provides
near-complete codebase awareness at a fraction of the tokens, while remaining
precise enough for tooling, search, and navigation.

## What It Produces

- Symbol graph: functions, types, traits/interfaces, methods, fields, constants
- Edges: exports, imports, implements, extends, calls (when resolvable)
- Skeleton views: per-file and per-module compact signatures
- Repo summary: public surface area and entry points

## Repository Layout

- `README.md` - project overview
- `requirements.md` - requirements in EARS notation
- `design.md` - architecture and data model

## Status

Tracer-bullet PoC: CLI -> scan -> parse -> store -> skeleton output. Rust-only
parsing today, JSON persistence for now.

## Quick Start

```bash
# Index a repository into .helix-map/index.json
cargo run -p helix-map -- index .

# Emit a skeleton (indexes if missing)
cargo run -p helix-map -- skeleton . --output -
```

## Current Scope

- Language support: Rust (`.rs`) only
- Storage: JSON file at `.helix-map/index.json`
- Skeleton output: per-file signatures (public + crate-visible by default)

## Implementation Backlog (High-Value Ideas)

- Add TypeScript/JavaScript extractor with Tree-Sitter queries
- Add Svelte extractor (script + markup exports)
- Add language plug-in registry and per-language config
- Capture doc summaries (`///` / JSDoc) into skeleton output
- Track module tree and re-exports (`pub use`) for public API mapping
- Record field lists for structs/enums and associated items
- Emit trait/impl relationships and where-clause constraints
- Add a minimal import/export graph in the index
- Add a stable symbol ID (hash of qual name + span)
- Add configurable include/exclude patterns and language filters
- Add project scopes for monorepo filtering within one index
- Add deterministic token-budget truncation in skeleton export
- Add JSON Lines export for tooling and streaming
- Add incremental watch mode with file change updates
- Add HelixDB storage backend (replace JSON store)
- Add call graph edges (best-effort) for quick navigation
- Add feature/cfg awareness in Rust symbol visibility
- Identify entrypoints (bin `main`, lib exports, tests, scripts)
- Rank symbols by usage frequency (call counts, import counts)
- Compute graph centrality to surface \"important\" nodes
- Estimate complexity (LOC, nesting depth, cyclomatic hints)
- Detect orchestrators vs. leaf functions (fan-out vs. fan-in)
