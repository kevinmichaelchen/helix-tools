# Requirements

This document defines user stories and acceptance criteria for `helix-map` using
[EARS notation][ears] (Easy Approach to Requirements Syntax).

[ears]: https://www.iaria.org/conferences2013/filesICCGI13/ICCGI_2013_Tutorial_Terzakis.pdf

## Scope Boundaries

`helix-map` is a codebase mapping tool. It focuses on structural context and
directionality (what exists, how it connects, and what is important) without
judging code quality or enforcing policy.

Out of scope for this project:

- Linting, code style rules, or prescriptive quality warnings
- Security scanning or compliance enforcement
- Automated remediation or refactoring suggestions

## EARS Notation Reference

| Pattern      | Template                                          |
| ------------ | ------------------------------------------------- |
| Ubiquitous   | THE SYSTEM SHALL `<action>`                       |
| Event-driven | WHEN `<trigger>` THE SYSTEM SHALL `<action>`      |
| State-driven | WHILE `<state>` THE SYSTEM SHALL `<action>`       |
| Optional     | WHERE `<feature>` THE SYSTEM SHALL `<action>`     |
| Complex      | IF `<condition>` THEN THE SYSTEM SHALL `<action>` |

---

## 1. Repository Indexing

### US-001: Index a Repository

**As a** developer **I want to** index a repository into a symbol graph **So
that** tools can understand the codebase without full source

| ID       | Acceptance Criterion                                                                                   |
| -------- | ------------------------------------------------------------------------------------------------------ |
| AC-001.1 | WHEN a user indexes a repository THE SYSTEM SHALL scan source files and detect supported languages     |
| AC-001.2 | WHEN a supported file is parsed THE SYSTEM SHALL extract symbol signatures (functions, types, exports) |
| AC-001.3 | WHEN indexing completes THE SYSTEM SHALL store symbols and relationships in HelixDB                    |
| AC-001.4 | WHEN a file is not supported THE SYSTEM SHALL skip it and record a warning                             |

---

### US-002: Incremental Updates

**As a** developer **I want** indexing to be incremental **So that** updates are
fast after small changes

| ID       | Acceptance Criterion                                                        |
| -------- | --------------------------------------------------------------------------- |
| AC-002.1 | WHEN a file hash is unchanged THE SYSTEM SHALL skip re-parsing that file    |
| AC-002.2 | WHEN a file changes THE SYSTEM SHALL update only affected symbols and edges |
| AC-002.3 | THE SYSTEM SHALL remove symbols that no longer exist in the file            |

---

### US-003: Skeleton Output

**As a** developer **I want** a compact skeleton view **So that** LLM context
stays small but accurate

| ID       | Acceptance Criterion                                                                         |
| -------- | -------------------------------------------------------------------------------------------- |
| AC-003.1 | WHEN a skeleton is generated THE SYSTEM SHALL include public signatures and type definitions |
| AC-003.2 | WHEN a symbol has documentation THE SYSTEM SHALL include a one-line summary                  |
| AC-003.3 | WHERE a user requests private symbols THE SYSTEM SHALL include them in the skeleton          |

---

### US-004: Symbol Queries

**As a** developer **I want** to query symbols by name and kind **So that**
tools can navigate code quickly

| ID       | Acceptance Criterion                                                                      |
| -------- | ----------------------------------------------------------------------------------------- |
| AC-004.1 | WHEN a user searches by name THE SYSTEM SHALL return matching symbols with file locations |
| AC-004.2 | WHEN a user filters by kind THE SYSTEM SHALL restrict results to that kind                |
| AC-004.3 | WHEN a symbol is selected THE SYSTEM SHALL return its signature and related edges         |

---

### US-005: Language Support

**As a** developer **I want** multi-language parsing **So that** monorepos are
supported

| ID       | Acceptance Criterion                                                   |
| -------- | ---------------------------------------------------------------------- |
| AC-005.1 | THE SYSTEM SHALL support a language plug-in model for extractors       |
| AC-005.2 | WHEN a language plug-in is missing THE SYSTEM SHALL degrade gracefully |

---

### US-006: Export to LLM Context

**As an** AI agent **I want** a minimal context export **So that** the LLM can
operate with high awareness

| ID       | Acceptance Criterion                                                            |
| -------- | ------------------------------------------------------------------------------- |
| AC-006.1 | WHEN exporting context THE SYSTEM SHALL include module paths and public symbols |
| AC-006.2 | WHEN a token budget is provided THE SYSTEM SHALL truncate deterministically     |

---

### US-006B: Public API Map

**As an** AI agent **I want** a public API map **So that** I can see the intended
surface area quickly

| ID        | Acceptance Criterion                                                              |
| --------- | --------------------------------------------------------------------------------- |
| AC-006B.1 | WHEN exports or re-exports are present THE SYSTEM SHALL record them in the index |
| AC-006B.2 | WHEN exporting context THE SYSTEM SHALL summarize public API by module            |

