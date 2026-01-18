---
id: dec-ac317c
type: decision
title: 'ADR-001: UI Data Rendering Strategy'
status: accepted
date: 2026-01-04
created_at: 2026-01-18T23:33:16Z
updated_at: 2026-01-18T23:33:16Z
created_by: Kevin Chen
tags:
- hbd-ui
- performance
- virtualization
- drag-drop
---

> Migrated from `.decisions/001-ui-data-rendering-strategy.md` into `.ixchel/decisions/`.

# ADR-001: UI Data Rendering Strategy

**Status:** Accepted\
**Date:** 2026-01-04\
**Deciders:** Kevin Chen\
**Tags:** hbd-ui, performance, virtualization, drag-drop

## Context and Problem Statement

hbd-ui needs to handle repositories with 1,000-10,000+ issues efficiently across multiple views:

- **Table view** - Sortable, filterable list of all issues
- **Kanban view** - Drag-drop board with status columns
- **Detail panel** - Single issue with dependency graph

We need to select libraries that:

1. Handle large datasets without performance degradation
2. Support Svelte 5 runes and modern patterns
3. Work well together as a cohesive stack
4. Provide excellent UX for drag-drop interactions

## Decision Drivers

1. **Performance at scale** - Must handle 10K+ issues with smooth scrolling
2. **Svelte 5 compatibility** - Must work with runes (`$state`, `$derived`, `$effect`)
3. **Library cohesion** - Prefer libraries that integrate well together
4. **Drag-drop UX** - Kanban requires accessible, smooth multi-container DnD
5. **Maintenance** - Prefer actively maintained, well-documented libraries

## Considered Options

### Table Libraries

| Option                       | Svelte 5    | Virtual      | Features   | Bundle | Maintenance       |
| ---------------------------- | ----------- | ------------ | ---------- | ------ | ----------------- |
| **shadcn-svelte data-table** | Via adapter | Via TanStack | Full       | ~25KB  | Active (8K stars) |
| svelte-headless-table        | Native      | None         | Full       | ~12KB  | Active            |
| AG Grid                      | Unknown     | Built-in     | Enterprise | ~500KB | Commercial        |
| TanStack Table direct        | Via adapter | Separate     | Full       | ~15KB  | Active (6K stars) |

### Virtualization Libraries

| Option                       | Svelte 5 | Features                    | Bundle | Maintenance       |
| ---------------------------- | -------- | --------------------------- | ------ | ----------------- |
| **@tanstack/svelte-virtual** | Yes      | Row/column, dynamic heights | ~10KB  | Active (TanStack) |
| svelte-virtual-list          | Partial  | Basic windowing             | ~5KB   | Stale             |
| svelte-virtuallists          | Yes      | Lists only                  | ~5KB   | Active            |

### Drag-Drop Libraries

| Option                | Svelte 5 | Multi-container | A11y  | Maintenance        |
| --------------------- | -------- | --------------- | ----- | ------------------ |
| **svelte-dnd-action** | Yes      | Excellent       | Best  | Active (2K stars)  |
| sveltednd             | Yes      | Good            | Basic | Active (477 stars) |
| Native HTML5 DnD      | Yes      | Manual          | Poor  | N/A                |

## Critical Finding: Virtualization + Drag-Drop Incompatibility

**svelte-dnd-action (and all similar libraries) cannot work with virtualized lists.**

### Why This Fails

1. Drag-drop requires all draggable elements to exist in the DOM
2. Virtualization removes off-screen elements to save memory
3. When dragging an item out of the viewport, it gets unmounted
4. Unmounting breaks the active drag operation

### Evidence

```
User drags item #5 → scrolls down → item #5 unmounts → drag breaks
```

This is a fundamental architectural incompatibility, not a bug to be fixed.

### Workarounds Evaluated

| Approach                         | Feasibility | Trade-off                              |
| -------------------------------- | ----------- | -------------------------------------- |
| **Pagination**                   | Recommended | Limits visible items but preserves DnD |
| Disable virtualization in Kanban | Acceptable  | Works for <500 items/column            |
| Custom portal-based drag         | Complex     | 100+ hours of custom work              |
| Virtual drop zones only          | Partial     | Breaks drag preview positioning        |

## Decision

### Chosen Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                        shadcn-svelte                             │
│  (UI primitives: Button, Card, Badge, Command, Sheet, etc.)     │
└──────────────────────────────┬──────────────────────────────────┘
                               │
        ┌──────────────────────┼──────────────────────┐
        │                      │                      │
        v                      v                      v
┌───────────────┐    ┌─────────────────┐    ┌────────────────┐
│  Table View   │    │   Kanban View   │    │  Detail Panel  │
│               │    │                 │    │                │
│ TanStack Table│    │ svelte-dnd-     │    │ shadcn Sheet   │
│ + Virtual     │    │ action          │    │ + dep graph    │
│               │    │                 │    │                │
│ Virtualized   │    │ Paginated       │    │ N/A            │
│ 10K+ rows     │    │ (100/column)    │    │                │
└───────────────┘    └─────────────────┘    └────────────────┘
```

### Libraries

| Purpose        | Library                               | Version  | Rationale                                             |
| -------------- | ------------------------------------- | -------- | ----------------------------------------------------- |
| UI Components  | shadcn-svelte                         | ^1.1.0   | Full Svelte 5, TanStack integration, copy-paste model |
| Table          | @tanstack/table-core + shadcn adapter | ^8.21.3  | Sorting, filtering, column resize, row selection      |
| Virtualization | @tanstack/svelte-virtual              | ^3.13.12 | Official TanStack, handles 50K+ rows                  |
| Drag-Drop      | svelte-dnd-action                     | ^0.9.64  | Best accessibility, multi-container, touch support    |

### Architecture by View

#### Table View (Virtualized)

```svelte
<script lang="ts">
  import { createSvelteTable } from '@tanstack/svelte-table';
  import { createVirtualizer } from '@tanstack/svelte-virtual';
  
  // Load first 1000 issues, paginate the rest
  let issues = $state<Issue[]>([]);
  let hasMore = $state(false);
  
  const table = createSvelteTable({ /* ... */ });
  
  $effect(() => {
    virtualizer = createVirtualizer({
      count: issues.length,
      getScrollElement: () => scrollEl,
      estimateSize: () => 48,
      overscan: 20,
    });
  });
