---
id: bd-f52a2a
title: Create domain/filters.rs with ready/blocked/stale logic
status: closed
priority: 1
type: task
created_at: 2026-01-04T19:21:44.581981+00:00
updated_at: 2026-01-04T19:25:38.541794+00:00
closed_at: 2026-01-04T19:25:38.541793+00:00
created_by: kevinchen
created_by_type: human
parent: bd-21a1ce
labels:
- refactor
---

Extract filtering logic: ready_issues(), blocked_issues(), stale_issues(). These are currently inline in cmd_ready, cmd_blocked, cmd_stale.

## Comments

### 2026-01-04T19:25:38.541814+00:00 — kevinchen (human)

Closed: Implemented domain/filters.rs with ready_issues, blocked_issues, stale_issues

