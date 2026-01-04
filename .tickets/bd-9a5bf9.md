---
id: bd-9a5bf9
title: Fix lib.rs exports - hide internal modules
status: closed
priority: 2
type: task
created_at: 2026-01-04T19:21:48.882707+00:00
updated_at: 2026-01-04T19:31:08.186474+00:00
closed_at: 2026-01-04T19:31:08.186472+00:00
created_by: kevinchen
created_by_type: human
parent: bd-21a1ce
labels:
- refactor
depends_on:
- id: bd-176b2e
  type: blocks
- id: bd-f52a2a
  type: blocks
---

Change id and markdown modules to pub(crate). Keep curated re-exports. Expose new domain module.

## Comments

### 2026-01-04T19:31:08.186493+00:00 — kevinchen (human)

Closed: Made id and markdown modules pub(crate), exposed domain module

