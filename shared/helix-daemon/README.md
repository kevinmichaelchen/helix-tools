# helix-daemon

Global per-user daemon for helix-tools. Provides a shared IPC layer for background
sync, single-writer enforcement, and queueing across repos and tools.

## Responsibilities

- Accept IPC requests over local sockets
- Namespace requests by `{repo_root, tool}`
- Enforce per-repo writer locks
- Queue and coalesce sync work
- Report status for `--sync` and health checks

## IPC

- Unix socket: `~/.helix/run/helixd.sock`
- Windows: named pipe (path defined in specs)

## Specs

- `specs/requirements.md`
- `specs/design.md`
- `specs/tasks.md`
