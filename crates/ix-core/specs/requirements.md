# Requirements

This document defines requirements for `ix-core` (Ixchel core).

`ix-core` owns the git-first, Markdown-first domain model and validation rules for
Ixchel repositories.

## 1. Repository Discovery & Initialization

### US-001: Open an Ixchel repo

**As a** caller (CLI, MCP server)\
**I want to** open an Ixchel repo from any path within a git repository\
**So that** tools can operate without explicit root configuration

| ID       | Acceptance Criterion                                                              |
| -------- | --------------------------------------------------------------------------------- |
| AC-001.1 | WHEN `IxchelRepo::open_from(start)` is called THE SYSTEM SHALL require a git root |
| AC-001.2 | IF `.ixchel/` is missing THEN THE SYSTEM SHALL return an initialization error     |
| AC-001.3 | THE SYSTEM SHALL load config from `.ixchel/config.toml`                           |
| AC-001.4 | THE SYSTEM SHALL compute all on-disk paths relative to the git root               |

### US-002: Initialize an Ixchel repo

**As a** developer\
**I want to** initialize `.ixchel/` in a git repository\
**So that** Ixchel artifacts have a canonical location and safe cache directories

| ID       | Acceptance Criterion                                                                              |
| -------- | ------------------------------------------------------------------------------------------------- |
| AC-002.1 | WHEN `IxchelRepo::init_from(start, force)` is called THE SYSTEM SHALL require a git root          |
| AC-002.2 | THE SYSTEM SHALL create `.ixchel/` and standard subdirectories (issues, decisions, sources, etc.) |
| AC-002.3 | THE SYSTEM SHALL write `.ixchel/config.toml` if missing (or when `force=true`)                    |
| AC-002.4 | THE SYSTEM SHALL ensure `.gitignore` contains `.ixchel/data/` and `.ixchel/models/`               |
| AC-002.5 | IF `.ixchel/` exists and `force=false` THEN THE SYSTEM SHALL return an error                      |

## 2. Markdown Entities

### US-003: Create entities

**As a** caller\
**I want to** create new entities as Markdown files with YAML frontmatter\
**So that** the git repo remains the canonical source of truth

| ID       | Acceptance Criterion                                                                 |
| -------- | ------------------------------------------------------------------------------------ |
| AC-003.1 | WHEN `create_entity(kind, title, status)` is called THE SYSTEM SHALL create a file   |
| AC-003.2 | THE SYSTEM SHALL write YAML frontmatter with `id`, `type`, `title`, timestamps, tags |
| AC-003.3 | THE SYSTEM SHALL store entities under `.ixchel/<kind directory>/`                    |
| AC-003.4 | THE SYSTEM SHALL not overwrite an existing file with the same id                     |

### US-004: Read and list entities

| ID       | Acceptance Criterion                                                             |
| -------- | -------------------------------------------------------------------------------- |
| AC-004.1 | WHEN `list(kind)` is called THE SYSTEM SHALL return summaries for matching files |
| AC-004.2 | WHEN `read_raw(id)` is called THE SYSTEM SHALL read the Markdown file by id      |
| AC-004.3 | IF the id prefix is unknown THEN THE SYSTEM SHALL return an error                |

### US-005: Delete entities

| ID       | Acceptance Criterion                                                              |
| -------- | --------------------------------------------------------------------------------- |
| AC-005.1 | WHEN `delete_entity(id)` is called THE SYSTEM SHALL delete the corresponding file |
| AC-005.2 | IF the entity does not exist THEN THE SYSTEM SHALL return an error                |

## 3. Relationships & Validation

### US-006: Link/unlink entities

**As a** caller\
**I want to** express relationships in frontmatter\
**So that** graph structure is human-editable and diff-friendly

| ID       | Acceptance Criterion                                                                           |
| -------- | ---------------------------------------------------------------------------------------------- |
| AC-006.1 | WHEN `link(from, rel, to)` is called THE SYSTEM SHALL append `to` to `rel` frontmatter list    |
| AC-006.2 | WHEN `unlink(from, rel, to)` is called THE SYSTEM SHALL remove `to` and report whether changed |
| AC-006.3 | THE SYSTEM SHALL update `updated_at` when a link changes                                       |

### US-007: Check repository integrity

| ID       | Acceptance Criterion                                                                 |
| -------- | ------------------------------------------------------------------------------------ |
| AC-007.1 | WHEN `check()` is called THE SYSTEM SHALL detect missing ids and duplicate ids       |
| AC-007.2 | THE SYSTEM SHALL verify id prefixes match the directory kind                         |
| AC-007.3 | THE SYSTEM SHALL verify file names match `<id>.md`                                   |
| AC-007.4 | THE SYSTEM SHALL treat non-metadata frontmatter keys as relationships                |
| AC-007.5 | THE SYSTEM SHALL only treat values shaped like `<prefix>-<6..12 hex>` as id targets  |
| AC-007.6 | IF an id prefix is unknown THEN THE SYSTEM SHALL report an “unknown id prefix” error |
| AC-007.7 | IF a referenced id does not exist THEN THE SYSTEM SHALL report a broken link error   |

## 4. Index Abstraction

### US-008: Storage adapter interface

| ID       | Acceptance Criterion                                                                 |
| -------- | ------------------------------------------------------------------------------------ |
| AC-008.1 | THE SYSTEM SHALL define an `IndexBackend` trait for `sync`, `search`, `health_check` |
| AC-008.2 | `ix-core` SHALL NOT depend on concrete storage backends (adapters live elsewhere)    |

## 5. Tag Aggregation

### US-009: Collect tags across repository

**As a** caller (CLI, MCP server)\
**I want to** aggregate all tags from entity frontmatter\
**So that** agents and users can discover the existing tag vocabulary

| ID       | Acceptance Criterion                                                                      |
| -------- | ----------------------------------------------------------------------------------------- |
| AC-009.1 | WHEN `collect_tags(kind)` is called THE SYSTEM SHALL scan all entities (or only `kind`) for `tags` frontmatter |
| AC-009.2 | THE SYSTEM SHALL return a map of tag → list of entity ids                                 |
| AC-009.3 | THE SYSTEM SHALL treat tags as case-sensitive, trimmed strings and ignore empty values    |
| AC-009.4 | THE SYSTEM SHALL include each entity id at most once per tag                              |
| AC-009.5 | THE SYSTEM SHALL handle entities without tags gracefully (skip them)                      |
