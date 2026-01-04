# hbd Style Guide

Project-specific conventions for maintaining consistent, idiomatic Rust code.

## Architecture

```
hbd/src/
├── main.rs           # CLI parsing + dispatch only
├── lib.rs            # Curated public API
├── commands/         # CLI command implementations
│   ├── issue.rs      # CRUD operations
│   ├── query.rs      # Read-only queries
│   ├── deps.rs       # Dependency management
│   └── labels.rs     # Labels and comments
├── domain/           # Business logic (testable, no I/O)
│   ├── graph.rs      # Graph algorithms
│   └── filters.rs    # Issue filtering
├── types.rs          # Core data types
├── storage.rs        # File I/O operations
├── error.rs          # Error types
├── id.rs             # ID generation (internal)
└── markdown.rs       # Serialization (internal)
```

### Layer Rules

| Layer | Can Import | Cannot Import |
|-------|------------|---------------|
| `main.rs` | commands, lib exports | domain directly |
| `commands/` | lib exports, domain | storage internals |
| `domain/` | types only | storage, commands, I/O |
| `storage.rs` | types, markdown, id | commands, domain |

## Enums

All enums with string representations implement the trio:

```rust
impl Status {
    pub const fn as_str(self) -> &'static str { ... }
}

impl std::fmt::Display for Status { ... }
impl std::str::FromStr for Status { ... }
```

Use `#[serde(rename_all = "snake_case")]` for serialization.
Use `#[default]` attribute for the default variant.

## Error Handling

- Define error variants in `error.rs` using `thiserror`
- Each variant has a distinct exit code via `exit_code()` method
- Use `?` operator for propagation
- Avoid `Other(String)` for new error cases—add specific variants

```rust
#[derive(Debug, thiserror::Error)]
pub enum HbdError {
    #[error("issue not found: {0}")]
    IssueNotFound(String),
    // ...
}
```

## CLI Commands

Each command function in `commands/`:

1. Takes `&TicketStore` as first parameter (except `init`)
2. Takes `json: bool` as last parameter
3. Returns `hbd::Result<()>`
4. Handles both human and JSON output

```rust
pub fn show(store: &TicketStore, id: &str, json: bool) -> hbd::Result<()> {
    let issue = store.read_issue(id)?;
    
    if json {
        println!("{}", serde_json::to_string_pretty(&issue)?);
    } else {
        // Human-readable output
    }
    Ok(())
}
```

## Builder Pattern

Use for structs with >3 optional fields:

```rust
let issue = Issue::builder("Title")
    .issue_type(IssueType::Bug)
    .priority(Priority::High)
    .labels(["urgent", "backend"])
    .build();
```

Builder methods:
- Return `Self` for chaining
- Use `#[must_use]` attribute
- Accept `impl Into<T>` for string parameters

## Function Parameters

- Use `&str` for borrowed strings
- Use `impl Into<String>` for owned strings in builders
- Use `Option<&str>` for optional string parameters

## Clippy Configuration

Workspace lints in `Cargo.toml`:

```toml
[workspace.lints.clippy]
correctness = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }

# Allowed for pragmatic reasons
module_name_repetitions = "allow"  # IssueType is fine
must_use_candidate = "allow"       # Too noisy
missing_errors_doc = "allow"       # Result<T> is pervasive
too_many_arguments = "allow"       # Builder pattern handles this
items_after_statements = "allow"   # Inline Row structs in functions
implicit_hasher = "allow"          # HashMap<K,V> without generics is fine
```

## Testing

### What to Test

- Graph algorithms in `domain/graph.rs`
- Edge cases: empty input, cycles, not found
- Business logic that doesn't require I/O

### What Not to Test

- CLI parsing (clap tests this)
- Serialization round-trips (serde tests this)
- Simple pass-through functions

### Test Naming

Use descriptive names that read as sentences:

```rust
#[test]
fn detects_cycle_in_a_blocks_b_blocks_c_blocks_a() { ... }

#[test]
fn no_false_positive_when_no_path_exists() { ... }
```

## Code Review Checklist

### Architecture
- [ ] Business logic in `domain/`, not `commands/` or `main.rs`
- [ ] New public items intentionally exposed via `lib.rs`
- [ ] No imports from internal modules (`id`, `markdown`)

### Types
- [ ] Enums implement `as_str()`, `Display`, `FromStr`
- [ ] Structs with >3 fields use builder pattern
- [ ] `#[must_use]` on builder methods returning `Self`

### Error Handling
- [ ] New errors have specific `HbdError` variants
- [ ] `?` operator used (not manual `match`)
- [ ] Error messages are actionable

### CLI
- [ ] Command accepts `--json` flag
- [ ] Human output uses tables/formatting
- [ ] JSON output matches struct serialization

### Testing
- [ ] `domain/` functions have unit tests
- [ ] Edge cases covered
- [ ] Tests don't require filesystem
