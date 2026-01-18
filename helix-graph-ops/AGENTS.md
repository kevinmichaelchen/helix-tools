# HELIX-GRAPH-OPS AGENTS

**Parent:** See `../AGENTS.md` for workspace context.

## Overview

HelixDB graph storage helpers for shared patterns like edge writes,
secondary index updates, and adjacency traversal.

## Guidance

- Keep this crate small and HelixDB-specific.
- Avoid domain logic; callers own schemas and mapping.

## Where To Look

| Task                   | Location     |
| ---------------------- | ------------ |
| Graph helper functions | `src/lib.rs` |
