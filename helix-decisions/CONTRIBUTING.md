# Contributing to helix-decisions

## Decision Format

All decisions must be markdown files in `.decisions/` with YAML frontmatter.

### Filename Convention

```
{NNN}-{title-in-kebab-case}.md
```

Examples:
- `001-initial-architecture.md`
- `002-database-selection.md`
- `003-api-versioning-strategy.md`

### Required Frontmatter

```yaml
---
id: 1                           # Sequential integer, unique within repo
title: "Decision Title"         # Human-readable title
status: proposed                # proposed | accepted | superseded | deprecated
date: 2026-01-06                # ISO 8601 date
---
```

### Optional Frontmatter

```yaml
---
uuid: hx-a1b2c3                 # Required: hash-based UUID for rename safety
deciders:                       # Who made this decision
  - Alice
  - Bob
tags:                           # Categorization
  - database
  - infrastructure
content_hash: abc123...         # SHA256 hash (set when accepted)
git_commit: def456...           # Git commit hash (set when accepted)
supersedes: 1                   # Decision ID(s) this replaces
amends: [2, 3]                  # Decision ID(s) this amends
depends_on: [4]                 # Prerequisite decision IDs
related_to: [5, 6]              # Related decision IDs
---
```

## Decision Template

```markdown
---
id: {NEXT_ID}
title: {TITLE}
status: proposed
date: {TODAY}
deciders:
  - {YOUR_NAME}
tags:
  - {TAG1}
  - {TAG2}
---

# Context and Problem Statement

Describe the context and problem you're trying to solve.

## Decision Drivers

- Driver 1
- Driver 2

## Considered Options

### Option 1: {NAME}

Description, pros, cons.

### Option 2: {NAME}

Description, pros, cons.

## Decision Outcome

Chosen option: **{CHOSEN}**

### Rationale

Why this option was chosen.

### Consequences

- Good: ...
- Bad: ...
- Neutral: ...
```

## Immutability

### Accepted Decisions Are Immutable

Once a decision has `status: accepted`, it should not be modified directly. This preserves the historical record.

### Amendment Pattern

To update an accepted decision, create a **new decision** with `amends: [id]`:

```yaml
---
id: 10
title: Updated Caching Strategy
status: proposed
date: 2026-01-10
amends: [5]  # References the original decision
---

# Context

Decision 005 established our caching strategy. This decision amends it to...
```

### Supersession Pattern

To replace a decision entirely, create a new decision with `supersedes: [id]`:

```yaml
---
id: 15
title: New Database Approach
status: accepted
date: 2026-01-15
supersedes: [3]  # This decision replaces decision 3
---
```

The original decision's status should be updated to `superseded`:

```yaml
# In the original decision:
status: superseded
superseded_by: 15
```

## Git Hooks

### Installing Hooks

```bash
helix-decisions init-hooks
```

This installs a pre-commit hook that:
- Blocks modifications to accepted decisions
- Allows new decisions (including amendments)
- Can be bypassed with `git commit --no-verify`

### Hook Behavior

| Change | Status | Result |
|--------|--------|--------|
| New decision | any | Allowed |
| Modify decision | proposed | Allowed |
| Modify decision | accepted (no hash) | Allowed |
| Modify decision | accepted (with hash) | **Blocked** |
| New decision with `amends: [id]` | any | Allowed |

## Testing Changes

```bash
# Run all tests
cargo test -p helix-decisions

# Run with embeddings (downloads ~30MB model)
cargo test -p helix-decisions -- --ignored

# Check for issues
cargo clippy -p helix-decisions -- -D warnings
```

## Code Style

- Follow existing patterns in the codebase
- Use `cargo fmt` before committing
- Keep functions focused and well-named
- Add tests for new functionality
