---
id: iss-6fe7c4
type: issue
title: 'Daemon: background sync + watch for ixchel'
status: open
created_at: 2026-01-19T03:08:24Z
updated_at: 2026-01-19T03:08:24Z
created_by: kevinchen
tags: []
---

## Problem

Ixchel indexing (`ixchel sync`) is currently a foreground operation. For
agent-driven workflows (MCP, editors, long-running research), we want a
background process that keeps `.ixchel/data/` warm and enforces a single-writer
model for HelixDB access.

## Plan

- [ ] Define daemon responsibilities for Ixchel (watch, sync queue, single-writer)
- [ ] Add `ixchel watch` / `ixchel sync --daemon` UX (opt-in)
- [ ] Integrate `ix-daemon` “enqueue sync” with `ix-storage-helixdb` sync
- [ ] Add incremental sync support so daemon work is cheap
- [ ] Add deterministic integration tests (no network) and opt-in E2E tests
