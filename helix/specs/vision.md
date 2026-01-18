# Helix: A Unified Knowledge Graph System

## Vision Statement

Helix is a **local-first, git-native knowledge operating system for agent swarms**. It unifies human knowledge with lightweight agent attribution into a single semantically searchable graph so multiple agents can coordinate safely and retain long-horizon memory.

## The Problem

Knowledge is scattered and loses context:

- ADRs get stale or forgotten after approval
- Issues are disconnected from the decisions and sources behind them
- Reports and retrospectives are hard to rediscover
- Ideas lose provenance and evolution trails

We need a **graph that keeps knowledge coherent and attributable** so agents can:

1. Coordinate safely (leases on work items)
2. Recover prior rationale and evidence
3. Answer “why/what/how” across many hops
4. Generate precise, debiased context for other agents and humans

## The Solution

Helix provides:

### 1. Unified Entity Model (Knowledge + Attribution)

- **Knowledge**: decisions, issues, ideas, reports, sources, citations
- **Attribution**: agents, sessions (light grouping)

All share YAML frontmatter, IDs, timestamps, attribution, and embeddings (chunked per section).

### 2. Graph-Vector Storage

HelixDB provides:

- **Graph layer**: Typed nodes and edges for relationship traversal
- **Vector layer**: HNSW index for semantic similarity search
- **Secondary indices**: Fast lookups by ID, status, tags, dates

### 3. Git-Native Persistence

- Source of truth: Markdown files in `.helix/` directories
- HelixDB is a cache/index, rebuildable from files
- Full version history via git
- Branch/merge workflow for knowledge
- Pre-commit hooks for integrity enforcement

### 4. Unified CLI

One command for all entity types:

```bash
helix create decision "Use PostgreSQL for primary storage"
helix create issue "Implement connection pooling" --relates-to dec-42
helix search "database performance" --types decision,issue,source
helix graph dec-42 --depth 3
helix context iss-17 --for-agent
```

### 5. Interactive TUI

- Entity browser with type filtering
- Graph visualization
- Semantic search interface
- Relationship editor
- Knowledge health dashboard

### 6. AI-Native by Default

- Agent/session attribution on every artifact
- Chunked embeddings and centroid reranking for small-context agents
- Deferred: run logs, patches, code-surface indexing (future extensions)

## Design Principles

### 1. Files Are the API

Every entity is a Markdown file. You can:

- Edit with any text editor
- Grep with standard tools
- Diff across git commits
- Review in pull requests

HelixDB indexes files; it doesn't own them.

### 2. Relationships Are First-Class

Edges are as important as nodes:

- `blocks`, `depends_on`, `relates_to`
- `supersedes`, `amends`, `evolves_into`
- `cites`, `quotes`, `supports`, `contradicts`

Graph queries answer real questions:

- "What's blocking this issue?"
- "What decisions cite this paper?"
- "Show me the evolution of this idea"

### 3. Semantic Search Everywhere

Every entity gets embedded. Search by meaning:

- "memory optimization" finds issues about allocation, decisions about pooling, papers about GC
- "authentication" finds OAuth decisions, login bugs, security RFCs

### 4. Knowledge Has Lifecycle

Entities age, evolve, and die:

- Ideas → accepted/rejected
- Decisions → active/superseded/deprecated
- Issues → open/closed
- Sources → cited/forgotten

Helix tracks and surfaces this.

### 5. AI Is a Collaborator

Agents create knowledge too:

- Issues from automated analysis
- Summaries from compaction
- Suggestions from similarity

Attribution matters. Track who (or what) created each artifact.

## Entity Families

| Family      | Type (prefix)                                                                                                   | Purpose                          |
| ----------- | --------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| Knowledge   | decision (`dec-`), issue (`iss-`), idea (`idea-`), report (`rpt-`), source (`src-`), citation (`cite-`)         | Human/agent knowledge artifacts  |
| Attribution | agent (`agt-`), session (`ses-`)                                                                                | Provenance and grouping          |
| Deferred    | run (`run-`), plan (`pln-`), patch (`pch-`), snapshot (`snap-`), file (`file-`), symbol (`sym-`), test (`tst-`) | Future execution/code extensions |

## Relationship Types (high level)

`relates_to`, `blocks`, `depends_on`, `supersedes`, `amends`, `evolves_into`, `spawns`, `implements`, `addresses`, `summarizes`, `cites`, `quotes`, `supports`, `contradicts`, `used_in`, `recommends`, `observes`, `duplicate_of`, `derives_from`, `claims` (issues). Deferred types will add more later.

## Success Metrics

1. **Discoverability**: Find relevant knowledge in <1s semantic search
2. **Traceability**: Answer "why?" by traversing 3+ hops in the graph
3. **Freshness**: Surface stale/orphaned knowledge automatically
4. **Adoption**: Replace bespoke run logs, ADR tools, and code notebooks with Helix
5. **AI Utility**: Generate useful, grounded context for agent swarms and humans

## Non-Goals

- **Cloud sync**: Git handles distribution
- **Real-time collaboration**: Async via git branches
- **WYSIWYG editing**: Markdown is the interface
- **Universal search**: Scoped to project repositories
- **Replacing GitHub Issues**: Complement, not replace

## Prior Art

- **Obsidian**: Graph-based note linking (but not git-native, not typed)
- **Notion**: Unified workspace (but cloud-only, not semantic search)
- **Roam Research**: Bidirectional links (but not structured entities)
- **Logseq**: Local-first outliner (but not graph-vector hybrid)
- **Steve Yegge's Beads**: Inspiration for hbd (but theoretical)
- **ADR Tools**: Decision records (but no search, no relationships)

Helix synthesizes the best of these with:

- Typed entities (not just pages)
- Graph + vector search (not just links)
- Git-native storage (not proprietary sync)
- CLI-first interface (not GUI-only)
- AI-native design (not bolted on)

## The Name

**Helix** evokes:

- **DNA double helix**: Intertwined strands of knowledge
- **Spiral growth**: Ideas evolving through iterations
- **Graph structure**: Nodes connected in complex patterns
- **The Helix editor**: Terminal-native, modern, Rust-based

The CLI is simply `helix`. Crates follow `helix-*` naming.
