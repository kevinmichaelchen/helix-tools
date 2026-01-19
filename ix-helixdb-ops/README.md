# ix-helixdb-ops

HelixDB graph storage helper functions shared across this workspace.

This crate is intentionally HelixDB-specific and should not contain domain logic.

## What It Provides

- Node/edge write helpers (`put_node`, `put_edge`)
- Secondary index helpers (`update_secondary_indices`, `lookup_secondary_index`)
- Simple adjacency helpers (`outgoing_neighbors`, `incoming_neighbors`)

## Kiro Specs

- [Requirements][requirements]
- [Design][design]
- [Tasks][tasks]

<!-- Links -->

[requirements]: specs/requirements.md
[design]: specs/design.md
[tasks]: specs/tasks.md
