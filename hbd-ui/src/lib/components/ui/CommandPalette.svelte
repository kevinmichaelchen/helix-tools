<script lang="ts">
  import type { Issue, LayoutMode, Status } from '$lib/types/issue';

  interface Props {
    issues: Issue[];
    selectedIssue: Issue | null;
    layoutMode: LayoutMode;
    onSelectIssue: (issue: Issue) => void;
    onLayoutChange: (mode: LayoutMode) => void;
    onRefresh: () => void;
    onStatusChange?: (issueId: string, status: Status) => void;
    open?: boolean;
  }

  let {
    issues,
    selectedIssue,
    layoutMode,
    onSelectIssue,
    onLayoutChange,
    onRefresh,
    onStatusChange,
    open = $bindable(false),
  }: Props = $props();
  let search = $state('');
  let selectedIndex = $state(0);
  let inputRef = $state<HTMLInputElement | null>(null);

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      open = true;
      setTimeout(() => inputRef?.focus(), 0);
    }
    if (e.key === 'Escape' && open) {
      closeAndReset();
    }
  }

  function handleDialogKeydown(e: KeyboardEvent) {
    const items = filteredItems;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const item = items[selectedIndex];
      if (item) {
        item.action();
      }
    }
  }

  function closeAndReset() {
    open = false;
    search = '';
    selectedIndex = 0;
  }

  function handleIssueSelect(issue: Issue) {
    onSelectIssue(issue);
    closeAndReset();
  }

  function handleLayoutToggle() {
    onLayoutChange(layoutMode === 'hierarchical' ? 'force' : 'hierarchical');
    closeAndReset();
  }

  function handleRefresh() {
    onRefresh();
    closeAndReset();
  }

  function handleStatusChange(issue: Issue, status: Status) {
    if (onStatusChange) {
      onStatusChange(issue.id, status);
    }
    closeAndReset();
  }

  const statusColors: Record<Status, string> = {
    open: 'bg-gray-500',
    in_progress: 'bg-yellow-500',
    blocked: 'bg-red-500',
    closed: 'bg-green-500',
  };

  const priorityLabels: Record<string, string> = {
    '0': 'P0',
    '1': 'P1',
    '2': 'P2',
    '3': 'P3',
    '4': 'P4',
    Critical: 'P0',
    High: 'P1',
    Medium: 'P2',
    Low: 'P3',
    Backlog: 'P4',
  };

  const typeIcons: Record<string, string> = {
    epic: '◆',
    feature: '★',
    bug: '⚠',
    task: '●',
    chore: '⚙',
  };

  interface CommandItem {
    id: string;
    type: 'action' | 'status' | 'issue';
    label: string;
    icon?: string;
    extra?: string;
    action: () => void;
    keywords: string[];
  }

  const actionItems = $derived<CommandItem[]>([
    {
      id: 'toggle-layout',
      type: 'action',
      label: 'Toggle Layout',
      icon: '⇄',
      extra: layoutMode === 'hierarchical' ? '→ Force' : '→ Hierarchical',
      action: handleLayoutToggle,
      keywords: ['layout', 'view', 'hierarchical', 'force', 'switch'],
    },
    {
      id: 'refresh',
      type: 'action',
      label: 'Refresh Issues',
      icon: '↻',
      action: handleRefresh,
      keywords: ['reload', 'sync', 'update'],
    },
  ]);

  const statusItems = $derived<CommandItem[]>(
    selectedIssue && onStatusChange
      ? (['open', 'in_progress', 'blocked', 'closed'] as Status[])
          .filter((s) => s !== selectedIssue.status)
          .map((status) => ({
            id: `status-${status}`,
            type: 'status' as const,
            label: `Set to ${status.replace('_', ' ')}`,
            icon: '●',
            action: () => handleStatusChange(selectedIssue, status),
            keywords: ['status', 'change', status],
          }))
      : []
  );

  const issueItems = $derived<CommandItem[]>(
    issues.map((issue) => ({
      id: issue.id,
      type: 'issue',
      label: issue.title,
      icon: typeIcons[issue.issue_type] || '●',
      extra: `${issue.id.slice(0, 8)} · ${priorityLabels[String(issue.priority)] ?? issue.priority}`,
      action: () => handleIssueSelect(issue),
      keywords: [issue.id, issue.title, issue.issue_type, issue.status, ...(issue.labels ?? [])],
    }))
  );

  const allItems = $derived([...actionItems, ...statusItems, ...issueItems]);

  const filteredItems = $derived(
    search.trim()
      ? allItems.filter((item) => {
          const searchLower = search.toLowerCase();
          return (
            item.label.toLowerCase().includes(searchLower) ||
            item.keywords.some((k) => k.toLowerCase().includes(searchLower))
          );
        })
      : allItems
  );

  // Reset selection when filtered items change
  $effect(() => {
    if (filteredItems.length > 0 && selectedIndex >= filteredItems.length) {
      selectedIndex = 0;
    }
  });
</script>

