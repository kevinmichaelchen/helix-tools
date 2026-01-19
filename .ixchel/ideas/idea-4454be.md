---
id: idea-4454be
type: idea
title: 'Ixchel Map: codebase index + symbol graph'
status: proposed
created_at: 2026-01-19T03:08:23Z
updated_at: 2026-01-19T03:08:23Z
created_by: kevinchen
tags: []
---

## Summary

Index a codebase into an explicit “code surface graph” (files, symbols, tests,
ownership) so agents can answer questions like “where is X implemented?” and
produce grounded context without heuristics.

## Notes

- Likely introduces deferred entity kinds (e.g., `file-*`, `sym-*`, `tst-*`).
- Should stay loosely coupled: the indexer produces canonical artifacts; Ixchel
  consumes them for context, search, and linking.
