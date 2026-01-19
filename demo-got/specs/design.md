# Design

**Crate:** `demo-got`\
**Purpose:** HelixDB graph + vector demo using a GoT family tree dataset

## Overview

`demo-got` is a small, deterministic dataset + CLI used to validate and
demonstrate:

- graph ingestion (nodes/edges)
- graph traversal queries (ancestors/descendants)
- vector embeddings + semantic search over bios

## Data Sources

- `data/westeros.yaml`: structured seed data (people + relationships)
- `data/*.md`: Markdown biographies used for semantic embeddings

## Storage

- Local persisted DB: `demo-got/.data/` (gitignored)

## Graph Model

Nodes:

- Label: `PERSON`
- Properties: `id`, `name`, `house`, `titles`, `alias`, `is_alive`, `vector_id`

Edges:

- `PARENT_OF` (directional)
- `SPOUSE_OF` (bidirectional)
- `SIBLING_OF` (bidirectional)

## Embedding Text

Biography embeddings use a composite text that combines person metadata with the
bio Markdown content:

```
{name} ({alias})
Titles: {titles}

{bio markdown}
```
