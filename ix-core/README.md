# ix-core

Core library for **Ixchel** (ik-SHEL): a git-first, knowledge-first system that
weaves durable Markdown artifacts into a queryable graph + vector index.

## Responsibilities

- Own the main orchestration façade used by apps (`ix-cli`, `ix-mcp`).
- Define “ports” (traits) for storage/backends to keep adapters swappable.
- Keep business rules close to the domain (validation, sync decisions, context assembly).
