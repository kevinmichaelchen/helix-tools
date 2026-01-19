# AI Agent Integration Guide

This document describes how AI agents (Claude, GPT, Copilot, etc.) can interact
with the Ixchel knowledge graph to provide better assistance.

Note: this document is a blueprint and may describe planned behavior. The
implemented system is **Ixchel** (binary `ixchel`, canonical dir `.ixchel/`) with
an MCP server in `ix-mcp/` (binary `ixchel-mcp`).

## Philosophy

Ixchel treats AI agents as **first-class collaborators**, not just consumers. Agents can:

1. **Read** the knowledge graph to understand context
2. **Create** entities when discovering work items or insights
3. **Link** entities to maintain relationship integrity
4. **Search** semantically to find relevant information
5. **Generate** context for other agents or humans

All agent actions are **attributed** and **traceable**.

## Agent Attribution

Every entity tracks its creator:

```yaml
created_by: claude
created_by_type: agent
agent_id: claude-opus-4-20250514
session_id: sess-abc123
```

This enables:

- Filtering by human vs agent-created content
- Auditing agent contributions
- Grouping related agent actions
- Understanding provenance

## Agent-Native Building Blocks

- **Agents (`agt-`)**: attribution for humans/AI with capabilities.
- **Sessions (`ses-`)**: light grouping for related creations.
- **Reservations (`CLAIMS`)**: leases on issues to avoid collisions.

Agents should attribute creations and acquire leases before taking work; run logs and code-surface indexing remain future extensions.

## CLI Flags for Agents

### `--agent <id>`

Marks the operation as agent-initiated:

```bash
ixchel create issue "Memory leak in parser" --agent claude-opus-4
```

### `--session <id>`

Groups related agent operations:

```bash
# All commands in a session are linked
ixchel create issue "Fix parser" --agent claude --session sess-123
ixchel create issue "Add tests" --agent claude --session sess-123
ixchel link iss-a1 blocks iss-b2 --agent claude --session sess-123
```

### `--json`

Machine-readable output for programmatic parsing:

```bash
ixchel show dec-42 --json | jq '.relationships'
```

## Context Generation

The `ixchel context` command generates AI-ready summaries:

```bash
ixchel context iss-17 --depth 2 --max-tokens 8000 --format markdown
```

### Output Structure

```markdown
# Context for iss-b2c3d4: Implement connection pooling

## This Issue

**Status:** open | **Priority:** high | **Type:** feature
**Assignee:** alice | **Created:** 2026-01-16 by kevin

### Description

Implement PgBouncer-style connection pooling for the database layer.
This should support both transaction and session pooling modes.

## Implements Decision

### dec-a1b2c3: Use PostgreSQL for Primary Storage (accepted)

**Date:** 2026-01-15 | **Deciders:** kevin, alice, bob

We decided to use PostgreSQL 16 for primary storage because:

- Strong ACID guarantees
- Excellent JSON support
- Mature ecosystem

## Blocking Issues

### iss-x1y2z3: Set up dev database (open)

Development database environment needs to be configured before
connection pooling can be implemented and tested.

## Relevant Sources

### src-pg2024: PostgreSQL 16 Connection Management

Official documentation on connection handling, pooling strategies,
and configuration parameters.

Key sections:

- 20.3 Connection via TCP/IP
- 20.4 Resource Limits
```

### Format Options

| Format     | Use Case                                      |
| ---------- | --------------------------------------------- |
| `markdown` | Human-readable, works well in chat interfaces |
| `xml`      | Structured for Claude-style XML tags          |
| `json`     | Programmatic processing                       |

### XML Format Example

```xml
<context entity="iss-b2c3d4">
  <issue>
    <id>iss-b2c3d4</id>
    <title>Implement connection pooling</title>
    <status>open</status>
    <priority>1</priority>
    <body>Implement PgBouncer-style connection pooling...</body>
  </issue>

  <implements>
    <decision id="dec-a1b2c3">
      <title>Use PostgreSQL for Primary Storage</title>
      <status>accepted</status>
      <body>We decided to use PostgreSQL 16...</body>
    </decision>
  </implements>

  <blocked_by>
    <issue id="iss-x1y2z3">
      <title>Set up dev database</title>
      <status>open</status>
    </issue>
  </blocked_by>
</context>
```

