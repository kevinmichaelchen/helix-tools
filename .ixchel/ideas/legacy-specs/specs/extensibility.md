# Extensibility Specification

This document describes how users can extend Ixchel with custom entity types,
relationships, and validation rules.

Note: this spec was drafted under the working name “Ixchel”. The implemented tool
is **Ixchel** (binary `ixchel`, canonical dir `.ixchel/`). Treat `ixchel` → `ixchel`
and `.ixchel/` → `.ixchel/` when reading.

## Design Philosophy

**Progressive complexity**: Start with knowledge-first + lightweight attribution; let users extend or override without forks.

- **Baseline**: Knowledge + attribution built-ins, strict validity matrix
- **v1.x**: Custom entity/relationship definitions, configurable validation modes
- **v2.0**: Full matrix customization and policy hooks

The internal architecture already supports dynamic types; only the UX surface is gated.

## Baseline Built-ins

### Entity Types

| Family      | Type     | Prefix  | Notes                       |
| ----------- | -------- | ------- | --------------------------- |
| Knowledge   | decision | `dec-`  | Immutable after `accepted`  |
|             | issue    | `iss-`  |                             |
|             | idea     | `idea-` | Immutable after `evolved`   |
|             | report   | `rpt-`  | Immutable after `published` |
|             | source   | `src-`  |                             |
|             | citation | `cite-` |                             |
| Attribution | agent    | `agt-`  | Identity for humans/AI      |
|             | session  | `ses-`  | Light grouping              |
| Deferred    | run      | `run-`  | Future extension            |
|             | plan     | `pln-`  | Future extension            |
|             | patch    | `pch-`  | Future extension            |
|             | snapshot | `snap-` | Future extension            |
|             | file     | `file-` | Future extension            |
|             | symbol   | `sym-`  | Future extension            |
|             | test     | `tst-`  | Future extension            |

### Relationships

Core relationships include (see `graph-schema.md` for validity):

`relates_to`, `blocks`, `depends_on`, `supersedes`, `amends`, `evolves_into`, `spawns`, `implements`, `addresses`, `summarizes`, `cites`, `quotes`, `supports`, `contradicts`, `used_in`, `recommends`, `observes`, `duplicate_of`, `derives_from`, minimal `claims` (issues only). Deferred types will gain additional edges later.

### Validity Matrix

Defined in `graph-schema.md`. Strict mode enforces the baseline; permissive/custom modes can widen it.

---

## Future: Custom Entity Types (v1.x)

### Configuration

`.ixchel/entities/meeting.toml`:

```toml
[entity]
name = "meeting"
prefix = "mtg"
directory = "meetings"

[status]
values = ["scheduled", "completed", "cancelled"]
default = "scheduled"
immutable_after = "completed"  # Optional

[fields]
# Required fields (beyond common: id, title, created_at, etc.)
required = ["date", "attendees"]

# Optional fields with types
[fields.date]
type = "date"
description = "Meeting date"

[fields.attendees]
type = "string[]"
description = "List of attendees"

[fields.location]
type = "string"
description = "Meeting location or video link"

[fields.duration_minutes]
type = "integer"
description = "Meeting duration in minutes"

# Relationships this type can participate in
[relationships]
outgoing = [
    { type = "discusses", targets = ["decision", "issue", "idea"] },
    { type = "action_items", targets = ["issue"] },
    { type = "follows_up", targets = ["meeting"] },
]
incoming = [
    { type = "follows_up", sources = ["meeting"] },
]
```

### Runtime Behavior

1. **Discovery**: On `ixchel init` or `ixchel sync`, scan `.ixchel/entities/*.toml`
2. **Registration**: Add to entity type registry
3. **Validation**: Enforce required fields and status values
4. **Storage**: Create directory `.ixchel/meetings/` if needed
5. **CLI**: `ixchel create meeting "Weekly sync"` just works

### File Format

Custom entities use the same Markdown + YAML frontmatter:

```markdown
---
id: mtg-a1b2c3
title: Weekly Architecture Sync
status: completed
date: 2026-01-20
attendees: [kevin, alice, bob]
location: https://zoom.us/j/123456
duration_minutes: 60
created_at: 2026-01-15T10:00:00Z
updated_at: 2026-01-20T11:00:00Z
created_by: kevin
tags: [architecture, weekly]
discusses: [dec-42, idea-17]
action_items: [iss-99, iss-100]
---

## Agenda

1. Review caching decision
2. Discuss new API proposal

## Notes

...

## Action Items

- [ ] @alice: Update caching implementation (iss-99)
- [ ] @bob: Draft API spec (iss-100)
```

