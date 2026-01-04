---
id: bd-85cae7
title: Add default status filter to list command (exclude closed)
status: closed
priority: 2
type: feature
created_at: 2026-01-04T14:06:44.134666+00:00
updated_at: 2026-01-04T14:22:02.185638+00:00
closed_at: 2026-01-04T14:22:02.185637+00:00
created_by: kevinchen
created_by_type: human
labels:
- cli
- spec-compliance
- ux
---

Per AC-003.1: WHEN a user runs hbd list THE SYSTEM SHALL display all non-closed issues. Currently list shows ALL issues including closed ones. Should default to excluding closed unless --status closed is explicitly passed.

## Comments

### 2026-01-04T14:22:02.185750+00:00 — kevinchen (human)

Closed: Implemented: list now excludes closed issues by default

