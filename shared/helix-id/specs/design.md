# Design

This document describes the design decisions and implementation details for `helix-id`.

## Overview

`helix-id` provides hash-based ID generation for the helix-tools ecosystem. IDs are designed for git-backed, offline-first systems where coordination between machines is not possible.

## Design Goals

1. **Collision resistance** - Low probability of ID collision across branches/machines
2. **Human readability** - Short enough to type and recognize
3. **Type safety** - Compile-time distinction between ID types
4. **Determinism** - Key-based IDs for deduplication and idempotency
5. **Minimal dependencies** - Small footprint for a foundational crate

## ID Strategy Decision Guide

The library provides two ID generation strategies. Choose based on your entity's characteristics:

### When to Use `from_key()` (Deterministic)

Use deterministic IDs when your entity has a **natural unique identifier**:

| Entity Type | Natural Key | Example |
|-------------|-------------|---------|
| GitHub repo | `owner/repo` | `SourceId::from_key("facebook/react")` |
| URL | Full URL | `PageId::from_key("https://docs.rs/tokio")` |
| File path | Relative path | `FileId::from_key("src/lib.rs")` |
| Git commit | SHA | `CommitId::from_key("abc123def")` |
| Package | `name@version` | `PkgId::from_key("serde@1.0.0")` |

**Benefits of deterministic IDs:**
- Same key always produces same ID (idempotent)
- Natural deduplication (can't create duplicates)
- Enables "upsert" semantics (create or update)
- IDs remain stable across re-indexing

### When to Use `from_parts()` (Composite Keys)

Use for entities that derive identity from a parent + local identifier:

| Entity Type | Key Parts | Example |
|-------------|-----------|---------|
| Doc in repo | `[source_id, path]` | `DocId::from_parts(&[source.as_str(), "docs/intro.md"])` |
| Chunk in doc | `[doc_id, index]` | `ChunkId::from_parts(&[doc.as_str(), "3"])` |
| Symbol in file | `[file_id, name]` | `SymId::from_parts(&[file.as_str(), "parse_args"])` |

**Note:** `from_parts` joins parts with `:` separator then hashes. Order matters!

### When to Use `random()` (Non-deterministic)

Use random IDs only when:
1. Entity has **no natural key** (user-created content)
2. **Duplicates are intentionally allowed** (multiple issues with same title)
3. Entity identity is **truly independent** of its content

| Entity Type | Why Random? | Example |
|-------------|-------------|---------|
| Issue/task | User may create multiple with same title | `IssueId::random()` |
| Comment | Multiple comments with same text allowed | `CommentId::random()` |
| Session | Ephemeral, no natural key | `SessionId::random()` |

**Warning:** Random IDs cannot be regenerated. If you lose the ID, you lose the reference.

## ID Format

```
{prefix}-{hex}

prefix: 2-4 lowercase alphanumeric characters
hex: 6-12 lowercase hexadecimal characters (default: 6)
```

Examples:
- `bd-a1b2c3` (issue ID)
- `src-f4e5d6` (source ID)
- `doc-789abc` (document ID)
- `chk-def012` (chunk ID)

## Algorithm

### Key-Based ID Generation (`from_key`)

```
1. Hash key string with Blake3 (256-bit output)
2. Take first N bytes (default: 3 = 6 hex chars)
3. Encode as lowercase hexadecimal
4. Prepend prefix with hyphen separator
```

Key-based IDs are deterministic: same key → same ID.

### Composite Key ID Generation (`from_parts`)

```
1. Join parts with ":" separator
2. Hash combined string with Blake3
3. Take first N bytes (default: 3 = 6 hex chars)
4. Encode as lowercase hexadecimal
5. Prepend prefix with hyphen separator
```

Example: `from_parts(&["src-abc123", "docs/intro.md"])` hashes `"src-abc123:docs/intro.md"`.

### Random ID Generation (`random`)

```
1. Generate UUID v4 (128 bits of randomness)
2. Hash with Blake3 (256-bit output)
3. Take first N bytes (default: 3 = 6 hex chars)
4. Encode as lowercase hexadecimal
5. Prepend prefix with hyphen separator
```

Why Blake3 over raw UUID truncation?
- Blake3 provides better bit distribution
- Enables key-based IDs with same output format
- Future-proofs against UUID version changes

## Collision Analysis

For 6 hex characters (3 bytes = 24 bits):
- Namespace size: 2^24 = 16,777,216
- Birthday problem: 50% collision at ~4,096 IDs
- 1% collision at ~580 IDs

For typical helix-tools usage:
- Issues per repo: 100-10,000 → Very low collision risk
- Docs per library: 10-1,000 → Very low collision risk
- Chunks per library: 1,000-100,000 → Consider 8-char IDs for large libraries

### Configurable Length

| Bytes | Hex Chars | Namespace | 50% Collision |
|-------|-----------|-----------|---------------|
| 3 | 6 | 16M | 4,096 |
| 4 | 8 | 4B | 65,536 |
| 5 | 10 | 1T | 1,048,576 |
| 6 | 12 | 281T | 16,777,216 |

Default of 6 chars is chosen for human readability while providing adequate collision resistance for typical use cases.

## Type-Safe ID Macro

```rust
define_id!(IssueId, "bd");
```

Expands to:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IssueId(String);

impl IssueId {
    // Deterministic: same key → same ID
    pub fn from_key(key: &str) -> Self {
        Self(id_from_key("bd", key))
    }

    // Deterministic: composite key from parts
    pub fn from_parts(parts: &[&str]) -> Self {
        Self(id_from_parts("bd", parts))
    }

    // Non-deterministic: each call produces unique ID
    pub fn random() -> Self {
        Self(id_random("bd"))
    }

    // Wrap existing ID string (no validation)
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn prefix() -> &'static str {
        "bd"
    }

    // Deprecated methods for backwards compatibility
    #[deprecated(note = "Use random() or from_key()")]
    pub fn generate() -> Self { Self::random() }

    #[deprecated(note = "Use from_key()")]
    pub fn from_content(content: &str) -> Self { Self::from_key(content) }
}

