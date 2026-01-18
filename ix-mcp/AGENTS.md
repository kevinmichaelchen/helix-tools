# IXCHEL AGENTS (ix-mcp)

## Scope

Applies to the `ix-mcp/` crate.

## Guidelines

- Keep transport concerns (MCP) here; keep domain logic in `ix-core`.
- Ensure responses are stable and easy for agents to consume.

## Commands

```bash
cargo test -p ix-mcp
dprint fmt
```
