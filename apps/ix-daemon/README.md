# ix-daemon

Global per-user daemon for Ixchel. Provides a shared IPC layer for background
sync, single-writer enforcement, and queueing across repos and tools.

## Responsibilities

- Accept IPC requests over local sockets
- Namespace requests by `{repo_root, tool}`
- Enforce per-repo writer locks
- Queue and coalesce sync work
- Report status for `--sync` and health checks

## IPC

- Unix socket: `~/.ixchel/run/ixcheld.sock`

## Platform Support

| Platform                      | Status               |
| ----------------------------- | -------------------- |
| macOS (Intel & Apple Silicon) | ✅ Supported         |
| Linux (x86_64 & ARM64)        | ✅ Supported         |
| Windows                       | ❌ Not yet supported |

Windows support requires rewriting the IPC layer to use named pipes instead of
Unix sockets. See [iss-7165bb](/.ixchel/issues/iss-7165bb.md).

## Specs

- [Requirements][requirements]
- [Design][design]
- [Tasks][tasks]

<!-- Links -->

[requirements]: specs/requirements.md
[design]: specs/design.md
[tasks]: specs/tasks.md
