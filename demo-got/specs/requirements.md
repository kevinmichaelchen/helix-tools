# Requirements

This document defines requirements for `demo-got`.

`demo-got` is a demonstration crate that showcases HelixDB’s embedded graph +
vector capabilities using a small Game of Thrones family tree dataset.

## 1. Ingestion

### US-001: Ingest a family tree dataset

| ID       | Acceptance Criterion                                                                |
| -------- | ----------------------------------------------------------------------------------- |
| AC-001.1 | WHEN a user runs `demo-got ingest` THE SYSTEM SHALL load `data/westeros.yaml`       |
| AC-001.2 | THE SYSTEM SHALL create a local `.data/` directory for persisted HelixDB data       |
| AC-001.3 | THE SYSTEM SHALL insert PERSON nodes and relationship edges into HelixDB            |
| AC-001.4 | WHERE `--clear` is provided THE SYSTEM SHALL delete existing `.data/` before ingest |

### US-002: Optional embedding generation

| ID       | Acceptance Criterion                                                                   |
| -------- | -------------------------------------------------------------------------------------- |
| AC-002.1 | BY DEFAULT the ingest command SHALL load biographies from `data/*.md`                  |
| AC-002.2 | BY DEFAULT the ingest command SHALL generate embeddings and store vectors for search   |
| AC-002.3 | WHERE `--skip-embeddings` is provided THE SYSTEM SHALL ingest without creating vectors |

## 2. Graph Queries

### US-003: Ancestor/descendant traversal

| ID       | Acceptance Criterion                                                                       |
| -------- | ------------------------------------------------------------------------------------------ |
| AC-003.1 | WHEN a user runs `demo-got query ancestors <person>` THE SYSTEM SHALL return ancestors     |
| AC-003.2 | WHEN a user runs `demo-got query descendants <person>` THE SYSTEM SHALL return descendants |
| AC-003.3 | The system SHALL traverse `PARENT_OF` edges with correct directionality                    |

### US-004: House membership and person views

| ID       | Acceptance Criterion                                                                     |
| -------- | ---------------------------------------------------------------------------------------- |
| AC-004.1 | WHEN a user runs `demo-got query house <house>` THE SYSTEM SHALL list house members      |
| AC-004.2 | WHEN a user runs `demo-got query person <person>` THE SYSTEM SHALL return person details |

## 3. Semantic Search

### US-005: Search biographies

| ID       | Acceptance Criterion                                                              |
| -------- | --------------------------------------------------------------------------------- |
| AC-005.1 | WHEN a user runs `demo-got search <query>` THE SYSTEM SHALL return ranked matches |
| AC-005.2 | WHERE `--limit` is provided THE SYSTEM SHALL cap results                          |

## 4. Output

### US-006: JSON output mode

| ID       | Acceptance Criterion                                                   |
| -------- | ---------------------------------------------------------------------- |
| AC-006.1 | WHERE `--json` is provided THE SYSTEM SHALL emit machine-readable JSON |