<svelte:document onkeydown={handleKeydown} />

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-50 bg-black/60 animate-fade-in"
    onclick={closeAndReset}
    role="button"
    tabindex="-1"
    onkeydown={(e) => e.key === 'Escape' && closeAndReset()}
  ></div>

  <!-- Dialog -->
  <div
    class="command-dialog"
    role="dialog"
    aria-modal="true"
    aria-label="Command Palette"
    tabindex="-1"
    onkeydown={handleDialogKeydown}
  >
    <!-- Search input -->
    <div class="flex items-center border-b border-border-card px-4">
      <svg
        class="mr-2 h-4 w-4 shrink-0 text-foreground-alt"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
        />
      </svg>
      <input
        bind:this={inputRef}
        bind:value={search}
        placeholder="Search issues, commands..."
        class="focus-override h-12 w-full bg-transparent text-sm text-foreground placeholder:text-foreground-alt/50 focus:outline-none"
      />
      <kbd
        class="ml-2 hidden rounded border border-border-card bg-muted px-1.5 py-0.5 text-xs text-foreground-alt sm:inline-block"
      >
        ESC
      </kbd>
    </div>

    <!-- Results list -->
    <div class="max-h-[400px] overflow-y-auto p-2">
      {#if filteredItems.length === 0}
        <div class="py-6 text-center text-sm text-foreground-alt">No results found.</div>
      {:else}
        <!-- Actions group -->
        {#if actionItems.some((a) => filteredItems.includes(a))}
          <div class="px-2 py-1.5 text-xs font-medium text-foreground-alt">Actions</div>
          {#each actionItems.filter((a) => filteredItems.includes(a)) as item (item.id)}
            {@const globalIndex = filteredItems.indexOf(item)}
            <button
              type="button"
              class="command-item"
              class:selected={selectedIndex === globalIndex}
              onclick={item.action}
              onmouseenter={() => (selectedIndex = globalIndex)}
            >
              <span class="text-base">{item.icon}</span>
              <span>{item.label}</span>
              {#if item.extra}
                <span class="ml-auto text-xs text-foreground-alt">{item.extra}</span>
              {/if}
            </button>
          {/each}
        {/if}

        <!-- Status group -->
        {#if statusItems.some((s) => filteredItems.includes(s))}
          <div class="my-2 h-px bg-border-card"></div>
          <div class="px-2 py-1.5 text-xs font-medium text-foreground-alt">
            Change Status: {selectedIssue?.title}
          </div>
          {#each statusItems.filter((s) => filteredItems.includes(s)) as item (item.id)}
            {@const globalIndex = filteredItems.indexOf(item)}
            {@const status = item.id.replace('status-', '') as Status}
            <button
              type="button"
              class="command-item"
              class:selected={selectedIndex === globalIndex}
              onclick={item.action}
              onmouseenter={() => (selectedIndex = globalIndex)}
            >
              <span class="h-2 w-2 rounded-full {statusColors[status]}"></span>
              <span>{item.label}</span>
            </button>
          {/each}
        {/if}

        <!-- Issues group -->
        {#if issueItems.some((issue) => filteredItems.includes(issue))}
          <div class="my-2 h-px bg-border-card"></div>
          <div class="px-2 py-1.5 text-xs font-medium text-foreground-alt">
            Issues ({issueItems.filter((issue) => filteredItems.includes(issue)).length})
          </div>
          {#each issueItems.filter((issue) => filteredItems.includes(issue)) as item (item.id)}
            {@const globalIndex = filteredItems.indexOf(item)}
            {@const issue = issues.find((i) => i.id === item.id)}
            <button
              type="button"
              class="command-item"
              class:selected={selectedIndex === globalIndex}
              onclick={item.action}
              onmouseenter={() => (selectedIndex = globalIndex)}
            >
              <span class="text-base text-foreground-alt">{item.icon}</span>
              <code class="rounded bg-muted px-1 py-0.5 text-xs text-foreground-alt">
                {item.id.slice(0, 8)}
              </code>
              <span class="flex-1 truncate">{item.label}</span>
              {#if issue}
                <span class="h-2 w-2 rounded-full {statusColors[issue.status]}"></span>
              {/if}
              <span class="text-xs text-foreground-alt">
                {priorityLabels[String(issue?.priority)] ?? ''}
              </span>
            </button>
          {/each}
        {/if}
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-between border-t border-border-card px-4 py-2">
      <div class="flex items-center gap-4 text-xs text-foreground-alt">
        <span class="flex items-center gap-1">
          <kbd class="rounded border border-border-card bg-muted px-1">↑↓</kbd>
          navigate
        </span>
        <span class="flex items-center gap-1">
          <kbd class="rounded border border-border-card bg-muted px-1">↵</kbd>
          select
        </span>
      </div>
      <div class="text-xs text-foreground-alt">
        <kbd class="rounded border border-border-card bg-muted px-1">⌘K</kbd>
        to open
      </div>
    </div>
  </div>
{/if}

<style>
  .command-dialog {
    position: fixed;
    left: 50%;
    top: 20%;
    z-index: 50;
    width: 100%;
    max-width: 560px;
    transform: translateX(-50%);
    border-radius: 0.75rem;
    border: 1px solid var(--border-card);
    background: var(--background-alt);
    box-shadow: var(--shadow-popover);
    animation: scale-in 0.2s ease;
  }

  .command-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem;
    border-radius: 0.5rem;
    font-size: 0.875rem;
    color: var(--foreground);
    text-align: left;
    transition: background 0.1s ease;
  }

  .command-item:hover,
  .command-item.selected {
    background: var(--muted);
  }

  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes scale-in {
    from {
      opacity: 0;
      transform: translateX(-50%) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) scale(1);
    }
  }

  .animate-fade-in {
    animation: fade-in 0.15s ease;
  }
</style>
