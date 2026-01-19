# ix-app Requirements

## Goals

- Provide a thin wiring layer that keeps `ix-core` backend-agnostic.
- Allow apps to call `sync` and `search` without depending on concrete backends.

## Acceptance Criteria

- `ix-app` exposes `sync()` and `search()` APIs using `ix-core` domain types.
- Backend selection is driven by `repo.config.storage.backend`.
- Unsupported backends return a clear error message.
