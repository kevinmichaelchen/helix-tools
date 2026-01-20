---
id: iss-c663c4
type: issue
title: 'Embeddings: chunking + hybrid retrieval improvements'
status: open
created_at: 2026-01-19T03:08:25Z
updated_at: 2026-01-19T03:08:25Z
created_by: kevinchen
tags: [embeddings, chunking, retrieval]
---

## Problem

Ixchel’s vector pipeline is MVP-grade (single embedding per entity, full rebuild
sync). We need “proper” embeddings and indexing: chunking, provenance, and
incremental updates keyed by content hash + model version.

## Plan

- [ ] Introduce heading-aware chunking + chunk IDs (keep Markdown canonical)
- [ ] Store centroid vectors per entity + chunk vectors for precision
- [ ] Track and persist model/version metadata for invalidation
- [ ] Make sync incremental (add/modify/delete) and preserve stable IDs
- [ ] Keep `demo-got` compiling and tests unchanged; add Ixchel tests where needed
