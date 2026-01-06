# Design

This document defines the architecture and protocol for `helix-daemon` (helixd).

## Overview

helixd is a global per-user daemon that provides:

- A shared IPC layer for helix-tools
- A single-writer gate for per-repo LMDB access
- A queue for background sync work

Requests are namespaced by `{repo_root, tool}` to keep data scoped to a repo.

## Architecture

```
CLI (helix-decisions, hbd, helix-docs, helix-map, ...)
    │
    ├─ JSON line IPC over local socket
    │
helixd (global per-user)
    ├─ Router (by {repo_root, tool})
    ├─ Queue manager (coalesce)
    ├─ Lock manager (per-repo writer lock)
    └─ Worker pool (runs sync jobs)
```

## Transport

- **Unix socket:** `~/.helix/run/helixd.sock`
- **Windows:** named pipe (implementation-specific path)

## IPC Protocol (v1)

All messages are UTF-8 JSON, one object per line (no newlines inside objects). The daemon
responds once per request. Requests and responses are correlated by `id`.

### Request Envelope
```json
{
  "version": 1,
  "id": "uuid",
  "repo_root": "/abs/path/to/repo",
  "tool": "decisions",
  "command": "enqueue_sync",
  "payload": {}
}
```

### Response Envelope
```json
{
  "version": 1,
  "id": "uuid",
  "status": "ok",
  "payload": {}
}
```

### Error Response
```json
{
  "version": 1,
  "id": "uuid",
  "status": "error",
  "error": { "code": "invalid_request", "message": "missing repo_root" }
}
```

### Commands

- `ping` → payload `{}`; response payload `{ "daemon_version": "x.y.z" }`
- `enqueue_sync` → payload `{ "directory": ".decisions", "force": false }`
  - response payload `{ "sync_id": "uuid", "queued_at_ms": 0 }`
- `wait_sync` → payload `{ "sync_id": "uuid", "timeout_ms": 30000 }`
  - response payload `{ "sync_id": "uuid", "state": "done", "stats": {...} }`
- `status` → payload `{ "repo_root": "...", "tool": "decisions" }` (both optional)
  - response payload `{ "queues": [...], "uptime_ms": 0 }`
- `shutdown` → payload `{ "reason": "dev" }` (dev/test only)

### Error Codes

- `invalid_request`
- `incompatible_version`
- `repo_not_found`
- `timeout`
- `internal_error`

## Lifecycle

- **Startup:** CLI attempts to connect; on failure, it starts `helixd` and retries.
- **Idle shutdown:** Daemon exits after `idle_timeout_ms` with no active queues.
- **Queueing:** One queue per `{repo_root, tool}`; multiple requests coalesce into a single
  pending sync.
- **Execution:** At most one active writer per repo; the daemon may process different repos
  sequentially or in parallel depending on resource limits.
- **`--sync` behavior:** CLI enqueues a sync and waits on `wait_sync` until completion or
  timeout. If the daemon cannot be started, CLI falls back to a direct sync.