---

### US-006C: Stable Symbol Identity

**As an** AI agent **I want** stable symbol identifiers **So that** changes can
be tracked across indexing runs

| ID        | Acceptance Criterion                                                             |
| --------- | -------------------------------------------------------------------------------- |
| AC-006C.1 | THE SYSTEM SHALL assign a deterministic ID per symbol (based on qual name + kind) |
| AC-006C.2 | WHEN a symbol moves files THE SYSTEM SHALL preserve its ID if identity is stable |

---

### US-006D: Configurable Scope

**As a** developer **I want** configurable indexing scope **So that** monorepos
can be indexed selectively

| ID        | Acceptance Criterion                                                                |
| --------- | ----------------------------------------------------------------------------------- |
| AC-006D.1 | THE SYSTEM SHALL support include/exclude path patterns                              |
| AC-006D.2 | THE SYSTEM SHALL support language filters per index run                             |
| AC-006D.3 | WHERE a project scope is defined THE SYSTEM SHALL index only that subtree           |

---

### US-006E: Change Deltas

**As an** AI agent **I want** index diffs between runs **So that** I can refresh
context efficiently

| ID        | Acceptance Criterion                                                                    |
| --------- | --------------------------------------------------------------------------------------- |
| AC-006E.1 | WHEN an index exists THE SYSTEM SHALL produce a list of added/removed/changed symbols   |
| AC-006E.2 | WHEN exporting context THE SYSTEM SHALL allow a delta-only view                          |

---

## 2. Directionality & Importance

### US-007: Entrypoint Detection

**As an** AI agent **I want** the system to identify entrypoints **So that** I
can orient quickly in a new codebase

| ID       | Acceptance Criterion                                                                 |
| -------- | ------------------------------------------------------------------------------------ |
| AC-007.1 | WHEN indexing completes THE SYSTEM SHALL identify executable entrypoints (e.g., main) |
| AC-007.2 | WHEN indexing completes THE SYSTEM SHALL identify public API entrypoints (exports)  |
| AC-007.3 | WHEN exporting context THE SYSTEM SHALL include a list of entrypoints with kinds    |

---

### US-008: Usage and Call Directionality

**As an** AI agent **I want** usage and call directionality metrics **So that** I
can see what is used most and where flow begins

| ID       | Acceptance Criterion                                                                     |
| -------- | ---------------------------------------------------------------------------------------- |
| AC-008.1 | WHEN call edges can be resolved THE SYSTEM SHALL record call-in and call-out counts      |
| AC-008.2 | WHEN import/export relationships exist THE SYSTEM SHALL record import and export counts  |
| AC-008.3 | WHEN exporting context THE SYSTEM SHALL surface top-used symbols by usage score          |

---

### US-009: Importance Ranking

**As an** AI agent **I want** a notion of symbol importance **So that** I can
prioritize key components

| ID       | Acceptance Criterion                                                                 |
| -------- | ------------------------------------------------------------------------------------ |
| AC-009.1 | WHEN a symbol graph is available THE SYSTEM SHALL compute a centrality score         |
| AC-009.2 | WHEN exporting context THE SYSTEM SHALL provide importance rankings for symbols      |

---

### US-010: Complexity and Orchestrators

**As an** AI agent **I want** complexity and orchestration hints **So that** I
can focus on risky or coordinating code

| ID       | Acceptance Criterion                                                                  |
| -------- | ------------------------------------------------------------------------------------- |
| AC-010.1 | WHEN parsing functions THE SYSTEM SHALL estimate complexity (LOC, nesting, branching) |
| AC-010.2 | WHEN usage metrics are available THE SYSTEM SHALL classify orchestrators vs. leaves   |
| AC-010.3 | WHEN exporting context THE SYSTEM SHALL include complexity and role hints per symbol  |

---

## Non-Functional Requirements

### NFR-001: Performance

| ID        | Requirement                                                                     |
| --------- | ------------------------------------------------------------------------------- |
| NFR-001.1 | THE SYSTEM SHALL index a 100k LOC repo in under 60 seconds on a laptop          |
| NFR-001.2 | THE SYSTEM SHALL complete incremental re-index of a single file in under 250 ms |

### NFR-002: Reliability

| ID        | Requirement                                                             |
| --------- | ----------------------------------------------------------------------- |
| NFR-002.1 | THE SYSTEM SHALL avoid corrupting HelixDB on crash (use atomic updates) |
| NFR-002.2 | THE SYSTEM SHALL log parse failures with file and language info         |

### NFR-003: Usability

| ID        | Requirement                                                    |
| --------- | -------------------------------------------------------------- |
| NFR-003.1 | THE SYSTEM SHALL provide a human-readable skeleton export      |
| NFR-003.2 | THE SYSTEM SHALL provide a machine-readable export for tooling |
