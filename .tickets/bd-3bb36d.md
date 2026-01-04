---
id: bd-3bb36d
title: Build Table view for issues with virtualization
status: open
priority: 2
type: feature
created_at: 2026-01-04T05:14:13.322144+00:00
updated_at: 2026-01-04T20:20:00.000000+00:00
created_by: kevinchen
created_by_type: human
labels:
- sunday-jan-5th
- ui
- table
- virtualization
depends_on:
- id: bd-1470ef
  type: blocks
- id: bd-c51871
  type: blocks
---

## Summary

Build a virtualized table view for issues per ADR-001. Must handle 10,000+ issues with smooth scrolling.

## Tasks

- [ ] Install TanStack dependencies:
  ```bash
  npm install @tanstack/table-core @tanstack/svelte-virtual
  ```
- [ ] Create `IssueTable.svelte`:
  - Use TanStack Table for data management
  - Use TanStack Virtual for row virtualization
  - Columns: ID, Type (icon), Title, Status (badge), Priority (badge), Labels, Updated
- [ ] Implement table features:
  - [ ] Column sorting (click header to sort)
  - [ ] Column filtering (dropdown filters)
  - [ ] Row selection (checkbox column)
  - [ ] Column resizing (drag column borders)
- [ ] Implement virtualization:
  - [ ] `estimateSize: () => 48` (row height)
  - [ ] `overscan: 20` (buffer rows)
  - [ ] Dynamic height support via `measureElement`
- [ ] Data loading strategy:
  - [ ] Load first 1,000 issues on mount
  - [ ] "Load more" button for additional batches
  - [ ] Client-side sorting/filtering for instant feedback
- [ ] Row interactions:
  - [ ] Click to select issue (dispatch event)
  - [ ] Hover states with cursor-pointer
  - [ ] Keyboard navigation (up/down arrows)

## Performance Targets

| Metric | Target |
|--------|--------|
| Initial render (10K rows) | < 100ms |
| Scroll performance | 60fps |
| DOM nodes | < 50 (regardless of data size) |
| Memory (10K rows) | < 20MB |

## Technical Notes

Per ADR-001:
- TanStack Virtual handles windowing (only renders visible rows)
- TanStack Table handles sorting/filtering in memory
- shadcn-svelte provides styled Table primitives
- Combined approach scales to 50K+ rows

## Rationale

See `.decisions/001-ui-data-rendering-strategy.md` for full ADR.
