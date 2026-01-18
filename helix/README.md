# Helix

An AI-aware, git-native knowledge graph for agent swarms. Knowledge artifacts stay central, with lightweight agent attribution. Run logs and code-surface indexing are future extensions.

```
helix search "why did we choose PostgreSQL"

┌───────┬────────────┬────────────────────────────────────┬──────────┐
│ Score │ ID         │ Title                              │ Type     │
├───────┼────────────┼────────────────────────────────────┼──────────┤
│ 0.94  │ dec-a1b2c3 │ Use PostgreSQL for Primary Storage │ decision │
│ 0.81  │ iss-b2c3d4 │ Implement connection pooling       │ issue    │
│ 0.76  │ src-pg2024 │ PostgreSQL 16 Performance Guide    │ source   │
└───────┴────────────┴────────────────────────────────────┴──────────┘
```

## What is Helix?

Helix unifies knowledge artifacts with lightweight agent/session attribution:

- **Knowledge** — decisions, issues, ideas, reports, sources, citations
- **Attribution** — agents, sessions (grouping)

Everything is local-first: Markdown manifests in Git, indexed in a graph + vector database for fast traversal and semantic recall.

## Why Helix?

| Problem                                | Helix Solution                                     |
| -------------------------------------- | -------------------------------------------------- |
| "Why did we build it this way?"        | Traverse decision → issue → source → citation      |
| "What is safe to work on right now?"   | `CLAIMS` edges + lease expiry checks on issues     |
| "Generate grounded context for agents" | Chunked embeddings + graph expansion via `context` |
| "Stale/contradictory knowledge"        | Health reports across decisions/issues/reports     |

## Quick Start

### Install

```bash
cargo install helix-cli
```

### Initialize

```bash
cd your-project
helix init
```

Creates:

```
.helix/
├── config.toml
├── decisions/
├── issues/
├── ideas/
├── reports/
├── sources/
├── citations/
├── agents/
└── sessions/
```

### Create Your First Decision

```bash
helix create decision "Use PostgreSQL for primary storage" --edit
```

Opens your `$EDITOR` with a template:

```markdown
---
id: dec-a1b2c3
title: Use PostgreSQL for Primary Storage
status: proposed
date: 2026-01-18
created_by: kevin
tags: []
---

## Context

_Why is this decision needed?_

## Decision

_What did we decide?_

## Consequences

_What are the implications?_
```

### Create an Issue

```bash
helix create issue "Implement connection pooling" \
  --type feature \
  --priority 1 \
  --implements dec-a1b2c3
```

### Search

```bash
helix search "database performance"
```

### View Relationships

```bash
helix graph dec-a1b2c3

dec-a1b2c3: Use PostgreSQL for Primary Storage
├── spawns
│   └── iss-b2c3d4: Implement connection pooling
├── depends_on
│   └── dec-8f9e0d: Use managed infrastructure
└── cites
    └── src-pg2024: PostgreSQL 16 Documentation
```

## Core Concepts

### Entity Families

- **Knowledge**: decision (`dec-`), issue (`iss-`), idea (`idea-`), report (`rpt-`), source (`src-`), citation (`cite-`)
- **Attribution**: agent (`agt-`), session (`ses-`)

### Relationships (examples)

```
Decision ──spawns────────▶ Issue
Issue ─────implements────▶ Decision
Report ────cites─────────▶ Source
Citation ─supports/contradicts▶ Decision|Idea|Report
Agent ─────claims────────▶ Issue (lease with expiry)
```

### Storage

```
.helix/
├── decisions/dec-a1b2c3.md    ← Source of truth (git-tracked)
├── issues/iss-b2c3d4.md
└── data/helix.db/             ← Index cache (gitignored: graph + vectors)
```

Files are Markdown with YAML frontmatter. The database is a rebuildable cache.

## Commands

### Entity Management

```bash
helix create <type> "<title>"  # Create entity
helix show <id>                # Display entity
helix list <type>              # List entities
helix update <id>              # Modify entity
helix delete <id>              # Remove entity
```

### Search & Discovery

```bash
helix search "<query>"         # Semantic search
helix graph <id>               # View relationships
helix context <id>             # Generate AI context
```

### Relationships

```bash
helix link <from> <rel> <to>   # Add relationship
helix unlink <from> <rel> <to> # Remove relationship
```

### Maintenance

```bash
helix init                     # Initialize project
helix sync                     # Sync files ↔ database
helix check                    # Validate all entities
helix health                   # Knowledge health report
```

## Configuration

`.helix/config.toml`:

```toml
[embedding]
provider = "fastembed"
model = "BAAI/bge-small-en-v1.5"

[hooks]
immutable_decisions = true
pre_commit = true
```

## AI Integration

### Generate Context

```bash
helix context iss-17 --depth 2

# Context for iss-b2c3d4: Implement connection pooling
#
# ## This Issue
# **Status:** open | **Priority:** high
# ...
#
# ## Implements Decision
# ### dec-a1b2c3: Use PostgreSQL for Primary Storage
# ...
```

### Agent Attribution

```bash
helix create issue "Found memory leak" \
  --agent claude \
  --session sess-abc123
```

### MCP Server

```bash
helix mcp serve  # Expose tools to Claude Code
```

## TUI

```bash
helix ui
```

```
┌────────────────────────────────────────────────────────────┐
│ helix                                    [main] ● synced   │
├────────────────────────────────────────────────────────────┤
│ ┌──────────────┐ ┌───────────────────────────────────────┐ │
│ │ ▸ All (234)  │ │ Issues (127)             [↑↓ navigate]│ │
│ │   Decisions  │ │                                       │ │
│ │   Issues     │ │ ● iss-b2c3d4  Connection pooling     │ │
│ │   Ideas      │ │   priority:1  [database]              │ │
│ │   Reports    │ │                                       │ │
│ │   Sources    │ │ ○ iss-e5f6g7  Fix memory leak        │ │
│ │   Citations  │ │   priority:0  [parser]                │ │
│ └──────────────┘ └───────────────────────────────────────┘ │
├────────────────────────────────────────────────────────────┤
│ [/] search  [n]ew  [e]dit  [g]raph  [q]uit                │
└────────────────────────────────────────────────────────────┘
```

## Documentation

- [Vision](./specs/vision.md) — Why Helix exists
- [Entities](./specs/entities.md) — Entity type specifications
- [Architecture](./specs/architecture.md) — Technical design
- [CLI](./specs/cli.md) — Command reference
- [TUI](./specs/tui.md) — Terminal UI design
- [Graph Schema](./specs/graph-schema.md) — Database schema
- [AI Integration](./docs/agents.md) — Working with AI agents
- [Roadmap](./docs/roadmap.md) — Implementation plan

## Migration from hbd / helix-decisions

Helix unifies and extends the existing `hbd` (issue tracker) and `helix-decisions` (ADR manager) crates:

```bash
# Migrate existing .tickets/ to .helix/issues/
helix migrate hbd

# Migrate existing .decisions/ to .helix/decisions/
helix migrate helix-decisions
```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development setup.

## License

MIT
