# Entity Type Specifications (Knowledge + Attribution)

Ixchel focuses on durable knowledge artifacts with lightweight attribution. Run
logs, patches, snapshots, and code-surface entities are deferred.

## Canonical File Format

Entities are Markdown files with YAML frontmatter stored under `.ixchel/`.

Minimum frontmatter expected by the current `ix-core` implementation:

```yaml
id: dec-a1b2c3         # required; `{prefix}-{6..12 hex}`
type: decision         # required; one of: decision|issue|idea|report|source|citation|agent|session
title: Use PostgreSQL  # required
status: proposed       # optional; free-form today
created_at: 2026-01-18T12:00:00Z
updated_at: 2026-01-18T12:00:00Z
created_by: kevin      # optional (from env `IXCHEL_ACTOR` or `USER`)
tags: []               # optional; list of strings
```

## Relationships

Relationships are represented as additional frontmatter keys whose values are
entity IDs or lists of entity IDs (for example `implements: [dec-a1b2c3]`).

Ixchel’s current validator:

- Ignores known metadata keys (`id`, `type`, `title`, `status`, timestamps,
  `created_by`, `tags`)
- Treats the remaining keys as potential relationships
- Only considers values that look like an Ixchel ID (`{prefix}-{hex}`)

## Knowledge Entities

### Decision (`dec-`)

- Typical statuses: `proposed` → `accepted` → [`superseded` | `deprecated`] (or `rejected`)
- Common relationships: `supersedes`, `amends`, `depends_on`, `spawns` (→ issue), `cites` (→ source/citation)

### Issue (`iss-`)

- Statuses are project-defined (Ixchel is permissive today)
- Common relationships: `blocks`/`depends_on` (↔ issue), `implements` (→ decision)

Note: `hbd` issues use the legacy `bd-` prefix, but are stored under
`.ixchel/issues/` and treated as issues by `ix-core` for backward
compatibility.

### Idea (`idea-`)

- Used for early-stage proposals that may evolve into decisions or issues
- Common relationships: `evolves_into`, `relates_to`, `duplicate_of`

### Report (`rpt-`)

- Used for retrospectives, research notes, analyses
- Common relationships: `summarizes`, `cites`, `recommends`, `observes`

### Source (`src-`)

- Used for durable references (papers, docs, repos, PRDs)
- Often linked from decisions/reports via `cites`

### Citation (`cite-`)

- Used for quotable evidence (quote spans + provenance)
- Often linked via `supports`/`contradicts` (→ decision/idea/report) and `quotes` (→ source)

## Attribution Entities (Lightweight)

### Agent (`agt-`)

Represents a human or AI actor.

### Session (`ses-`)

Groups a set of actions/creations under one narrative unit of work.

## ID Generation

Ixchel uses the shared `ix-id` helper crate:

- IDs are `{prefix}-{hex}` where `hex` is 6–12 characters
- Current `ix-core` uses random IDs (UUIDv4 → BLAKE3 → hex) for user-created entities
- Deterministic IDs (from stable keys) are available and may be adopted later

## Directory Layout

```
.ixchel/
├── config.toml
├── decisions/dec-*.md
├── issues/iss-*.md (and legacy `bd-*.md` from hbd)
├── ideas/idea-*.md
├── reports/rpt-*.md
├── sources/src-*.md
├── citations/cite-*.md
├── agents/agt-*.md
├── sessions/ses-*.md
└── data/          # rebuildable cache (gitignored)
```
