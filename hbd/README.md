# hbd - Helix Issue Tracker Specification

**Version:** 0.1.0  
**Status:** Draft  
**Last Updated:** 2026-01-03

## Overview

`hbd` is a distributed, git-first issue tracker built on
[HelixDB](https://github.com/HelixDB/helix-db), designed for AI-supervised
coding workflows. The name pays homage to both **H**elix and **B**ea**d**s.

It combines the best of [Beads](https://github.com/steveyegge/beads) (AI
compaction, gates, dependency graphs) with HelixDB's unique graph-vector
capabilities (semantic search, path algorithms, hybrid reranking).

## Acknowledgments

The Markdown-first storage approach (`.tickets/*.md` with YAML frontmatter) is
inspired by [wedow/ticket](https://github.com/wedow/ticket), which pioneered the
idea of storing issues as individual Markdown files for better AI agent
compatibility and IDE navigation.

## Goals

1. **Git-first**: Issues stored as Markdown in `.tickets/`, synced via git
2. **Offline-first**: Full functionality without network using local embeddings
   (`fastembed`)
3. **AI-native**: Agent coordination, context compaction, semantic discovery
4. **Graph-powered**: Dependency analysis, cycle detection, critical path
   finding

## Non-Goals (MVP)

- Full Agile/Scrum board UI (use external tools)
- Real-time collaboration (git is the sync mechanism)
- External integrations (GitHub Issues, Jira, Linear sync)
- Multi-tenant SaaS hosting

## Specification Documents

| Document                             | Purpose                                    |
| ------------------------------------ | ------------------------------------------ |
| [requirements.md](./requirements.md) | User stories with EARS acceptance criteria |
| [design.md](./design.md)             | Data model, HelixQL queries, architecture  |
| [tasks.md](./tasks.md)               | Implementation phases and task breakdown   |

## Why Not Just Use Beads?

[Beads](https://github.com/steveyegge/beads) is excellent. Steve Yegge's vision
of git-backed, AI-native issue tracking inspired this project, and we share many
of the same goals: offline-first, dependency-aware, agent-friendly. If you're
happy with beads, keep using it!

**hbd exists because we wanted capabilities that SQLite can't provide.**

### Semantic Search

Beads finds issues by keywords. hbd finds issues by _meaning_.

```bash
# Beads: exact match only
bd list --title-contains "auth"

# hbd: understands "login", "authentication", "sign-in" are related
hbd search "user can't log in" --semantic
hbd similar bd-a1b2   # Find issues like this one
```

When you create an issue, hbd warns you about potential duplicates—even if they
use completely different words.

### Hybrid Search with Reranking

Why choose between keyword and semantic search? hbd fuses them.

```bash
hbd search "memory leak in parser" --hybrid
```

This runs BM25 (keywords) + vector search (meaning), fuses results with
[Reciprocal Rank Fusion](https://plg.uwaterloo.ca/~gvcormac/cormacksigir09-rrf.pdf),
then applies
[MMR](https://www.cs.cmu.edu/~jgc/publication/The_Use_MMR_Diversity_Based_LTMIR_1998.pdf)
for diversity. You get precise matches AND conceptually related issues.

### Graph Algorithms

Beads tracks dependencies. hbd _analyzes_ them.

```bash
# Find the longest chain blocking your epic
hbd critical-path bd-epic-123

# Output:
# Critical path (est. 12.5 hours):
#   bd-f1a2 [P0, 2h] → bd-b3c4 [P1, 4h] → bd-d5e6 [P2, 3h] → bd-epic-123
#   ↑ Start here for maximum impact
```

This uses Dijkstra's algorithm with weights based on priority × estimated time.
Beads can show you _what's_ blocked; hbd tells you _where to focus_.

### Native Graph Storage

Beads uses SQLite with recursive CTEs for dependency traversal. hbd uses
HelixDB—a purpose-built graph database.

| Operation                 | Beads (SQLite)   | hbd (HelixDB)         |
| ------------------------- | ---------------- | --------------------- |
| "What blocks X?"          | Recursive CTE    | Single edge traversal |
| "All transitive blockers" | Complex SQL      | `::Out<DEPENDS_ON>*`  |
| Cycle detection           | Application code | Native BFS            |
| Weighted paths            | Not supported    | Dijkstra built-in     |

### What We Kept from Beads

We're not replacing beads—we're building on its foundation:

- Git-backed storage (`.tickets/*.md`)
- Hash-based IDs (`bd-a1b2c3`) for conflict-free merging
- Dependency tracking with blocking semantics
- AI compaction for context window management
- Gate coordination for async workflows
- Agent tracking with `--agent` and `--session` flags
- `ready` and `blocked` commands
- Full offline support

### What We Skipped

Beads has features we intentionally left out:

- **Molecular Chemistry** (templates, wisps, bonds) — Powerful but complex. We
  use simple epics + labels.
- **12+ issue types** — We have 6: bug, feature, task, epic, chore, gate.
- **Multi-daemon coordination** — Single daemon is simpler.
- **Editor integrations** — Add AGENTS.md yourself; we focus on the core.

### The Bottom Line

| Use Beads if...              | Use hbd if...                 |
| ---------------------------- | ----------------------------- |
| SQLite simplicity is enough  | You want semantic search      |
| You need molecular templates | You need graph algorithms     |
| Mature, battle-tested        | You want vector + BM25 hybrid |
| More editor integrations     | You're already using HelixDB  |

## Key Differentiators vs. Traditional Trackers

| Feature           | GitHub Issues   | Linear          | hbd                       |
| ----------------- | --------------- | --------------- | ------------------------- |
| Storage           | Cloud DB        | Cloud DB        | Local git + HelixDB       |
| Offline           | Read-only cache | Read-only cache | Full read/write           |
| AI context        | None            | AI features     | First-class agent support |
| Semantic search   | None            | Basic           | Vector + BM25 hybrid      |
| Dependency graphs | Basic           | Basic           | Full graph traversal      |
| Self-hosted       | Enterprise only | No              | Yes (default)             |

## Quick Reference

### CLI Commands (Planned)

```bash
# Project setup
hbd init                          # Initialize in current directory
hbd sync                          # Sync git <-> HelixDB
hbd info                          # Show system status

# Issue CRUD
hbd create "Title" --type bug     # Create issue
hbd show <id>                     # Display issue details
hbd list --status open            # List with filters
hbd update <id> --priority 1      # Modify issue
hbd close <id> --reason "Done"    # Close issue

# Labels
hbd label add <id> bug,urgent     # Add labels
hbd label remove <id> bug         # Remove label
hbd label list <id>               # Show issue labels
hbd label list-all                # Show all project labels

# Comments
hbd comment <id> "message"        # Add comment
hbd comments <id>                 # List comments

# Dependencies
hbd dep add <from> blocks <to>    # Add dependency
hbd dep remove <from> blocks <to> # Remove dependency
hbd dep list <id>                 # Show dependencies
hbd dep cycles                    # Find all dependency cycles
hbd ready                         # Unblocked issues
hbd blocked                       # Blocked issues
hbd explain <id>                  # Show blocker chain
hbd critical-path <epic-id>       # Longest blocking chain
hbd graph <id>                    # DOT format visualization

# Search & Discovery
hbd search "query"                # BM25 text search
hbd search "query" --semantic     # Vector similarity search
hbd search "query" --hybrid       # BM25 + vector fusion
hbd similar <id>                  # Find similar issues
hbd stale --days 14               # Find forgotten issues

# AI/Agent features
hbd compact                       # Summarize old issues
hbd context --query "topic"       # Get context for LLM
hbd wait <gate-id>                # Wait for gate condition
hbd restore <id>                  # View pre-compaction content

# Analytics
hbd health                        # Project health signals
hbd stats                         # Issue statistics
hbd count --status open           # Quick count

# Maintenance
hbd merge <ids> --into <target>   # Merge duplicates
hbd admin cleanup --older-than 90 # Delete old closed issues
```

### File Structure

```
your-project/
├── .tickets/
│   ├── bd-a1b2c3.md             # Issue files (YAML frontmatter + Markdown)
│   ├── bd-d4e5f6.md
│   └── ...
├── .helix/
│   ├── config.toml              # hbd configuration
│   ├── helix.db/                # HelixDB data directory
│   └── models/                  # Cached embedding models
└── helix.toml                   # HelixDB schema
```

### Issue File Format

```markdown
---
id: bd-a1b2c3
title: Fix memory leak in parser
status: open
priority: 1
type: bug
created_at: 2026-01-03T10:30:00Z
created_by: kevin
labels:
  - performance
  - parser
depends_on:
  - bd-x7y8z9
---

## Description

The parser leaks memory when processing large files...

## Acceptance Criteria

- [ ] No memory growth over 1000 file parses
- [ ] Add regression test
```

## Technology Stack

| Component  | Technology | Why                                        |
| ---------- | ---------- | ------------------------------------------ |
| Language   | Rust       | Performance, safety, HelixDB compatibility |
| Database   | HelixDB    | Graph + vector + BM25 in one               |
| Embeddings | fastembed  | Native Rust, offline, no server            |
| CLI        | clap       | Standard Rust CLI framework                |
| Git ops    | git2       | Native Rust git bindings                   |
| File watch | notify     | Cross-platform file watching               |

## Related Documents

- [HelixDB Documentation](https://github.com/HelixDB/helix-db)
- [Beads Architecture](https://github.com/steveyegge/beads/blob/main/docs/ARCHITECTURE.md)
- [Kiro Specs Concept](https://kiro.dev/docs/specs/concepts/)
