<script lang="ts">
  import type { Issue } from '$lib/types/issue';

  interface Props {
    issue: Issue;
    issues: Issue[];
    onClose: () => void;
    onSelectIssue: (issue: Issue) => void;
  }

  let { issue, issues, onClose, onSelectIssue }: Props = $props();

  function handleDependencyClick(depId: string) {
    const targetIssue = issues.find((i) => i.id === depId);
    if (targetIssue) {
      onSelectIssue(targetIssue);
    }
  }

  const statusColors: Record<string, string> = {
    open: 'bg-gray-500',
    in_progress: 'bg-yellow-500',
    blocked: 'bg-red-500',
    closed: 'bg-green-500',
  };

  const priorityLabels: Record<string, { label: string; color: string }> = {
    '0': { label: 'Critical', color: 'text-red-400' },
    '1': { label: 'High', color: 'text-orange-400' },
    '2': { label: 'Medium', color: 'text-yellow-400' },
    '3': { label: 'Low', color: 'text-blue-400' },
    '4': { label: 'Backlog', color: 'text-gray-400' },
    Critical: { label: 'Critical', color: 'text-red-400' },
    High: { label: 'High', color: 'text-orange-400' },
    Medium: { label: 'Medium', color: 'text-yellow-400' },
    Low: { label: 'Low', color: 'text-blue-400' },
    Backlog: { label: 'Backlog', color: 'text-gray-400' },
  };

  const labels = $derived(issue.labels ?? []);
  const dependsOn = $derived(issue.depends_on ?? []);
  const comments = $derived(issue.comments ?? []);

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }
</script>

<div class="h-full flex flex-col bg-gray-900">
  <header class="flex items-start justify-between border-b border-gray-700 p-4">
    <div class="min-w-0 flex-1">
      <h3 class="text-lg font-semibold text-white">{issue.title}</h3>
      <code class="mt-1 block text-sm text-gray-500">{issue.id}</code>
    </div>
  </header>

  <div class="flex-1 overflow-y-auto p-4">
    <div class="mb-4 flex flex-wrap gap-2">
      <span class="rounded px-2 py-1 text-xs text-white {statusColors[issue.status]}">
        {issue.status.replace('_', ' ')}
      </span>
      <span class="rounded bg-gray-800 px-2 py-1 text-xs capitalize text-gray-300">
        {issue.issue_type}
      </span>
      <span
        class="rounded bg-gray-800 px-2 py-1 text-xs {priorityLabels[String(issue.priority)]
          ?.color ?? 'text-gray-400'}"
      >
        {priorityLabels[String(issue.priority)]?.label ?? issue.priority}
      </span>
    </div>

    {#if issue.body}
      <div class="mb-4">
        <h4 class="mb-1 text-xs font-semibold uppercase text-gray-500">Description</h4>
        <p class="whitespace-pre-wrap text-sm text-gray-300">{issue.body}</p>
      </div>
    {/if}

    {#if labels.length > 0}
      <div class="mb-4">
        <h4 class="mb-1 text-xs font-semibold uppercase text-gray-500">Labels</h4>
        <div class="flex flex-wrap gap-1">
          {#each labels as label (label)}
            <span class="rounded bg-gray-700 px-2 py-0.5 text-xs text-gray-300">{label}</span>
          {/each}
        </div>
      </div>
    {/if}

    {#if dependsOn.length > 0}
      <div class="mb-4">
        <h4 class="mb-1 text-xs font-semibold uppercase text-gray-500">Dependencies</h4>
        <ul class="space-y-1">
          {#each dependsOn as dep (dep.id)}
            {@const targetIssue = issues.find((i) => i.id === dep.id)}
            <li class="flex items-center gap-2 text-sm">
              <button
                type="button"
                class="rounded bg-gray-800 px-1.5 py-0.5 text-xs font-mono hover:bg-gray-700 transition-colors"
                class:text-blue-400={targetIssue}
                class:text-gray-500={!targetIssue}
                class:cursor-pointer={targetIssue}
                class:cursor-not-allowed={!targetIssue}
                onclick={() => handleDependencyClick(dep.id)}
                disabled={!targetIssue}
                title={targetIssue ? targetIssue.title : 'Issue not found'}
              >
                {dep.id}
              </button>
              <span class="text-xs text-gray-500">{dep.dep_type}</span>
              {#if targetIssue}
                <span class="text-xs text-gray-600 truncate max-w-[150px]">{targetIssue.title}</span
                >
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <div class="grid grid-cols-2 gap-2 text-xs text-gray-500">
      <div>
        <span class="text-gray-600">Created:</span>
        <span>{formatDate(issue.created_at)}</span>
      </div>
      <div>
        <span class="text-gray-600">Updated:</span>
        <span>{formatDate(issue.updated_at)}</span>
      </div>
      {#if issue.assignee}
        <div>
          <span class="text-gray-600">Assignee:</span>
          <span class="text-gray-300">{issue.assignee}</span>
        </div>
      {/if}
      <div>
        <span class="text-gray-600">Created by:</span>
        <span class="text-gray-300">{issue.created_by}</span>
      </div>
    </div>

    {#if comments.length > 0}
      <div class="mt-4 border-t border-gray-700 pt-4">
        <h4 class="mb-2 text-xs font-semibold uppercase text-gray-500">
          Comments ({comments.length})
        </h4>
        <div class="space-y-2">
          {#each comments as comment (comment.id)}
            <div class="rounded bg-gray-800 p-2">
              <div class="mb-1 flex items-center gap-2 text-xs text-gray-500">
                <span class="font-medium text-gray-300">{comment.created_by}</span>
                <span>{formatDate(comment.created_at)}</span>
              </div>
              <p class="text-sm text-gray-300">{comment.body}</p>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
