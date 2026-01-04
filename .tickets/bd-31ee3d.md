---
id: bd-31ee3d
title: Build Kanban view with drag-drop status changes
status: open
priority: 1
type: feature
created_at: 2026-01-04T05:14:10.191769+00:00
updated_at: 2026-01-04T05:15:31.629215+00:00
created_by: kevinchen
created_by_type: human
labels:
- sunday-jan-5th
- ui
- kanban
- drag-drop
depends_on:
- id: bd-1470ef
  type: blocks
- id: bd-c51871
  type: blocks
---

Install sveltednd for Svelte 5 drag-drop. Create KanbanBoard.svelte with 4 columns (Open, In Progress, Blocked, Closed). Create KanbanColumn.svelte with ScrollArea and dndzone. Create KanbanCard.svelte using shadcn Card with hover effects. Implement onStatusChange when card dropped in new column.