</script>
```

**Behavior:**

- Load first 1,000 issues on mount
- Virtualize rendering (only ~40 DOM nodes regardless of count)
- "Load more" button for additional batches
- Client-side sorting/filtering for instant feedback

#### Kanban View (Paginated)

```svelte
<script lang="ts">
  import { dndzone } from 'svelte-dnd-action';
  
  const PAGE_SIZE = 100;
  
  // Each column shows max 100 items
  let columnPages = $state<Record<Status, number>>({
    open: 0,
    in_progress: 0,
    blocked: 0,
    closed: 0,
  });
  
  function getColumnItems(status: Status) {
    const page = columnPages[status];
    return allIssues
      .filter(i => i.status === status)
      .slice(0, (page + 1) * PAGE_SIZE);
  }
</script>

{#each statuses as status}
  <div class="column" 
       use:dndzone={{ items: getColumnItems(status), type: 'issue' }}
       onconsider={e => handleDnd(status, e)}
       onfinalize={e => handleDnd(status, e)}>
    {#each getColumnItems(status) as issue (issue.id)}
      <IssueCard {issue} />
    {/each}
    {#if hasMoreInColumn(status)}
      <button onclick={() => columnPages[status]++}>Load more</button>
    {/if}
  </div>
{/each}
```

**Behavior:**

- Show 100 items per column initially
- "Load more" within each column
- Full drag-drop functionality preserved
- Performance acceptable up to ~400 total visible items

#### Detail Panel

- Uses shadcn-svelte Sheet component
- No virtualization needed (single issue)
- Dependency graph rendered separately

## Performance Expectations

### Virtualization Thresholds

| Item Count   | Table View            | Kanban View             | Recommended Strategy    |
| ------------ | --------------------- | ----------------------- | ----------------------- |
| < 100        | No virtualization     | No pagination           | Direct render           |
| 100-500      | Optional              | No pagination           | Consider virtualization |
| 500-1,000    | **Required**          | Pagination helpful      | Virtualize table        |
| 1,000-10,000 | **Required**          | **Pagination required** | Full strategy           |
| 10,000+      | Required + pagination | Pagination + filters    | Server-side filtering   |

### Benchmarks (Expected)

| Metric               | Without Virtualization | With Virtualization |
| -------------------- | ---------------------- | ------------------- |
| Initial render (10K) | ~500ms                 | ~50ms               |
| Scroll performance   | Janky (<30fps)         | Smooth (60fps)      |
| Memory (10K items)   | ~50-100MB              | ~5-10MB             |
| DOM nodes            | ~10,000                | ~40                 |

### Configuration

```typescript
// Recommended virtualizer settings
createVirtualizer({
  count: items.length,
  getScrollElement: () => scrollEl,
  estimateSize: () => 48,        // Row height in pixels
  overscan: 20,                  // Buffer rows for smooth scroll
  measureElement: (el) =>        // Dynamic height support
    el.getBoundingClientRect().height,
});
```

## Consequences

### Positive

- **Scalability**: Table view handles 10K+ issues without degradation
- **UX**: Drag-drop remains fully functional with 100 items/column
- **Accessibility**: svelte-dnd-action provides keyboard nav + screen reader support
- **Cohesion**: TanStack ecosystem (Table + Virtual) integrates seamlessly
- **Maintainability**: All libraries actively maintained with strong communities

### Negative

- **Kanban limit**: 100 items/column is a soft cap; heavy usage requires filtering
- **Adapter complexity**: TanStack Table requires shadcn adapter for Svelte 5
- **Learning curve**: TanStack Virtual API requires understanding windowing concepts

### Risks and Mitigations

| Risk                             | Likelihood | Impact | Mitigation                     |
| -------------------------------- | ---------- | ------ | ------------------------------ |
| TanStack Svelte 5 adapter breaks | Low        | High   | Pin versions, monitor releases |
| svelte-dnd-action deprecation    | Low        | Medium | sveltednd as fallback          |
| 100 items/column insufficient    | Medium     | Medium | Add column-level filters       |

## Dependencies

```json
{
  "dependencies": {
    "@tanstack/svelte-virtual": "^3.13.12",
    "@tanstack/table-core": "^8.21.3",
    "bits-ui": "^2.15.2",
    "svelte-dnd-action": "^0.9.64",
    "clsx": "^2.x",
    "tailwind-merge": "^2.x"
  }
}
```

## Related Decisions

- ADR-002: Linting and Formatting Toolchain
- Ticket bd-eb3ae6: Initialize shadcn-svelte
- Ticket bd-31ee3d: Build Kanban view with drag-drop
- Ticket bd-3bb36d: Build Table view for issues

## References

- [TanStack Virtual Documentation](https://tanstack.com/virtual/latest)
- [TanStack Table Documentation](https://tanstack.com/table/latest)
- [svelte-dnd-action GitHub](https://github.com/isaacHagoel/svelte-dnd-action)
- [shadcn-svelte Documentation](https://www.shadcn-svelte.com/)
- [Svelte 5 Runes Documentation](https://svelte.dev/docs/svelte/what-are-runes)