## MCP Server Integration

Ixchel can run as an MCP (Model Context Protocol) server, exposing tools to AI assistants:

### Starting the Server

```bash
ixchel mcp serve
```

### Available Tools

#### `ixchel_search`

Semantic search across the knowledge graph.

```json
{
  "name": "ixchel_search",
  "arguments": {
    "query": "database performance optimization",
    "types": ["decision", "issue", "source"],
    "limit": 10
  }
}
```

#### `ixchel_show`

Get full details of an entity.

```json
{
  "name": "ixchel_show",
  "arguments": {
    "id": "dec-a1b2c3"
  }
}
```

#### `ixchel_list`

List entities with filters.

```json
{
  "name": "ixchel_list",
  "arguments": {
    "type": "issue",
    "status": ["open", "in_progress"],
    "limit": 20
  }
}
```

#### `ixchel_graph`

Traverse relationships.

```json
{
  "name": "ixchel_graph",
  "arguments": {
    "id": "dec-42",
    "depth": 2,
    "direction": "both"
  }
}
```

#### `ixchel_create`

Create a new entity.

```json
{
  "name": "ixchel_create",
  "arguments": {
    "type": "issue",
    "title": "Fix memory leak in parser",
    "properties": {
      "priority": 1,
      "type": "bug",
      "tags": ["parser", "memory"]
    },
    "relationships": {
      "implements": ["dec-42"]
    }
  }
}
```

#### `ixchel_link`

Add a relationship.

```json
{
  "name": "ixchel_link",
  "arguments": {
    "from": "dec-42",
    "relationship": "spawns",
    "to": "iss-17"
  }
}
```

#### `ixchel_context`

Generate context for an entity.

```json
{
  "name": "ixchel_context",
  "arguments": {
    "id": "iss-17",
    "depth": 2,
    "max_tokens": 8000,
    "format": "markdown"
  }
}
```

### MCP Configuration

Add to your Claude Code MCP settings:

```json
{
  "mcpServers": {
    "ixchel": {
      "command": "ixchel",
      "args": ["mcp", "serve"],
      "cwd": "/path/to/project"
    }
  }
}
```

## Agent Workflows

### 1. Issue Triage

When an agent encounters a bug report:

```bash
# 1. Search for similar issues
ixchel search "memory leak parser" --types issue --limit 5 --json

# 2. Check if related decision exists
ixchel search "parser architecture" --types decision --limit 3 --json

# 3. Create issue if new
ixchel create issue "Memory leak when parsing large files" \
  --type bug \
  --priority 1 \
  --tags parser,memory \
  --agent claude \
  --session $SESSION_ID

# 4. Link to related decision
ixchel link iss-new implements dec-42 --agent claude --session $SESSION_ID
```

### 2. Code Review Context

Before reviewing code, gather context:

```bash
# Get context for the issue being addressed
ixchel context iss-17 --depth 3 --format markdown

# Find related decisions
ixchel search "API error handling" --types decision

# Check for relevant sources
ixchel search "retry patterns" --types source,citation
```

### 3. Knowledge Discovery

When asked "why did we build it this way?":

```bash
# Find the relevant decision
ixchel search "authentication JWT" --types decision

# Show full context including sources and related decisions
ixchel context dec-42 --depth 3

# Visualize the decision chain
ixchel graph dec-42 --depth 4 --direction incoming --format tree
```

### 4. Automated Issue Creation

When an agent discovers work needed:

```bash
# Create parent epic
ixchel create issue "Improve database performance" \
  --type epic \
  --priority 2 \
  --agent claude \
  --session $SESSION_ID

# Create child tasks
ixchel create issue "Implement connection pooling" \
  --type task \
  --parent iss-epic \
  --implements dec-42 \
  --agent claude \
  --session $SESSION_ID

ixchel create issue "Add query caching" \
  --type task \
  --parent iss-epic \
  --depends-on iss-pooling \
  --agent claude \
  --session $SESSION_ID
```

