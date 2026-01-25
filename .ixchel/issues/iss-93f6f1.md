---
id: iss-93f6f1
type: issue
title: Consider consolidating small crates into ix-core
status: backlog
created_at: 2026-01-25T19:41:25Z
updated_at: 2026-01-25T19:41:25Z
created_by: kevinchen
tags: []
---

## Problem

The workspace has 12 crates, which may be more granular than necessary. Small
utility crates like `ix-config` and `ix-id` could potentially be merged into
`ix-core` to reduce complexity and improve discoverability.

### Current crate structure

| Crate                | Purpose                        | Lines (approx) |
| -------------------- | ------------------------------ | -------------- |
| ix-config            | Config file loading            | Small          |
| ix-id                | Hash-based ID generation       | Small          |
| ix-core              | Domain logic, registries, sync | Medium         |
| ix-embeddings        | Embedding providers            | Medium         |
| ix-storage-surrealdb | SurrealDB backend              | Medium         |
| ix-storage-helixdb   | HelixDB backend                | Medium         |
| ix-app               | Wiring/orchestration           | Small          |
| ix-cli               | CLI binary                     | Small          |
| ix-daemon            | Daemon binary                  | Medium         |
| ix-mcp               | MCP server binary              | Small          |

## Plan

- [ ] Audit crate boundaries and dependencies
- [ ] Evaluate which crates are truly reusable independently
- [ ] Consider merging `ix-config` into `ix-core`
- [ ] Consider merging `ix-id` into `ix-core`
- [ ] Update documentation if changes made

## Considerations

### Arguments for consolidation

- Fewer crates = simpler dependency graph
- Easier onboarding for contributors
- Less publish coordination needed
- `ix-config` and `ix-id` are unlikely to be used outside ixchel

### Arguments against consolidation

- Smaller crates = faster incremental builds
- Clear separation of concerns
- Easier to test in isolation
- Following Rust ecosystem conventions (many small crates)

## Decision

_TBD: Discuss and decide whether to consolidate or keep separate._
