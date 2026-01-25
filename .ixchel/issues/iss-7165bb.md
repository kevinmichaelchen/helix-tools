---
id: iss-7165bb
type: issue
title: Add Windows support via named pipes IPC
status: backlog
created_at: 2026-01-25T19:41:05Z
updated_at: 2026-01-25T19:41:05Z
created_by: kevinchen
tags: []
---

## Problem

The `ix-daemon` uses Unix sockets for IPC (`tokio::net::UnixStream`), which
aren't available on Windows. This blocks Windows support entirely — the build
fails with type inference errors because `UnixStream` doesn't exist on Windows.

### Current state

- IPC: `~/.ixchel/run/ixcheld.sock` (Unix socket)
- Windows build: ❌ Fails

## Plan

- [ ] Research cross-platform IPC options (named pipes, `interprocess` crate)
- [ ] Design abstraction layer for platform-specific IPC
- [ ] Implement Windows named pipe support
- [ ] Add conditional compilation for Unix vs Windows
- [ ] Update CI to include Windows builds
- [ ] Add integration tests for Windows IPC

## Notes

### Implementation options

**Option A: Conditional compilation**

```rust
#[cfg(unix)]
use tokio::net::{UnixStream, UnixListener};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ServerOptions, ClientOptions};
```

**Option B: Use `interprocess` crate**

- Cross-platform abstraction over Unix sockets and Windows named pipes
- https://crates.io/crates/interprocess

### Files to modify

- `apps/ix-daemon/src/server.rs`
- `apps/ix-daemon/src/client.rs`
- Possibly extract to `crates/ix-ipc/` for reuse
