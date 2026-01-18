# Entity Type Specifications (Knowledge + Attribution)

Helix focuses on durable knowledge artifacts with lightweight agent attribution. Run logs, patches, snapshots, and code-surface entities are deferred.

## Common Properties

All entities share these base properties (YAML frontmatter keys unless stated otherwise):

```yaml
# Required
id: string              # Unique identifier with type prefix
title: string           # Human-readable title
created_at: datetime
updated_at: datetime

# Attribution
created_by: string      # Actor/agent handle
created_by_type: human | agent | system
agent_id: string        # Specific model/run identity if agent-created
session_id: string      # Session that groups related operations

# Optional
tags: string[]
external_ref: string    # Link to external system (GitHub issue, etc.)
confidence: number      # 0..1 for speculative or auto-created items
```

## Knowledge Entities

### Decision (`dec-`)

- **Status**: `proposed` → `accepted` → [`superseded` | `deprecated`] (or `rejected`)
- **Fields**: `status`, `date`, `deciders[]`, `context`, `consequences[]`, `alternatives[]`
- **Relationships**: `supersedes`, `amends`, `depends_on`, `spawns` (→ Issue), `implements` (← Issue), `supports`/`contradicts` (← Citation), `addresses` (→ Issue/Idea), `used_in` (← Report)

### Issue (`iss-`)

- **Status**: `open` → `claimed` → `in_progress` → [`closed` | `blocked`]
- **Fields**: `status`, `type` (`bug|feature|task|epic|chore`), `priority` (0–4), `assignees[]`, `parent_id`, `estimated_minutes`, `closed_at`, `closed_reason`
- **Relationships**: `blocks`/`depends_on` (↔ Issue), `implements` (→ Decision), `spawned_by` (← Decision), `addresses` (→ Report), `claims` (← Agent)

### Idea (`idea-`)

- **Status**: `draft` → [`proposed` | `parked` | `rejected` | `evolved`]
- **Fields**: `champion`, `effort`, `impact`
- **Relationships**: `evolves_into` (→ Decision/Issue), `inspired_by` (→ Source/Idea/Citation), `duplicate_of`, `relates_to`

### Report (`rpt-`)

- **Status**: `draft` → [`published` | `archived`]
- **Fields**: `report_type` (`postmortem|rfc|retrospective|analysis|research`), `period_start`, `period_end`, `incident_date`
- **Relationships**: `summarizes` (→ Issue/Decision/Session), `cites` (→ Source/Citation), `recommends` (→ Decision/Idea), `addresses` (→ Issue)

### Source (`src-`)

- **Fields**: `source_type`, `url`, `authors[]`, `published_date`, `publisher`, `doi`, `isbn`, `archived_at`, `local_path`
- **Relationships**: `cited_by` (computed), `quotes` (← Citation), `supports`/`contradicts` (← Citation)

### Citation (`cite-`)

- **Fields**: `from_source`, `quote`, `page`, `timestamp`, `is_paraphrase`
- **Relationships**: `supports`/`contradicts` (→ Decision/Idea/Report), `used_in` (→ Report/Decision), `from_source` (→ Source)

## Attribution Entities (Lightweight)

### Agent (`agt-`)

- **Fields**: `kind` (`human|agent|system`), `model` (for AI), `vendor`, `capabilities[]`, `contact`
- **Relationships**: `participates_in` (→ Session), `created` (→ knowledge entities)

### Session (`ses-`)

- **Fields**: `scope`, `started_at`, `ended_at`, `participants[]`, `intent`, `outcome`
- **Relationships**: `groups` (→ knowledge entities), `summary_of` (→ Report)

## Deferred/Out-of-Scope for Now

The following remain future extensions and are not part of the current built-ins: run logs, plans, patches/diffs, workspace snapshots, and code surface (file/symbol/test) nodes.

## Embedding & Chunking Strategy

- **Knowledge & Reports**: chunk by headings/sections; store embeddings per chunk + document-level centroid.
- **Citations/Sources**: embed quotes and abstract separately for high-precision grounding.

## ID Generation

IDs follow `{prefix}-{6-char-hex}` using a BLAKE3 hash of: `prefix + title + created_at + salt`. Partial matching is supported when unambiguous.

## Directory Layout

```
.helix/
├── decisions/dec-*.md
├── issues/iss-*.md
├── ideas/idea-*.md
├── reports/rpt-*.md
├── sources/src-*.md
├── citations/cite-*.md
├── agents/agt-*.md
└── sessions/ses-*.md
```
