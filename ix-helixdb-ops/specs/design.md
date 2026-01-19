# Design

**Crate:** `ix-helixdb-ops`\
**Purpose:** Shared HelixDB graph storage helpers

## Overview

This crate centralizes small HelixDB storage patterns so downstream crates don’t
re-implement:

- node/edge bincode serialization
- adjacency index updates for edges
- secondary index updates/lookups

## Non-Goals

- Domain-level schemas (labels/properties) — owned by callers
- Higher-level traversals (BFS/shortest-path/etc.) — belongs in domain crates

## Implementation Notes

- Functions are thin wrappers over `helix_db` storage APIs.
- Callers are responsible for transaction lifetimes and error handling strategy.