impl Display for IssueId { /* ... */ }
impl AsRef<str> for IssueId { /* ... */ }
```

## API Surface

```rust
// Core functions (v0.2.0+)
pub fn id_from_key(prefix: &str, key: &str) -> String;
pub fn id_from_key_with_length(prefix: &str, key: &str, bytes: usize) -> String;
pub fn id_from_parts(prefix: &str, parts: &[&str]) -> String;
pub fn id_from_parts_with_length(prefix: &str, parts: &[&str], bytes: usize) -> String;
pub fn id_random(prefix: &str) -> String;
pub fn id_random_with_length(prefix: &str, bytes: usize) -> String;
pub fn parse_id(id: &str) -> Result<(String, String), IdError>;

// Deprecated (use new API above)
#[deprecated] pub fn generate_id(prefix: &str) -> String;
#[deprecated] pub fn generate_content_id(prefix: &str, content: &str) -> String;

// Macro
#[macro_export]
macro_rules! define_id { /* ... */ }

// Error type
#[derive(Debug, Error)]
pub enum IdError {
    #[error("Invalid ID format: {0}")]
    InvalidFormat(String),
    #[error("Invalid hex in ID: {0}")]
    InvalidHex(String),
}
```

## Dependencies

| Crate | Purpose | Required |
|-------|---------|----------|
| `blake3` | Hashing | Yes |
| `uuid` | Random generation | Yes |
| `hex` | Hex encoding | Yes |
| `serde` | Serialization | Optional (feature) |
| `thiserror` | Error types | Yes |

## Consumers

This crate is used by:

| Crate | ID Types | Prefix |
|-------|----------|--------|
| `hbd` | IssueId | `bd` |
| `helix-docs` | SourceId, DocId, ChunkId | `src`, `doc`, `chk` |
| `helix-map` | SymbolId, FileId | `sym`, `fil` |

## Migration Notes

### From hbd

Current `hbd/src/id.rs` uses the same algorithm. Migration:
1. Add `helix-id` dependency
2. Replace local `generate_id` with `helix_id::generate_id`
3. Replace `define_id!` macro invocations (API compatible)

### From helix-docs

Current `helix-docs/src/domain/id.rs` uses the same algorithm. Migration:
1. Add `helix-id` dependency
2. Remove local `id.rs` module
3. Re-export from `helix_id`: `pub use helix_id::{define_id, SourceId, DocId, ChunkId};`