### Embedding Strategy

Custom entities are embedded using:

```
{title}

{body}

Tags: {tags}
Type: {entity_type}
```

The `Type: meeting` helps semantic search distinguish between entity types.

---

## Future: Custom Relationships (v1.x)

### Configuration

`.ixchel/relationships/discusses.toml`:

```toml
[relationship]
name = "discusses"
display_name = "Discusses"
inverse_name = "discussed_in"  # Optional: for bidirectional display

[semantics]
blocking = false
transitive = false  # If A discusses B and B discusses C, does A discuss C?

# Optional: restrict which entity types can use this
[validity]
from = ["meeting", "report"]
to = ["decision", "issue", "idea"]

# Optional: cardinality constraints
[cardinality]
max_outgoing = null  # Unlimited
max_incoming = null  # Unlimited
```

### Built-in vs Custom

Built-in relationships are always available. Custom relationships extend the set.

If a custom relationship conflicts with a built-in (same name), the custom one wins for that project.

---

## Future: Custom Validity Matrix (v2.0)

### Why Wait?

The validity matrix is a safety feature. Misconfiguration can lead to:

- Semantically meaningless relationships
- Broken graph queries (expecting certain paths that don't exist)
- Confusion when moving between projects

We want to learn from real usage before exposing this.

### Configuration

`.ixchel/config.toml`:

```toml
[validation]
# Strategy: "strict" (default), "permissive", "custom"
relationship_validation = "custom"

# Only if relationship_validation = "custom"
[validation.matrix]
# Format: "EntityType.relationship -> EntityType"
# Use "*" for any type

rules = [
    # Built-in rules (can be overridden)
    "Decision.supersedes -> Decision",
    "Decision.spawns -> Issue",
    "Issue.blocks -> Issue",

    # Custom rules
    "Meeting.discusses -> *",          # Meeting can discuss anything
    "*.relates_to -> *",               # relates_to between any types
    "RFC.requires_approval -> Decision", # Custom entity + relationship
]

# Explicitly deny certain combinations
deny = [
    "Citation.blocks -> *",            # Citations can't block anything
    "*.supersedes -> Issue",           # Only decisions can be superseded
]
```

### Validation Modes

| Mode         | Behavior                                                |
| ------------ | ------------------------------------------------------- |
| `strict`     | Only explicitly allowed relationships (built-in matrix) |
| `permissive` | Any relationship between any types (no validation)      |
| `custom`     | User-defined rules + denials                            |

### Migration Path

1. **v1.0**: `strict` only, no configuration
2. **v1.x**: Add `permissive` mode for power users
3. **v2.0**: Add `custom` mode with full matrix control

---

## Internal Architecture

### Entity Registry

```rust
pub struct EntityRegistry {
    types: HashMap<String, EntityTypeDefinition>,
}

pub struct EntityTypeDefinition {
    pub name: String,
    pub prefix: String,
    pub directory: String,
    pub status_values: Vec<String>,
    pub immutable_after: Option<String>,
    pub required_fields: Vec<FieldDefinition>,
    pub optional_fields: Vec<FieldDefinition>,
    pub builtin: bool,
}

impl EntityRegistry {
    pub fn load_defaults() -> Self { ... }
    pub fn load_custom(&mut self, dir: &Path) -> Result<()> { ... }
    pub fn get(&self, name: &str) -> Option<&EntityTypeDefinition> { ... }
    pub fn get_by_prefix(&self, prefix: &str) -> Option<&EntityTypeDefinition> { ... }
}
```

### Relationship Registry

```rust
pub struct RelationshipRegistry {
    types: HashMap<String, RelationshipDefinition>,
}

pub struct RelationshipDefinition {
    pub name: String,
    pub display_name: String,
    pub inverse_name: Option<String>,
    pub blocking: bool,
    pub transitive: bool,
    pub builtin: bool,
}
```

### Validity Matrix

```rust
pub struct ValidityMatrix {
    rules: HashSet<ValidityRule>,
    denials: HashSet<ValidityRule>,
    mode: ValidationMode,
}

pub struct ValidityRule {
    pub from_type: TypeMatcher,  // Specific type or wildcard
    pub relationship: String,
    pub to_type: TypeMatcher,
}

pub enum TypeMatcher {
    Specific(String),
    Any,
}

impl ValidityMatrix {
    pub fn allows(&self, from: &str, rel: &str, to: &str) -> bool {
        match self.mode {
            ValidationMode::Strict => self.rules.contains_match(from, rel, to),
            ValidationMode::Permissive => true,
            ValidationMode::Custom => {
                !self.denials.contains_match(from, rel, to)
                    && self.rules.contains_match(from, rel, to)
            }
        }
    }
}
```

### Dynamic Entity Handling

```rust
// Instead of:
pub enum Entity {
    Decision(Decision),
    Issue(Issue),
    // ...
}

// Use:
pub struct DynamicEntity {
    pub type_name: String,
    pub id: String,
    pub title: String,
    pub status: String,
    pub body: String,
    pub metadata: EntityMetadata,
    pub properties: HashMap<String, Value>,  // Type-specific fields
    pub relationships: Vec<Relationship>,
}

impl DynamicEntity {
    pub fn validate(&self, registry: &EntityRegistry) -> Result<()> {
        let def = registry.get(&self.type_name)
            .ok_or(Error::UnknownEntityType)?;

        // Check required fields
        for field in &def.required_fields {
            if !self.properties.contains_key(&field.name) {
                return Err(Error::MissingField(field.name.clone()));
            }
        }

        // Check status validity
        if !def.status_values.contains(&self.status) {
            return Err(Error::InvalidStatus(self.status.clone()));
        }

        Ok(())
    }
}
```

---

## Migration Considerations

### Adding Custom Type to Existing Project

1. Create `.ixchel/entities/meeting.toml`
2. Run `ixchel sync` — registers new type
3. `ixchel create meeting "Weekly sync"` — just works
4. Existing entities unaffected

### Removing Custom Type

1. Delete `.ixchel/entities/meeting.toml`
2. Run `ixchel check` — warns about orphaned entities
3. Options:
   - Delete meeting entities: `ixchel delete --type meeting --all`
   - Keep files (become untyped, excluded from search)

### Changing Validity Rules

1. Update `.ixchel/config.toml` with new rules
2. Run `ixchel check` — validates existing relationships
3. Invalid relationships flagged as warnings
4. Fix or remove invalid relationships

---

## FAQ

### Q: Can I use a custom type without defining it?

No. Unknown prefixes in file names are errors. This prevents typos from creating invalid entities.

```bash
$ ixchel create foo "Test"
Error: Unknown entity type 'foo'. Define it in .ixchel/entities/foo.toml
```

### Q: Can custom types have their own CLI subcommands?

Not in v1.x. Use the generic syntax:

```bash
ixchel create meeting "Title" --property date=2026-01-20
```

Future versions might support generated CLI extensions.

### Q: What about custom validation logic beyond the matrix?

Future consideration. Options:

- TOML-based constraint language
- Lua/Wasm plugins
- Git hooks (already supported)

### Q: Can I share entity definitions across projects?

Not built-in, but you can:

- Copy `.ixchel/entities/*.toml` files
- Create a shared git submodule
- Future: `ixchel plugin install company/entity-definitions`

---

## Appendix: Example Custom Entities

### RFC (Request for Comments)

```toml
[entity]
name = "rfc"
prefix = "rfc"
directory = "rfcs"

[status]
values = ["draft", "review", "approved", "rejected", "withdrawn"]
default = "draft"
immutable_after = "approved"

[fields]
required = ["authors"]

[fields.authors]
type = "string[]"
description = "RFC authors"

[fields.reviewers]
type = "string[]"
description = "Assigned reviewers"

[fields.deadline]
type = "date"
description = "Review deadline"

[relationships]
outgoing = [
    { type = "proposes", targets = ["decision"] },
    { type = "addresses", targets = ["issue", "idea"] },
]
```

### Experiment

```toml
[entity]
name = "experiment"
prefix = "exp"
directory = "experiments"

[status]
values = ["planned", "running", "completed", "abandoned"]
default = "planned"

[fields]
required = ["hypothesis"]

[fields.hypothesis]
type = "string"
description = "What we're testing"

[fields.start_date]
type = "date"
description = "Experiment start"

[fields.end_date]
type = "date"
description = "Experiment end"

[fields.outcome]
type = "string"
description = "Result summary"

[relationships]
outgoing = [
    { type = "tests", targets = ["idea", "decision"] },
    { type = "led_to", targets = ["decision", "issue"] },
]
```

### Milestone

```toml
[entity]
name = "milestone"
prefix = "ms"
directory = "milestones"

[status]
values = ["planned", "in_progress", "completed", "missed"]
default = "planned"

[fields]
required = ["target_date"]

[fields.target_date]
type = "date"
description = "Target completion date"

[fields.actual_date]
type = "date"
description = "Actual completion date"

[relationships]
outgoing = [
    { type = "includes", targets = ["issue", "decision"] },
    { type = "depends_on", targets = ["milestone"] },
]
incoming = [
    { type = "blocks", sources = ["issue"] },
]
```
