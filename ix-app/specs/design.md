# ix-app Design

## Overview

`ix-app` is the wiring/orchestration crate for “top-level” binaries like
`ix-cli` and `ix-mcp`.

It selects concrete adapters (e.g. `ix-storage-helixdb`) based on the repo’s
configuration, then calls the domain-level traits in `ix-core` (e.g.
`IndexBackend`).

## Dependency Rules

- `ix-core` must not depend on concrete adapters.
- Apps should depend on `ix-core` and `ix-app`, not adapter crates directly.

## API

- `sync(repo: &IxchelRepo) -> Result<SyncStats>`
- `search(repo: &IxchelRepo, query: &str, limit: usize) -> Result<Vec<SearchHit>>`
