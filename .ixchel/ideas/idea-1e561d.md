---
id: idea-1e561d
type: idea
title: 'Ixchel Repo: repo clone manager + workspace registry'
status: proposed
created_at: 2026-01-19T03:08:24Z
updated_at: 2026-01-19T03:08:24Z
created_by: kevinchen
tags: []
---

## Summary

Provide a global “workspace registry” for cloning, updating, and finding repos
across machines (useful for agents), without changing the canonical per-repo
Ixchel data model.

## Notes

- Global config should have precedence; global caches belong under `~/.ixchel/`.
- Repo knowledge artifacts still live inside each repo at `.ixchel/`.
