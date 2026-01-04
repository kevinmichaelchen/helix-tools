<script lang="ts">
  import type { PageData } from './$types';
  import { invalidateAll, goto } from '$app/navigation';
  import { page } from '$app/stores';
  import IssuePanel from '$lib/components/ui/IssuePanel.svelte';
  import DetailPanel from '$lib/components/ui/DetailPanel.svelte';
  import IssueCard from '$lib/components/ui/IssueCard.svelte';
  import CommandPalette from '$lib/components/ui/CommandPalette.svelte';
  import NodeContextMenu from '$lib/components/ui/NodeContextMenu.svelte';
  import type { Issue, LayoutMode, Status, Priority, IssueType } from '$lib/types/issue';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { Sheet, SheetContent } from '$lib/components/ui/sheet';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';

  let { data }: { data: PageData } = $props();

  const urlIssueId = $derived($page.url.searchParams.get('issue'));
  const urlView = $derived($page.url.searchParams.get('view') ?? 'table');
  const urlSearch = $derived($page.url.searchParams.get('q') ?? '');

  let selectedIssue = $state<Issue | null>(null);
  let isDetailOpen = $state(false);
  let view = $state('table');
  let searchQuery = $state('');

  function updateUrl(params: Record<string, string | null>) {
    const url = new URL($page.url);
    for (const [key, value] of Object.entries(params)) {
      if (value === null || value === '') {
        url.searchParams.delete(key);
      } else {
        url.searchParams.set(key, value);
      }
    }
    goto(url, { replaceState: true, noScroll: true, keepFocus: true });
  }

  $effect(() => {
    if (urlView && urlView !== view) {
      view = urlView;
    }
  });

  $effect(() => {
    if (urlSearch !== searchQuery) {
      searchQuery = urlSearch;
    }
  });

  $effect(() => {
    if (urlIssueId && data.issues.length > 0) {
      const issue = data.issues.find((i) => i.id === urlIssueId);
      if (issue && (!selectedIssue || selectedIssue.id !== urlIssueId)) {
        selectedIssue = issue;
        isDetailOpen = true;
      }
    }
  });

  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  $effect(() => {
    if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
    searchDebounceTimer = setTimeout(() => {
      if (searchQuery !== urlSearch) {
        updateUrl({ q: searchQuery || null });
      }
    }, 300);
    return () => {
      if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
    };
  });

  $effect(() => {
    if (view !== urlView) {
      updateUrl({ view: view === 'table' ? null : view });
    }
  });

  let layoutMode = $state<LayoutMode>('hierarchical');
  let focusedIssueId = $state<string | null>(null);
  let isLoading = $state(false);

  let contextMenuOpen = $state(false);
  let contextMenuPosition = $state({ x: 0, y: 0 });
  let contextMenuIssue = $state<Issue | null>(null);

  const issues = $derived(data.issues);
  const demoMode = $derived(data.demoMode);
  const error = $derived(data.error);

  const priorityToNumber: Record<string, number> = {
    '0': 0,
    Critical: 0,
    '1': 1,
    High: 1,
    '2': 2,
    Medium: 2,
    '3': 3,
    Low: 3,
    '4': 4,
    Backlog: 4,
  };

  function getPriorityNum(p: number | string): number {
    return priorityToNumber[String(p)] ?? 2;
  }

  const filteredIssues = $derived(
    issues
      .filter((i) => {
        if (searchQuery) {
          const query = searchQuery.toLowerCase();
          return (
            i.title.toLowerCase().includes(query) ||
            i.id.toLowerCase().includes(query) ||
            (i.labels && i.labels.some((l) => l.toLowerCase().includes(query)))
          );
        }
        return true;
      })
      .sort((a, b) => {
        if (a.issue_type === 'epic' && b.issue_type !== 'epic') return -1;
        if (b.issue_type === 'epic' && a.issue_type !== 'epic') return 1;
        return getPriorityNum(a.priority) - getPriorityNum(b.priority);
      })
  );

  const statuses: Status[] = ['open', 'in_progress', 'blocked', 'closed'];

  const issuesByStatus = $derived(
    statuses.reduce(
      (acc, status) => {
        acc[status] = filteredIssues.filter((i) => i.status === status);
        return acc;
      },
      {} as Record<Status, Issue[]>
    )
  );

  async function handleRefresh() {
    if (demoMode) return;
    isLoading = true;
    try {
      await invalidateAll();
    } finally {
      isLoading = false;
    }
  }

  function handleSelectIssue(issue: Issue) {
    selectedIssue = issue;
    focusedIssueId = issue.id;
    isDetailOpen = true;
    updateUrl({ issue: issue.id });
  }

  function handleCloseDetail() {
    isDetailOpen = false;
    updateUrl({ issue: null });
  }

  $effect(() => {
    if (!isDetailOpen) {
      const timer = setTimeout(() => {
        if (!isDetailOpen) selectedIssue = null;
      }, 300);
      return () => clearTimeout(timer);
    }
  });

  function handleLayoutChange(mode: LayoutMode) {
    layoutMode = mode;
  }

  function handleNodeContextMenu(event: MouseEvent, issue: Issue) {
    contextMenuIssue = issue;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    contextMenuOpen = true;
  }

  function handleContextMenuClose() {
    contextMenuOpen = false;
    contextMenuIssue = null;
  }

  async function handleStatusChange(issueId: string, status: Status) {
    if (demoMode) return;
    isLoading = true;
    try {
      await fetch(`/api/issues/${issueId}/status`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status }),
      });
      await handleRefresh();
    } finally {
      isLoading = false;
    }
  }

  async function handlePriorityChange(issueId: string, priority: Priority) {
    if (demoMode) return;
    isLoading = true;
    try {
      await fetch(`/api/issues/${issueId}/priority`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ priority }),
      });
      await handleRefresh();
    } finally {
      isLoading = false;
    }
  }

  function handleAddDependency(issueId: string) {
    alert(
      `Add dependency: Coming soon!\n\nUse command line:\nhbd dep add ${issueId} <target-id> --type blocks`
    );
  }

  function handleOpenInEditor(issueId: string) {
    alert(`Open in editor: Coming soon!\n\nUse command line:\nhbd edit ${issueId}`);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
    if (commandPaletteOpen) return;

    switch (e.key) {
      case 'j':
      case 'ArrowDown':
        e.preventDefault();
        if (filteredIssues.length > 0) {
          const selectedId = selectedIssue?.id;
          const currentIndex = selectedId
            ? filteredIssues.findIndex((i) => i.id === selectedId)
            : -1;
          const nextIndex = currentIndex < filteredIssues.length - 1 ? currentIndex + 1 : 0;
          handleSelectIssue(filteredIssues[nextIndex]);
        }
        break;
      case 'k':
      case 'ArrowUp':
        e.preventDefault();
        if (filteredIssues.length > 0) {
          const selectedId = selectedIssue?.id;
          const currentIndex = selectedId
            ? filteredIssues.findIndex((i) => i.id === selectedId)
            : 0;
          const prevIndex = currentIndex > 0 ? currentIndex - 1 : filteredIssues.length - 1;
          handleSelectIssue(filteredIssues[prevIndex]);
        }
        break;
      case 'Escape':
        if (isDetailOpen) {
          handleCloseDetail();
        }
        break;
    }
  }

  let commandPaletteOpen = $state(false);
  function triggerCommandPalette() {
    commandPaletteOpen = true;
  }
