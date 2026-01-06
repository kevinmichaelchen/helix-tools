# Requirements

This document defines requirements for `helix-id` using [EARS notation](https://www.iaria.org/conferences2015/filesICCGI15/EARS.pdf).

## EARS Notation Reference

| Pattern | Template |
|---------|----------|
| Ubiquitous | THE SYSTEM SHALL `<action>` |
| Event-driven | WHEN `<trigger>` THE SYSTEM SHALL `<action>` |
| State-driven | WHILE `<state>` THE SYSTEM SHALL `<action>` |
| Optional | WHERE `<feature>` THE SYSTEM SHALL `<action>` |
| Complex | IF `<condition>` THEN THE SYSTEM SHALL `<action>` |

---

## 1. ID Generation

### US-001: Generate Random ID

**As a** developer  
**I want to** generate a unique, prefixed ID  
**So that** I can identify entities without coordination

| ID | Acceptance Criterion |
|----|---------------------|
| AC-001.1 | WHEN `id_random(prefix)` is called THE SYSTEM SHALL return a string in format `{prefix}-{6 hex chars}` |
| AC-001.2 | THE SYSTEM SHALL use UUID v4 as the source of randomness |
| AC-001.3 | THE SYSTEM SHALL hash the UUID with Blake3 before truncating |
| AC-001.4 | THE SYSTEM SHALL use the first 3 bytes (6 hex chars) of the hash |

---

### US-002: Generate Key-Based ID

**As a** developer  
**I want to** generate a deterministic ID from a natural key  
**So that** the same key always produces the same ID (idempotent)

| ID | Acceptance Criterion |
|----|---------------------|
| AC-002.1 | WHEN `id_from_key(prefix, key)` is called THE SYSTEM SHALL hash the key with Blake3 |
| AC-002.2 | THE SYSTEM SHALL return a string in format `{prefix}-{6 hex chars}` |
| AC-002.3 | THE SYSTEM SHALL return the same ID for the same key |

---

### US-002B: Generate Composite Key ID

**As a** developer  
**I want to** generate a deterministic ID from multiple key parts  
**So that** I can create hierarchical IDs (e.g., doc within source)

| ID | Acceptance Criterion |
|----|---------------------|
| AC-002B.1 | WHEN `id_from_parts(prefix, parts)` is called THE SYSTEM SHALL join parts with `:` separator |
| AC-002B.2 | THE SYSTEM SHALL hash the combined string with Blake3 |
| AC-002B.3 | THE SYSTEM SHALL return a string in format `{prefix}-{6 hex chars}` |
| AC-002B.4 | THE SYSTEM SHALL return the same ID for the same parts in the same order |

---

### US-003: Strongly-Typed IDs

**As a** developer  
**I want to** define type-safe ID wrappers  
**So that** I cannot accidentally mix ID types

| ID | Acceptance Criterion |
|----|---------------------|
| AC-003.1 | THE SYSTEM SHALL provide a `define_id!` macro for creating ID types |
| AC-003.2 | WHEN `TypedId::random()` is called THE SYSTEM SHALL return a new random ID with the type's prefix |
| AC-003.3 | WHEN `TypedId::from_key(key)` is called THE SYSTEM SHALL return a deterministic ID |
| AC-003.4 | WHEN `TypedId::from_parts(parts)` is called THE SYSTEM SHALL return a deterministic composite ID |
| AC-003.5 | WHEN `TypedId::from_string(s)` is called THE SYSTEM SHALL wrap the string without validation |
| AC-003.6 | THE SYSTEM SHALL implement `Display`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` for typed IDs |
| AC-003.7 | WHERE the `serde` feature is enabled THE SYSTEM SHALL implement `Serialize` and `Deserialize` |

---

### US-004: ID Parsing

**As a** developer  
**I want to** parse and validate ID strings  
**So that** I can work with IDs from external sources

| ID | Acceptance Criterion |
|----|---------------------|
| AC-004.1 | WHEN `parse_id(s)` is called THE SYSTEM SHALL return the prefix and hash parts |
| AC-004.2 | IF the ID format is invalid THEN THE SYSTEM SHALL return an error |
| AC-004.3 | THE SYSTEM SHALL accept IDs with 6-12 hex characters after the prefix |

---

## 2. Collision Resistance

### US-005: Collision Properties

**As a** developer  
**I want to** understand collision probability  
**So that** I can make informed decisions about ID length

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005.1 | THE SYSTEM SHALL document collision probability for 6-char IDs (~16 million namespace) |
| AC-005.2 | THE SYSTEM SHALL provide configurable hash length via `generate_id_with_length(prefix, bytes)` |
| AC-005.3 | THE SYSTEM SHALL support 3-6 byte lengths (6-12 hex chars) |

---

## Non-Functional Requirements

### NFR-001: Performance

| ID | Requirement |
|----|-------------|
| NFR-001.1 | ID generation SHALL complete in under 1 microsecond |
| NFR-001.2 | Content hashing SHALL process at least 1 GB/s |

### NFR-002: Dependencies

| ID | Requirement |
|----|-------------|
| NFR-002.1 | THE SYSTEM SHALL have minimal dependencies (blake3, uuid, hex) |
| NFR-002.2 | THE SYSTEM SHALL make serde support optional via feature flag |
