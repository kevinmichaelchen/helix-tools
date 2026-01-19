# Ixchel Workspace Blueprint (Draft)

Lightweight proposal for how the **Ixchel** system could be structured as
first-class crates/apps in the **root workspace**.

Note: `./helix` currently exists for brainstorming/specs. The real crates/apps
should not live under `./helix`.

## Layout (concise)

- `ix-app` (wiring layer for apps)
- `ix-cli` (CLI app)
- `ix-mcp` (MCP server app)
- `ix-core` (core library + façade)
- `ix-storage-helixdb` (HelixDB storage adapter)
- Likely next crates as the system grows:
  - `ix-types` (I/O-free domain types)
  - `ix-config` (config + registry loading)
  - `ix-storage` (traits), `ix-storage-file`, `ix-storage-vector`, `ix-index`
  - `ix-embed`, `ix-suggest`, `ix-testkit`, `ix-obs`

## Coupling Rules (non-negotiables)

- Apps depend on `ix-core` + `ix-app` only (no direct adapter/index/embedding deps).
- `ix-core` depends on traits/interfaces, not concrete storage adapters.
- `ix-types` has no I/O and no runtime framework dependencies (no tokio).
- Adapters implement `ix-storage` traits and do not contain business logic.
- `ix-embed` and `ix-suggest` depend on `ix-types` + storage traits only.
- Dependency graph stays acyclic; adding a cycle triggers a design rethink.

## Responsibilities (target end-state)

- `ix-types`: ID newtypes, enums, DTOs; serde only, no I/O.
- `ix-config`: Load/validate `.ixchel/config.toml` + dynamic entities/relationships.
- `ix-core`: Domain layer for registry, validation, context build; depends on traits, not adapters.
- `ix-app`: Wiring layer that selects adapters and calls `ix-core` traits.
- `ix-storage` (traits): `FileStorage`, `GraphStorage`, `VectorStorage`, `ChunkStore`.
- `ix-storage-file`: Markdown/frontmatter I/O, directory helpers, content hashing + TTL checks.
- `ix-storage-helixdb`: Typed edges/nodes (HelixDB adapter), enforce validity matrix + lease expiry at the boundary.
- `ix-storage-vector`: Vector index adapter with model-version metadata; async batch ingest.
- `ix-index`: HNSW configs + adaptive `ef` guardrails; optional sharding.
- `ix-embed`: Chunker (heading-aware) + embedder abstraction; template helpers for context payloads.
- `ix-suggest`: retrieve→filter→rerank→calibrate pipeline; per-relation calibrators; provenance-based invalidation hooks.
- `ix-cli`: Clap front-end; thin wrappers around `ix-core`.
- `ix-tui`: UI over `ix-core`; no storage specifics.
- `ix-mcp`: MCP tool surface returning typed responses via `ix-core`.
- `ix-testkit`: Sample `.ixchel` trees, golden files, labeled pair evals; harness to gate retrieval/rerank/calibration changes.
- `ix-obs` (optional): Shared logging/metrics/tracing setup.

## Refactor Sequence (short)

1. Extract `ix-types` and `ix-config`; point new code at them.
2. Introduce storage traits + file adapter; replace direct I/O calls with trait usage.
3. Move business logic into `ix-core`; slim the CLI.
4. Add `ix-embed`, `ix-index`, `ix-storage-vector`; wire sync/search through traits.
5. Add `ix-suggest` with calibration + invalidation; gate via `ix-testkit`.

## Implementation Notes

- Provenance/invalidation: store content hash + model version + TTL; on change, re-run `ix-suggest` for affected nodes.
- Calibration: per-relation temperature/Dirichlet stored with model version; surfaced via `ix-suggest`.
- Adaptive HNSW: search-until-k-found or latency-budget-hit in `ix-index` with logs/metrics.
- Context builder stays in `ix-core`; pulls chunk spans from `ix-embed` and uses storage traits only.