### 5. Research Documentation

When researching a topic:

```bash
# Create source for paper found
ixchel create source "Database Connection Pooling Best Practices" \
  --type article \
  --url "https://example.com/pooling" \
  --agent claude \
  --session $SESSION_ID

# Create citation for key insight
ixchel create citation "Connection pool sizing formula" \
  --from src-pooling \
  --quote "Optimal pool size = (core_count * 2) + effective_spindle_count" \
  --page "Section 4.2" \
  --supports dec-42 \
  --agent claude \
  --session $SESSION_ID
```

## Best Practices for Agents

### 1. Always Use Attribution

```bash
# Good
ixchel create issue "Fix bug" --agent claude --session sess-123

# Bad (no attribution)
ixchel create issue "Fix bug"
```

### 2. Check Before Creating

```bash
# Search first to avoid duplicates
existing=$(ixchel search "connection pooling" --types issue --json)
if [ "$(echo $existing | jq '.results | length')" -eq 0 ]; then
  ixchel create issue "Implement connection pooling" ...
fi
```

### 3. Maintain Relationships

```bash
# When creating implementation issues, link to decisions
ixchel create issue "Add caching" --implements dec-42

# When closing issues, check for blocking relationships
ixchel graph iss-17 --direction incoming --types blocks
```

### 4. Use Sessions for Related Work

```bash
SESSION_ID="sess-$(date +%s)"

# All related operations share session
ixchel create issue "Epic" --session $SESSION_ID
ixchel create issue "Task 1" --session $SESSION_ID
ixchel create issue "Task 2" --session $SESSION_ID
ixchel link iss-epic spawns iss-task1 --session $SESSION_ID
ixchel link iss-epic spawns iss-task2 --session $SESSION_ID
```

### 5. Generate Context Proactively

When starting work on an issue:

```bash
# Get full context before diving in
  context=$(ixchel context iss-17 --depth 2 --format markdown)
  echo "$context"
```

### 6. Avoid Collisions (lightweight)

```bash
# Acquire a lease on an issue for 30 minutes
ixchel claim iss-17 --agent claude --lease 30m
```

## Filtering Agent Content

### View Only Human-Created

```bash
ixchel list issues --created-by-type human
```

### View Agent Contributions

```bash
ixchel list --created-by-type agent
ixchel list --agent claude
ixchel list --session sess-123
```

### Audit Agent Actions

```bash
# All entities created in a session
ixchel list --session sess-123 --json | jq '.[] | {id, title, created_at}'
```

## Error Handling

### Graceful Degradation

If ixchel commands fail, agents should:

1. **Log the error** for debugging
2. **Continue without blocking** the main task
3. **Inform the user** if critical

### Common Errors

| Error            | Handling                               |
| ---------------- | -------------------------------------- |
| Entity not found | Create if appropriate, or inform user  |
| Cycle detected   | Don't create the blocking relationship |
| Ambiguous ID     | Use full ID or ask user to clarify     |
| Sync conflict    | Retry after sync, or use `--no-sync`   |

## Future: Autonomous Knowledge Management

Planned capabilities for agents:

### 1. Automatic Linking

Agent detects relationships in conversation and suggests links:

> "I notice you're discussing the authentication decision (dec-42). Should I link
> this new issue to that decision?"

### 2. Knowledge Health Alerts

Agent proactively surfaces issues:

> "3 ideas have been in draft status for over 30 days. Would you like to review
> them?"

### 3. Context-Aware Search

Agent automatically searches relevant knowledge when user asks questions:

> User: "How should we handle API retries?"
> Agent: _[searches ixchel]_ "Based on dec-78 (API Resilience Decision) and the
> cited source (Exponential Backoff paper), we should..."

### 4. Automatic Summarization

Agent compacts old issues:

```bash
ixchel compact iss-old --agent claude --session $SESSION_ID
# Creates summary, archives original, preserves relationships
```