</script>

<svelte:document onkeydown={handleKeydown} />

<div class="flex h-screen w-screen overflow-hidden bg-background text-foreground">
  <Tabs bind:value={view} class="flex h-full w-full flex-col">
    <header class="flex h-14 items-center justify-between border-b bg-card px-4">
      <div class="flex items-center gap-4">
        <div class="relative w-64">
          <svg
            class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground"
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
          <Input
            type="search"
            placeholder="Search issues..."
            class="pl-9 h-9"
            bind:value={searchQuery}
          />
        </div>

        <TabsList>
          <TabsTrigger value="table" class="flex items-center gap-2">
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6h16M4 10h16M4 14h16M4 18h16"
              />
            </svg>
            Table
          </TabsTrigger>
          <TabsTrigger value="kanban" class="flex items-center gap-2">
            <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
              />
            </svg>
            Kanban
          </TabsTrigger>
        </TabsList>
      </div>

      <div class="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onclick={triggerCommandPalette}
          class="text-muted-foreground"
        >
          <span class="mr-2 text-xs">⌘K</span>
          Command
        </Button>
      </div>
    </header>

    <div class="flex-1 overflow-hidden relative">
      {#if error}
        <div
          class="absolute left-1/2 top-4 z-50 -translate-x-1/2 rounded bg-destructive px-4 py-2 text-sm text-destructive-foreground shadow-lg"
        >
          {error}
        </div>
      {/if}

      <TabsContent value="table" class="h-full m-0 border-0 p-0">
        <IssuePanel issues={filteredIssues} {selectedIssue} onSelectIssue={handleSelectIssue} />
      </TabsContent>

      <TabsContent value="kanban" class="h-full m-0 border-0 p-0">
        <div class="flex h-full gap-4 overflow-x-auto p-4 bg-muted/10">
          {#each statuses as status (status)}
            <div class="w-80 flex-shrink-0 flex flex-col gap-2">
              <div class="flex items-center justify-between px-2">
                <h3 class="font-semibold text-sm uppercase text-muted-foreground tracking-wider">
                  {status.replace('_', ' ')}
                </h3>
                <span class="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
                  {issuesByStatus[status].length}
                </span>
              </div>

              <div class="flex-1 overflow-y-auto rounded-lg p-2 gap-2 flex flex-col">
                {#each issuesByStatus[status] as issue (issue.id)}
                  <IssueCard
                    {issue}
                    selected={selectedIssue?.id === issue.id}
                    onclick={() => handleSelectIssue(issue)}
                  />
                {/each}
              </div>
            </div>
          {/each}
        </div>
      </TabsContent>
    </div>
  </Tabs>

  <Sheet bind:open={isDetailOpen}>
    <SheetContent class="w-[400px] sm:w-[540px] p-0 border-l border-border bg-card">
      {#if selectedIssue}
        <DetailPanel
          issue={selectedIssue}
          {issues}
          onClose={handleCloseDetail}
          onSelectIssue={handleSelectIssue}
        />
      {/if}
    </SheetContent>
  </Sheet>
</div>

<CommandPalette
  {issues}
  {selectedIssue}
  {layoutMode}
  onSelectIssue={handleSelectIssue}
  onLayoutChange={handleLayoutChange}
  onRefresh={handleRefresh}
  bind:open={commandPaletteOpen}
/>

<NodeContextMenu
  bind:open={contextMenuOpen}
  position={contextMenuPosition}
  issue={contextMenuIssue}
  onClose={handleContextMenuClose}
  onStatusChange={handleStatusChange}
  onPriorityChange={handlePriorityChange}
  onAddDependency={handleAddDependency}
  onOpenInEditor={handleOpenInEditor}
/>
