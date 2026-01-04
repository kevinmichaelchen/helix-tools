---
id: bd-31ee3d
title: Build Kanban view with drag-drop status changes
status: open
priority: 1
type: feature
created_at: 2026-01-04T05:14:10.191769+00:00
updated_at: 2026-01-04T20:20:00.000000+00:00
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

## Summary

Build a Kanban board view with drag-drop between status columns per ADR-001.

**Key constraint:** Virtualization is incompatible with drag-drop (dragging unmounts elements). Use pagination instead with 100 items per column.

## Tasks

- [ ] Install svelte-dnd-action (2K+ stars, best accessibility):
  ```bash
  npm install svelte-dnd-action
  ```
- [ ] Create `KanbanBoard.svelte`:
  - 4 columns: Open, In Progress, Blocked, Closed
  - Responsive grid layout
  - Column headers with issue counts
- [ ] Create `KanbanColumn.svelte`:
  - `use:dndzone` for drag-drop
  - ScrollArea with max-height
  - **Pagination: 100 items per column initially**
  - "Load more" button when column has >100 items
- [ ] Create `KanbanCard.svelte`:
  - shadcn Card with hover effects
  - Show: title, priority badge, type icon, labels
  - `animate:flip` for smooth reordering
- [ ] Implement drag-drop handlers:
  - `onconsider` for drag preview
  - `onfinalize` to update issue status
  - Call `hbd update <id> --status <new-status>`
- [ ] Add keyboard accessibility:
  - Tab between cards
  - Space/Enter to pick up
  - Arrow keys to move
  - Tab to switch columns

## Technical Notes

Per ADR-001, svelte-dnd-action was chosen over sveltednd for:
- Better accessibility (keyboard + screen reader support)
- Multi-container support (built for Kanban)
- Touch support with `delayTouchStart`
- Active maintenance (2K+ stars)

## Rationale

See `.decisions/001-ui-data-rendering-strategy.md` for full ADR.
