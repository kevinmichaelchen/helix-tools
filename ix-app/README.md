# ix-app

Wiring/orchestration layer for Ixchel apps.

This crate exists to keep:

- `ix-core` adapter-free (domain logic + traits only)
- apps (`ix-cli`, `ix-mcp`) thin and backend-agnostic

## Responsibilities

- Select concrete backends from `repo.config.storage.backend`
- Call into `ix-core` traits (`IndexBackend`) using those backends

## Kiro Specs

- [Requirements][requirements]
- [Design][design]
- [Tasks][tasks]

<!-- Links -->

[requirements]: specs/requirements.md
[design]: specs/design.md
[tasks]: specs/tasks.md
