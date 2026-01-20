# Design

**Crate:** `ix-cli`\
**Purpose:** Command-line interface for Ixchel

`ix-cli` is a thin binary wrapper over `ix-core` and a configured index backend.

## Architecture

```
ixchel (bin)
  ├─ ix-core (repo + markdown + validation)
  └─ ix-storage-helixdb (IndexBackend impl; rebuildable cache)
```

## Command Model

- Repo-level commands: `init`, `check`, `sync`
- Entity CRUD: `create`, `show`, `list`, `delete`, `edit`
- Relationships: `link`, `unlink`, `graph`, `context`
- Search: `search`
- Metadata: `tags`

## Tags Command

`ixchel tags` lists all unique tags from entity frontmatter across the repository.

**Use case:** Agents query existing tags before creating entities to maintain vocabulary
consistency. LLMs are smart enough to detect similarity/synonyms themselves.

**Output:** List of unique tags with usage counts (count = number of entities
containing the tag), sorted alphabetically. Tag identity is case-sensitive and
based on trimmed tag values; empty tags are ignored and duplicates within a
single entity count once.

**Flags:**

- `--json`: Output as JSON object with `total` and `tags` array
- `--kind <kind>`: Filter tags to a specific entity kind

## JSON Output

`--json` switches output to JSON for AI/agent consumption. The CLI prints one
JSON object per invocation on stdout.

Example:

```
{
  "total": 2,
  "tags": [
    { "tag": "bug", "count": 5 },
    { "tag": "feature", "count": 3 }
  ]
}
```

## Editor Launch

`ixchel edit <id>` launches `$IXCHEL_EDITOR` or `$EDITOR`, defaulting to `vi`.
